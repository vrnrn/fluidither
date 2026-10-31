//! Loader, Skeleton, Progress, Divider, Sparkline and Stat. Anything animated steps on
//! a slow timer, because an animation frame would redraw the whole window at display
//! rate just to move a few cells.

use crate::dither::{dither_tones, fill_sparkline, Matrix};
use crate::dither_gradient::expand_to_image;
use crate::generated::{DITHER_HOVER_CELL, RADIUS_PILL, WEIGHT_STRONG};
use crate::paint::checker;
use crate::theme::{fade, Theme};
use gpui::{
    canvas, div, fill, point, prelude::*, px, relative, size, App, Bounds, Context, Corners,
    ElementId, FontWeight, Hsla, IntoElement, RenderImage, RenderOnce, Rgba, SharedString, Task,
    Timer, Window,
};
use std::sync::Arc;
use std::time::Duration;

const COUNT_STEP: Duration = Duration::from_millis(300);
const DRIFT_STEP: Duration = Duration::from_millis(400);
const SPARK_FADE: f64 = 0.6;

/// A counter that ticks on a timer while something shows it.
struct Ticker {
    count: usize,
    timer: Option<Task<()>>,
}

impl Ticker {
    fn new() -> Self {
        Self { count: 0, timer: None }
    }

    fn run(&mut self, every: Duration, cx: &mut Context<Self>) {
        if self.timer.is_some() {
            return;
        }
        self.timer = Some(cx.spawn(async move |this, cx| loop {
            Timer::after(every).await;
            let ticked = this.update(cx, |this, cx| {
                this.count = this.count.wrapping_add(1);
                cx.notify();
            });
            if ticked.is_err() {
                break;
            }
        }));
    }
}

fn ticks(id: ElementId, every: Duration, window: &mut Window, cx: &mut App) -> usize {
    let reduced = Theme::get(cx).reduced_motion;
    let state = window.use_keyed_state(id, cx, |_, _| Ticker::new());
    state.update(cx, |ticker, cx| {
        if !reduced {
            ticker.run(every, cx);
        }
        ticker.count
    })
}

/// Three cells counting up.
#[derive(IntoElement)]
pub struct Loader {
    id: ElementId,
}

impl Loader {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self { id: id.into() }
    }
}

impl RenderOnce for Loader {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let ink = Theme::get(cx).colors().ink;
        let reduced = Theme::get(cx).reduced_motion;
        let shown = if reduced { 3 } else { ticks(self.id, COUNT_STEP, window, cx) % 3 + 1 };
        let cell = |index: usize| div().size(px(3.)).when(index < shown, |this| this.bg(ink));
        div().flex().flex_none().items_center().gap(px(3.)).child(cell(0)).child(cell(1)).child(cell(2))
    }
}

/// A placeholder while content loads. The dither drifts one cell at a time.
#[derive(IntoElement)]
pub struct Skeleton {
    id: ElementId,
    height: f32,
}

impl Skeleton {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self { id: id.into(), height: 18.0 }
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }
}

impl RenderOnce for Skeleton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let ink = fade(Theme::get(cx).colors().ink, 0.16);
        let drift = ticks(self.id, DRIFT_STEP, window, cx) % 2;
        div()
            .relative()
            .w_full()
            .h(px(self.height))
            .child(div().absolute().top_0().bottom_0().left(px(drift as f32 * 2.)).right_0().child(checker(ink, 2.0, RADIUS_PILL)))
            .when(drift == 1, |this| {
                this.child(div().absolute().top_0().bottom_0().left_0().w(px(2.)).child(checker(ink, 2.0, 0.0)))
            })
    }
}

/// How far along something is, 0 to 1. Solid where done, dithered ahead.
#[derive(IntoElement)]
pub struct Progress {
    share: f32,
}

impl Progress {
    pub fn new(share: f32) -> Self {
        Self { share: share.clamp(0.0, 1.0) }
    }
}

impl RenderOnce for Progress {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let ink = Theme::get(cx).colors().ink;
        div()
            .relative()
            .w_full()
            .h(px(6.))
            .child(checker(fade(ink, 0.45), 2.0, 0.0))
            .child(div().absolute().h_full().w(relative(self.share)).rounded(px(1.)).bg(ink))
    }
}

/// A dithered rule that fades out at both ends.
#[derive(IntoElement)]
pub struct Divider;

