//! Save dither cells as a PNG, with `scale` pixels per cell, so the result stays crisp
//! however it is used.

use gpui::Rgba;
use image::{ImageResult, RgbaImage};
use std::path::Path;

/// `cells` holds `columns * rows` entries, and anything but 0 is inked.
pub fn save_png(path: impl AsRef<Path>, cells: &[u8], columns: usize, rows: usize, scale: u32, ink: Rgba) -> ImageResult<()> {
    let to_byte = |value: f32| (value * 255.0).round() as u8;
    let color = image::Rgba([to_byte(ink.r), to_byte(ink.g), to_byte(ink.b), to_byte(ink.a)]);
    let picture = RgbaImage::from_fn(columns as u32 * scale, rows as u32 * scale, |x, y| {
        if cells[(y / scale) as usize * columns + (x / scale) as usize] != 0 {
            color
        } else {
            image::Rgba([0, 0, 0, 0])
        }
    });
    picture.save(path)
}
