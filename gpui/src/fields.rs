//! Animated tone fields. Plain arithmetic with no trig, so Rust and JS produce the same
//! bits: `wave` stands in for a sine. `web/src/core/fields.js` is the same code, and
//! `fixtures/fields.json` holds both to the same cells.

use crate::dither::grain;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Field {
    #[default]
    Noise,
    Waves,
    Plasma,
    Ripples,
    Tunnel,
    Metaballs,
    Orb,
    Stars,
    Aurora,
    Contours,
    Beams,
    Eclipse,
    Horizon,
    Mesh,
    Grid,
    Ridges,
    Silk,
    Smoke,
}

/// A smooth wave with period 1, from -1 to 1.
#[inline]
pub fn wave(t: f64) -> f64 {
    let turn = t - t.floor();
    let half = if turn < 0.5 { turn * 2.0 } else { (turn - 0.5) * 2.0 };
    let shape = 4.0 * half * (1.0 - half);
    if turn < 0.5 { shape } else { -shape }
}

#[inline]
fn clamp01(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

fn star_at(x: f64, y: f64, t: f64) -> f64 {
    let cell_x = (x * 14.0).floor();
    let cell_y = (y * 14.0).floor();
    let seed = grain(cell_x * 7.31, cell_y * 3.17);
    if seed < 0.52 {
        return 0.0;
    }
    let dx = x * 14.0 - cell_x - 0.5;
    let dy = y * 14.0 - cell_y - 0.5;
    let glow = 1.0 - (dx * dx + dy * dy).sqrt() * 3.0;
    clamp01(glow) * (0.65 + 0.35 * wave(t * 0.3 + seed * 5.0))
}

#[inline]
fn tone_at(field: Field, x: f64, y: f64, wide: f64, t: f64) -> f64 {
    let dx = x - wide / 2.0;
    let dy = y - 0.5;
    let distance = (dx * dx + dy * dy).sqrt();
    match field {
        Field::Waves => 0.5 + 0.5 * wave(y * 3.0 + 0.3 * wave(x * 1.1 + t * 0.09) + t * 0.05),
        Field::Plasma => {
            0.5 + 0.125
                * (wave(x * 1.3 + t * 0.07)
                    + wave(y * 1.9 - t * 0.05)
                    + wave((x + y) * 1.1 + t * 0.04)
                    + wave(distance * 2.7 - t * 0.09))
        }
        Field::Ripples => (0.5 + 0.5 * wave(distance * 5.0 - t * 0.3)) * clamp01(1.25 - distance * 1.4),
        Field::Tunnel => (0.5 + 0.5 * wave(0.35 / (distance + 0.04) + t * 0.25)) * clamp01(distance * 2.4),
        Field::Metaballs => {
            let mut sum = 0.0;
            for ball in 0..3 {
                let ball = f64::from(ball);
                let bx = wide / 2.0 + wave(t * 0.04 + ball * 0.31) * wide * 0.3;
                let by = 0.5 + wave(t * 0.055 + ball * 0.47) * 0.3;
                sum += 0.035 / ((x - bx) * (x - bx) + (y - by) * (y - by) + 0.002);
            }
            clamp01((sum - 0.6) * 0.5)
        }
        Field::Orb => {
            let radius = 0.42;
            if distance >= radius {
                return 0.0;
            }
            let depth = (radius * radius - distance * distance).sqrt() / radius;
            let light_x = wave(t * 0.04) * 0.55;
            clamp01(0.08 + 0.92 * (depth * 0.75 + (dx / radius) * light_x - (dy / radius) * 0.45))
        }
        Field::Stars => star_at(x, y, t),
        Field::Aurora => {
            let line = 0.42 + 0.14 * wave(x * 0.7 + t * 0.03);
            clamp01(1.0 - (y - line).abs() * 3.2) * (0.25 + 0.75 * grain(x * 3.0 + t * 0.09, y * 0.6 - t * 0.02))
        }
        Field::Contours => {
            let height = grain(x * 2.2 + t * 0.02, y * 2.2) * 9.0;
            if height - height.floor() < 0.16 { 0.9 } else { 0.06 }
        }
        Field::Beams => {
            let mut light: f64 = 0.0;
            for beam in 0..3 {
                let center = wide * (0.5 + 0.4 * wave(t * 0.03 + f64::from(beam) * 0.29));
                let reach = 1.0 - (x - center + (y - 0.5) * 0.4).abs() * 7.0;
                if reach > light {
                    light = reach;
                }
            }
            clamp01(light) * (1.1 - y * 0.8)
        }
        Field::Eclipse => {
            let disc = 0.3;
            if distance < disc {
                return 0.0;
            }
            let flicker = 0.75 + 0.5 * grain(dx * 5.0 + t * 0.1, dy * 5.0);
            clamp01((1.0 - (distance - disc) * 4.5) * flicker)
        }
        Field::Horizon => {
            if y < 0.5 {
                return 0.1 + (0.5 - y) * 0.2;
            }
            let depth = 0.25 / (y - 0.45) + t * 0.15;
            (if depth - depth.floor() < 0.5 { 0.85 } else { 0.15 }) * clamp01((y - 0.5) * 4.0)
        }
        Field::Mesh => {
            let mut sum = 0.0;
            for point in 0..3 {
                let point = f64::from(point);
                let px = wide * (0.5 + 0.42 * wave(t * 0.025 + point * 0.37));
                let py = 0.5 + 0.42 * wave(t * 0.03 + point * 0.61);
                sum += 0.45 / (1.0 + 7.0 * ((x - px) * (x - px) + (y - py) * (y - py)));
            }
            clamp01(sum)
        }
        Field::Grid => {
            if y < 0.42 {
                return 0.05;
            }
            let rise = y - 0.4;
            let across = dx / rise;
            let along = 0.3 / rise + t * 0.2;
            let line = across * 2.0 - (across * 2.0).floor() < 0.1 || along - along.floor() < 0.14;
            if line { clamp01(rise * 3.5) } else { 0.03 }
        }
        Field::Ridges => {
            let row = y * 8.0;
            let band = row.floor();
            let peak = grain(x * 3.0 + band * 5.0, t * 0.05) * 0.85;
            let height = row - band - (1.0 - peak);
            if height < 0.0 {
                0.0
            } else if height < 0.09 {
                1.0
            } else {
                0.14
            }
        }
        Field::Silk => 0.5 + 0.5 * wave(x * 1.2 + 0.4 * wave(y * 1.7 + t * 0.05) + 0.3 * wave(x * 0.7 + y * 0.9 - t * 0.04)),
        Field::Smoke => {
            let bend = grain(x * 2.0 + t * 0.06, y * 2.0 - t * 0.1);
            grain(x * 3.0 + bend * 2.0, y * 3.0 + bend * 2.0 - t * 0.08) * (1.25 - y * 0.9)
        }
        Field::Noise => grain(x * 4.0 + t * 0.12, y * 4.0 - t * 0.08),
    }
}

/// Fill `tones` (width * height) with a field at time `t`.
pub fn fill_field(tones: &mut [f64], width: usize, height: usize, field: Field, t: f64) {
    let wide = width as f64 / height as f64;
    let mut index = 0;
    for row in 0..height {
        let y = (row as f64 + 0.5) / height as f64;
        for column in 0..width {
            let x = (column as f64 + 0.5) / width as f64 * wide;
            tones[index] = tone_at(field, x, y, wide, t);
            index += 1;
        }
    }
}
