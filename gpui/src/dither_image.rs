//! A picture redrawn in dither cells. It reads the file on first draw, on the main
//! thread, so keep images small.

use crate::dither::{dither_tones, Matrix};
use crate::dither_gradient::expand_to_image;
use crate::theme::Theme;
use gpui::{canvas, prelude::*, App, Bounds, Corners, ElementId, IntoElement, Pixels, RenderImage, RenderOnce, Window};
use image::GrayAlphaImage;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(IntoElement)]
pub struct DitherImage {
    id: ElementId,
    path: PathBuf,
    matrix: Matrix,
    cell: f32,
    width: f32,
    height: f32,
}

impl DitherImage {
    pub fn new(id: impl Into<ElementId>, path: impl Into<PathBuf>) -> Self {
        Self { id: id.into(), path: path.into(), matrix: Matrix::Halftone, cell: 3.0, width: 240.0, height: 160.0 }
    }

    pub fn matrix(mut self, matrix: Matrix) -> Self {
        self.matrix = matrix;
        self
    }

    pub fn cell(mut self, cell: f32) -> Self {
        self.cell = cell;
        self
    }

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }
}

/// Ink is laid where the picture matches it, so a dark theme's light ink falls on the
/// bright parts and the picture never turns into a negative. Sampled at cell middles.
pub fn tones_from(picture: &GrayAlphaImage, columns: usize, rows: usize, light_ink: bool) -> Vec<f64> {
    let (width, height) = picture.dimensions();
    let mut tones = Vec::with_capacity(columns * rows);
    for row in 0..rows {
        for column in 0..columns {
            let x = ((column as f64 + 0.5) / columns as f64 * f64::from(width)) as u32;
            let y = ((row as f64 + 0.5) / rows as f64 * f64::from(height)) as u32;
            let [gray, alpha] = picture.get_pixel(x.min(width - 1), y.min(height - 1)).0;
            let brightness = f64::from(gray) / 255.0;
            // See-through parts stay paper, so logos on transparency work.
            tones.push((if light_ink { brightness } else { 1.0 - brightness }) * f64::from(alpha) / 255.0);
        }
    }
    tones
}

#[derive(Default)]
struct ImageState {
    picture: Option<GrayAlphaImage>,
    loaded: bool,
    key: Option<(usize, usize, u32)>,
    image: Option<Arc<RenderImage>>,
    retired: Vec<Arc<RenderImage>>,
}

impl RenderOnce for DitherImage {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let ink = Theme::get(cx).colors().ink;
        let state = window.use_keyed_state(self.id, cx, |_, cx| {
            cx.on_release(|state: &mut ImageState, cx| {
                for image in state.retired.drain(..).chain(state.image.take()) {
                    cx.drop_image(image, None);
                }
            })
            .detach();
            ImageState::default()
        });
        let painted = state.clone();
        let (path, matrix, cell) = (self.path, self.matrix, self.cell);
        canvas(
            move |bounds: Bounds<Pixels>, window, cx| {
                let scale = window.scale_factor();
                let device_cell = ((cell * scale).round() as usize).max(1);
                let width = (f32::from(bounds.size.width) * scale).ceil() as usize;
                let height = (f32::from(bounds.size.height) * scale).ceil() as usize;
                state.update(cx, |state, _| {
                    if !state.loaded {
                        state.loaded = true;
                        state.picture = image::open(&path).ok().map(|picture| picture.to_luma_alpha8());
                    }
                    let key = (width, height, u32::from(ink));
                    let Some(picture) = state.picture.as_ref().filter(|_| state.key != Some(key)) else { return };
                    let columns = width.div_ceil(device_cell);
                    let rows = height.div_ceil(device_cell);
                    let light_ink = 0.2126 * ink.r + 0.7152 * ink.g + 0.0722 * ink.b > 0.5;
                    let tones = tones_from(picture, columns, rows, light_ink);
                    let mut cells = vec![0u8; columns * rows];
                    dither_tones(&tones, columns, rows, matrix.map(), &[0u8, 1], &mut cells);
                    let image = expand_to_image(&cells, columns, device_cell, width, height, ink);
                    state.retired.extend(state.image.take());
                    state.image = Some(Arc::new(image));
                    state.key = Some(key);
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
        .w(gpui::px(self.width))
        .h(gpui::px(self.height))
    }
}
