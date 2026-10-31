//! A dithered chart with a legend. GPUI has no accessibility API, so unlike the web
//! version there is no way to expose the numbers to a screen reader.

use crate::charts::{chart_cells, ChartKind};
use crate::dither::Matrix;
use crate::dither_gradient::expand_palette;
use crate::generated::ThemeColors;
use crate::theme::Theme;
use gpui::{
    canvas, div, prelude::*, px, App, Bounds, Corners, ElementId, IntoElement, Pixels, RenderImage,
    RenderOnce, Rgba, SharedString, Window,
};
use std::sync::Arc;

const CHART_CELL: f32 = 2.0;

fn series_colors(colors: &ThemeColors) -> [Rgba; 7] {
    [colors.ink, colors.orange, colors.blue, colors.green, colors.violet, colors.pink, colors.red]
}

#[derive(IntoElement)]
pub struct Chart {
    id: ElementId,
    kind: ChartKind,
    series: Vec<Vec<f64>>,
    names: Vec<SharedString>,
    height: f32,
}

impl Chart {
    /// `series` is a list of value lists. A gauge takes one `[value, max]`, a heatmap
    /// reads its lists as rows, and a donut uses its first list as slices.
    pub fn new(id: impl Into<ElementId>, kind: ChartKind, series: impl IntoIterator<Item = Vec<f64>>) -> Self {
        Self { id: id.into(), kind, series: series.into_iter().collect(), names: Vec::new(), height: 160.0 }
    }

    /// Legend entries, in series order, or slice order for a donut.
    pub fn names<T: Into<SharedString>>(mut self, names: impl IntoIterator<Item = T>) -> Self {
        self.names = names.into_iter().map(Into::into).collect();
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }
}

#[derive(Default)]
struct ChartState {
    key: Option<(Vec<u64>, usize, usize, usize, Vec<u32>)>,
    image: Option<Arc<RenderImage>>,
    retired: Vec<Arc<RenderImage>>,
}

impl RenderOnce for Chart {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme_colors = Theme::get(cx).colors();
        let palette = series_colors(theme_colors);
        let state = window.use_keyed_state(self.id, cx, |_, cx| {
            cx.on_release(|state: &mut ChartState, cx| {
                for image in state.retired.drain(..).chain(state.image.take()) {
                    cx.drop_image(image, None);
                }
            })
            .detach();
            ChartState::default()
        });
        let painted = state.clone();
        let (kind, series) = (self.kind, self.series);

        let plot = canvas(
            move |bounds: Bounds<Pixels>, window, cx| {
                let scale = window.scale_factor();
                let device_cell = ((CHART_CELL * scale).round() as usize).max(1);
                let width = (f32::from(bounds.size.width) * scale).ceil() as usize;
                let height = (f32::from(bounds.size.height) * scale).ceil() as usize;
                if width == 0 || height == 0 || series.is_empty() {
                    return;
                }
                let bits = series.iter().flatten().map(|value| value.to_bits()).collect();
                let inks = palette.iter().map(|color| u32::from(*color)).collect();
                let key = (bits, width, height, kind as usize, inks);
                state.update(cx, |state, _| {
                    if state.key.as_ref() != Some(&key) {
                        let columns = width.div_ceil(device_cell);
                        let rows = height.div_ceil(device_cell);
                        let mut cells = vec![0u8; columns * rows];
                        chart_cells(kind, &series, columns, rows, Matrix::Bayer4.map(), &mut cells);
                        state.retired.extend(state.image.take());
                        state.image = Some(Arc::new(expand_palette(&cells, columns, device_cell, width, height, &palette)));
                        state.key = Some(key);
                    }
                });
            },
            move |bounds, _, window, cx| {
                let (image, retired) = painted.update(cx, |state, _| (state.image.clone(), std::mem::take(&mut state.retired)));
                if let Some(image) = image {
                    window.paint_image(bounds, Corners::default(), image, 0, false).ok();
                }
                for image in retired {
                    window.drop_image(image).ok();
                }
            },
        )
        .w_full()
        .h(px(self.height));

        let legend = self.names.into_iter().enumerate().map(|(index, name)| {
            div()
                .flex()
                .items_center()
                .gap(px(5.))
                .child(div().size(px(8.)).bg(palette[index % palette.len()]))
                .child(name)
        });

        div()
            .flex()
            .flex_col()
            .w_full()
            .child(plot)
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap(px(12.))
                    .mt(px(8.))
                    .text_size(px(12.))
                    .line_height(px(16.))
                    .text_color(theme_colors.ink_soft)
                    .children(legend),
            )
    }
}
