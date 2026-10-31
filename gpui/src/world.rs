//! Dot maps and a globe from a coarse hand-drawn outline of the land, which is a sketch
//! and not geographic data. `web/src/core/world.js` is the same code, and
//! `fixtures/fields.json` holds both to the same cells.

use crate::generated::{WORLD_BITS, WORLD_HEIGHT, WORLD_WIDTH};
use std::f64::consts::PI;

const DOT_SPACING: usize = 3;
const TILT: f64 = 0.35;

/// Whether a place is land, with longitude and latitude in degrees.
pub fn is_land(longitude: f64, latitude: f64) -> bool {
    let x = ((longitude + 180.0) / 360.0 * WORLD_WIDTH as f64).floor() as i64;
    let y = ((90.0 - latitude) / 180.0 * WORLD_HEIGHT as f64).floor() as i64;
    let column = x.rem_euclid(WORLD_WIDTH as i64) as usize;
    let row = y.clamp(0, WORLD_HEIGHT as i64 - 1) as usize;
    let bit = row * WORLD_WIDTH + column;
    WORLD_BITS[bit >> 3] >> (bit & 7) & 1 == 1
}

/// A dotted map, wrapped sideways by `turn` degrees.
pub fn fill_map(tones: &mut [f64], width: usize, height: usize, turn: f64) {
    let mut index = 0;
    for y in 0..height {
        let latitude = 90.0 - (y as f64 + 0.5) / height as f64 * 180.0;
        for x in 0..width {
            let longitude = (x as f64 + 0.5) / width as f64 * 360.0 - 180.0 + turn;
            let dot = x % DOT_SPACING == 0 && y % DOT_SPACING == 0;
            tones[index] = if dot && is_land(longitude, latitude) { 1.0 } else { 0.0 };
            index += 1;
        }
    }
}

/// A globe seen from outside, turned `turn` degrees, lit from the front.
pub fn fill_globe(tones: &mut [f64], width: usize, height: usize, turn: f64) {
    let radius = width.min(height) as f64 / 2.0 - 1.0;
    let (tilt_sin, tilt_cos) = (TILT.sin(), TILT.cos());
    let mut index = 0;
    for y in 0..height {
        let dy = (height as f64 / 2.0 - y as f64 - 0.5) / radius;
        for x in 0..width {
            let dx = (x as f64 + 0.5 - width as f64 / 2.0) / radius;
            let reach = dx * dx + dy * dy;
            tones[index] = if reach >= 1.0 {
                0.0
            } else {
                let front = (1.0 - reach).sqrt();
                let up = dy * tilt_cos + front * tilt_sin;
                let depth = front * tilt_cos - dy * tilt_sin;
                let latitude = up.asin() * 180.0 / PI;
                let longitude = dx.atan2(depth) * 180.0 / PI + turn;
                // JS `%` keeps the sign, which Rust's does too.
                if is_land((longitude + 180.0) % 360.0 - 180.0, latitude) {
                    0.2 + 0.5 * front
                } else {
                    0.1 * front
                }
            };
            index += 1;
        }
    }
}