impl RenderOnce for Divider {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let ink = Hsla::from(fade(Theme::get(cx).colors().ink, 0.6));
        canvas(
            |_, _, _| (),
            move |bounds: Bounds<gpui::Pixels>, _, window, _| {
                let width = f32::from(bounds.size.width);
                let edge = width * 0.15;
                let cell = DITHER_HOVER_CELL / 2.0;
                let mut x = 0.0;
                while x < width {
                    let strength = (x.min(width - x) / edge).min(1.0);
                    let origin = bounds.origin + point(px(x), px(0.));
                    let color = Hsla { a: ink.a * strength, ..ink };
                    window.paint_quad(fill(Bounds::new(origin, size(px(cell), px(cell))), color));
                    x += cell * 2.0;
                }
            },
        )
        .w_full()
        .h(px(2.))
    }
}

/// A number with its label and an optional line about how it changed.
#[derive(IntoElement)]
pub struct Stat {
    label: SharedString,
    value: SharedString,
    change: Option<SharedString>,
}

impl Stat {
    pub fn new(label: impl Into<SharedString>, value: impl Into<SharedString>) -> Self {
        Self { label: label.into(), value: value.into(), change: None }
    }

    pub fn change(mut self, change: impl Into<SharedString>) -> Self {
        self.change = Some(change.into());
        self
    }
}

impl RenderOnce for Stat {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = Theme::get(cx).colors();
        div()
            .flex()
            .flex_col()
            .gap(px(2.))
            .text_size(px(13.))
            .line_height(px(18.))
            .text_color(colors.ink)
            .child(div().text_color(colors.ink_soft).child(self.label))
            .child(div().text_size(px(24.)).line_height(px(30.)).font_weight(FontWeight(WEIGHT_STRONG)).child(self.value))
            .when_some(self.change, |this, change| this.child(div().text_color(colors.ink_soft).child(change)))
    }
}

/// A small dithered chart. Give it a size with `.h()` and `.w_full()`.
#[derive(IntoElement)]
pub struct Sparkline {
    id: ElementId,
    values: Vec<f64>,
    height: f32,
}

impl Sparkline {
    pub fn new(id: impl Into<ElementId>, values: impl IntoIterator<Item = f64>) -> Self {
        Self { id: id.into(), values: values.into_iter().collect(), height: 32.0 }
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }
}

/// Scales values to 0..1. A flat line sits at the bottom.
pub fn normalized(values: &[f64]) -> Vec<f64> {
    let low = values.iter().copied().fold(f64::INFINITY, f64::min);
    let high = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let range = if high > low { high - low } else { 1.0 };
    values.iter().map(|value| (value - low) / range).collect()
}

#[derive(Default)]
struct SparkState {
    key: Option<(Vec<u64>, usize, usize, usize, u32)>,
    image: Option<Arc<RenderImage>>,
    retired: Vec<Arc<RenderImage>>,
}

fn spark_image(values: &[f64], width: usize, height: usize, device_cell: usize, ink: Rgba) -> RenderImage {
    let columns = width.div_ceil(device_cell);
    let rows = height.div_ceil(device_cell);
    let mut tones = vec![0.0; columns * rows];
    let mut cells = vec![0u8; columns * rows];
    fill_sparkline(&mut tones, columns, rows, values, SPARK_FADE);
    dither_tones(&tones, columns, rows, Matrix::Bayer4.map(), &[0u8, 1], &mut cells);
    expand_to_image(&cells, columns, device_cell, width, height, ink)
}

impl RenderOnce for Sparkline {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let ink = Theme::get(cx).colors().ink;
        let values = normalized(&self.values);
        let state = window.use_keyed_state(self.id, cx, |_, cx| {
            cx.on_release(|state: &mut SparkState, cx| {
                for image in state.retired.drain(..).chain(state.image.take()) {
                    cx.drop_image(image, None);
                }
            })
            .detach();
            SparkState::default()
        });
        let painted = state.clone();
        canvas(
            move |bounds: Bounds<gpui::Pixels>, window, cx| {
                let scale = window.scale_factor();
                let width = (f32::from(bounds.size.width) * scale).ceil() as usize;
                let height = (f32::from(bounds.size.height) * scale).ceil() as usize;
                let device_cell = ((DITHER_HOVER_CELL * scale).round() as usize).max(1);
                if width == 0 || height == 0 || values.is_empty() {
                    return;
                }
                let key = (values.iter().map(|value| value.to_bits()).collect(), width, height, device_cell, u32::from(ink));
                state.update(cx, |state, _| {
                    if state.key.as_ref() != Some(&key) {
                        state.retired.extend(state.image.take());
                        state.image = Some(Arc::new(spark_image(&values, width, height, device_cell, ink)));
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
        .h(px(self.height))
    }
}
