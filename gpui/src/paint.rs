//! Small paint helpers shared by components.

use gpui::{canvas, div, fill, point, prelude::*, px, size, Bounds, Div, Hsla, IntoElement, Rgba};

/// Fills its parent with a one-cell checker. GPUI clips to rectangles only, so
/// cells outside the rounded corners are left out here.
pub fn checker(ink: Rgba, cell: f32, radius: f32) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            let color = Hsla::from(ink);
            let width = f32::from(bounds.size.width);
            let height = f32::from(bounds.size.height);
            if color.a <= 0.0 || width <= 0.0 || height <= 0.0 {
                return;
            }
            for row in 0..(height / cell).ceil() as u32 {
                for column in (row % 2..(width / cell).ceil() as u32).step_by(2) {
                    let (left, top) = (column as f32 * cell, row as f32 * cell);
                    if in_corner(left + cell / 2., top + cell / 2., width, height, radius) {
                        continue;
                    }
                    let origin = bounds.origin + point(px(left), px(top));
                    let extent = size(px(cell.min(width - left)), px(cell.min(height - top)));
                    window.paint_quad(fill(Bounds::new(origin, extent), color));
                }
            }
        },
    )
    .absolute()
    .size_full()
}

fn in_corner(x: f32, y: f32, width: f32, height: f32, radius: f32) -> bool {
    let beyond_x = (radius - x).max(x - (width - radius));
    let beyond_y = (radius - y).max(y - (height - radius));
    beyond_x > 0.0 && beyond_y > 0.0 && beyond_x.hypot(beyond_y) > radius
}

const GLYPH_CELL: f32 = 2.0;

/// Draws a pixel mark from the tokens, centered in its parent.
pub fn glyph(rows: &'static [&'static str], ink: Rgba) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            let color = Hsla::from(ink);
            let wide = rows[0].len() as f32 * GLYPH_CELL;
            let tall = rows.len() as f32 * GLYPH_CELL;
            let left = (f32::from(bounds.size.width) - wide) / 2.0;
            let top = (f32::from(bounds.size.height) - tall) / 2.0;
            for (y, row) in rows.iter().enumerate() {
                for (x, mark) in row.bytes().enumerate() {
                    if mark == b'#' {
                        let offset = point(px(left + x as f32 * GLYPH_CELL), px(top + y as f32 * GLYPH_CELL));
                        let cell = Bounds::new(bounds.origin + offset, size(px(GLYPH_CELL), px(GLYPH_CELL)));
                        window.paint_quad(fill(cell, color));
                    }
                }
            }
        },
    )
    .absolute()
    .size_full()
}

/// A one-pixel ring `gap` outside its parent, for keyboard focus.
pub fn focus_ring(color: Rgba, radius: f32, gap: f32) -> Div {
    div()
        .absolute()
        .top(px(-gap - 1.))
        .bottom(px(-gap - 1.))
        .left(px(-gap - 1.))
        .right(px(-gap - 1.))
        .rounded(px(radius + gap + 1.))
        .border_1()
        .border_color(color)
}
