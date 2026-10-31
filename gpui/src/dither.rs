//! Ordered dithering on the CPU, because GPUI has no public shader API. Plain
//! arithmetic on slices with no GPUI types. `web/src/core/dither.js` is the same
//! algorithm, and `fixtures/dither.json` holds both to the same cells.

use crate::generated::{BAYER_2, BAYER_4, BAYER_8, BLUE_NOISE_64, HALFTONE_8};
use std::f64::consts::{PI, TAU};
use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Matrix {
    Bayer2,
    #[default]
    Bayer4,
    Bayer8,
    BlueNoise,
    /// Round dots on a 45 degree grid, as in print.
    Halftone,
}

/// Thresholds in (0, 1), row by row. The map tiles in both directions.
pub struct ThresholdMap {
    pub size: usize,
    pub thresholds: Vec<f64>,
}

fn build_map(size: usize, ranks: &[u8], levels: f64) -> ThresholdMap {
    // Centered in each level, so no tone lands exactly on a threshold.
    let thresholds = ranks.iter().map(|rank| (f64::from(*rank) + 0.5) / levels).collect();
    ThresholdMap { size, thresholds }
}

impl Matrix {
    pub fn map(self) -> &'static ThresholdMap {
        static MAPS: [OnceLock<ThresholdMap>; 5] = [const { OnceLock::new() }; 5];
        match self {
            Matrix::Bayer2 => MAPS[0].get_or_init(|| build_map(2, &BAYER_2, 4.0)),
            Matrix::Bayer4 => MAPS[1].get_or_init(|| build_map(4, &BAYER_4, 16.0)),
            Matrix::Bayer8 => MAPS[2].get_or_init(|| build_map(8, &BAYER_8, 64.0)),
            Matrix::BlueNoise => MAPS[3].get_or_init(|| build_map(64, &BLUE_NOISE_64, 256.0)),
            Matrix::Halftone => MAPS[4].get_or_init(|| build_map(8, &HALFTONE_8, 64.0)),
        }
    }
}

/// The geometry of CSS gradients. Angles are CSS degrees. Centers run 0 to 1 from
/// the top left, and a radius of 1 reaches the farthest corner.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Shape {
    Linear { angle: f64 },
    Radial { center: (f64, f64), radius: f64 },
    Conic { center: (f64, f64), angle: f64 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gradient {
    pub shape: Shape,
    /// Tone at the start, 0 (paper) to 1 (ink).
    pub from: f64,
    /// Tone at the end.
    pub to: f64,
    /// How far drifting noise bends the bands, 0 to 1.
    pub warp: f64,
}

impl Default for Gradient {
    fn default() -> Self {
        Self { shape: Shape::Linear { angle: 180.0 }, from: 0.0, to: 1.0, warp: 0.12 }
    }
}

// Lattice noise. Integer hashing rather than a random source, so both targets
// get the same bits.
#[inline]
fn lattice(x: i32, y: i32) -> f64 {
    let mut mix = x.wrapping_mul(0x27d4eb2f) ^ y.wrapping_mul(0x165667b1);
    mix = (mix ^ ((mix as u32) >> 15) as i32).wrapping_mul(0x2c1b3c6d);
    mix = (mix ^ ((mix as u32) >> 12) as i32).wrapping_mul(0x297a2d39);
    f64::from((mix ^ ((mix as u32) >> 15) as i32) as u32) / 4294967296.0
}

#[inline]
fn ease(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
}

#[inline]
fn smooth_noise(x: f64, y: f64) -> f64 {
    let (left, top) = (x.floor(), y.floor());
    let across = ease(x - left);
    let down = ease(y - top);
    let (left, top) = (left as i64 as i32, top as i64 as i32);
    let upper = lattice(left, top);
    let lower = lattice(left, top.wrapping_add(1));
    let upper_row = upper + (lattice(left.wrapping_add(1), top) - upper) * across;
    let lower_row = lower + (lattice(left.wrapping_add(1), top.wrapping_add(1)) - lower) * across;
    upper_row + (lower_row - upper_row) * down
}

/// Two layers of smooth noise, 0 to 1.
#[inline]
pub fn grain(x: f64, y: f64) -> f64 {
    (2.0 * smooth_noise(x, y) + smooth_noise(2.03 * x + 41.0, 2.03 * y + 23.0)) / 3.0
}

const WARP_SCALE: f64 = 2.6;
const WARP_DRIFT_X: f64 = 0.07;
const WARP_DRIFT_Y: f64 = 0.05;

/// Fill `tones` (width * height) with a gradient at time `t`.
pub fn fill_gradient(tones: &mut [f64], width: usize, height: usize, gradient: &Gradient, t: f64) {
    let (columns, rows) = (width as f64, height as f64);
    let (mode, angle, center, radius) = match gradient.shape {
        Shape::Linear { angle } => (0, angle, (0.5, 0.5), 1.0),
        Shape::Radial { center, radius } => (1, 0.0, center, radius),
        Shape::Conic { center, angle } => (2, angle, center, 1.0),
    };
    let origin_x = center.0 * columns;
    let origin_y = center.1 * rows;
    let radians = angle * PI / 180.0;
    let (sin, cos) = (radians.sin(), radians.cos());
    let line_length = (columns * sin).abs() + (rows * cos).abs();
    let reach = origin_x.max(columns - origin_x).hypot(origin_y.max(rows - origin_y)) * radius;
    let noise_scale = WARP_SCALE / rows;
    let drift_x = t * WARP_DRIFT_X;
    let drift_y = t * WARP_DRIFT_Y;

    let mut index = 0;
    for row in 0..height {
        let cell_y = row as f64 + 0.5;
        let y = cell_y - origin_y;
        for column in 0..width {
            let cell_x = column as f64 + 0.5;
            let x = cell_x - origin_x;
            let mut along = match mode {
                0 => (x * sin - y * cos) / line_length + 0.5,
                1 => x.hypot(y) / reach,
                _ => {
                    let turn = x.atan2(-y) / TAU - angle / 360.0;
                    turn - turn.floor()
                }
            };
            if gradient.warp != 0.0 {
                let bend = grain(cell_x * noise_scale + drift_x, cell_y * noise_scale + drift_y);
                along += gradient.warp * (bend - 0.5);
            }
            tones[index] = gradient.from + (gradient.to - gradient.from) * along.clamp(0.0, 1.0);
            index += 1;
        }
    }
}

/// Pick a palette entry per cell. A tone falls between two neighbouring entries
/// and the threshold decides which.
pub fn dither_tones<T: Copy>(
    tones: &[f64],
    width: usize,
    height: usize,
    map: &ThresholdMap,
    palette: &[T],
    out: &mut [T],
) {
    let wrap = map.size - 1;
    let last = palette.len() - 1;
    let mut index = 0;
    for y in 0..height {
        let row = (y & wrap) * map.size;
        for x in 0..width {
            let scaled = tones[index] * last as f64;
            let lower = scaled.floor();
            let upper = scaled - lower > map.thresholds[row + (x & wrap)];
            out[index] = palette[(lower as usize + usize::from(upper)).min(last)];
            index += 1;
        }
    }
}

/// Fill `tones` with a sparkline: solid ink along the line, fading out below it.
/// `values` run 0 to 1, spread evenly across the width.
pub fn fill_sparkline(tones: &mut [f64], width: usize, height: usize, values: &[f64], fade: f64) {
    let span = values.len().saturating_sub(1).max(1);
    let tops: Vec<f64> = (0..width)
        .map(|column| {
            let along = (column as f64 + 0.5) / width as f64 * span as f64;
            let left = (along.floor() as usize).min(span - 1);
            let right = (left + 1).min(values.len() - 1);
            let value = values[left] + (values[right] - values[left]) * (along - left as f64);
            (1.0 - value) * (height as f64 - 1.0)
        })
        .collect();
    let mut index = 0;
    for row in 0..height {
        for top in &tops {
            let below = row as f64 - top;
            tones[index] = if below < 0.0 {
                0.0
            } else if below < 1.0 {
                1.0
            } else {
                fade * (1.0 - below / (height as f64 - top))
            };
            index += 1;
        }
    }
}

/// A horizontal stretch of inked cells.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Run {
    pub x: u32,
    pub y: u32,
    pub len: u32,
}

/// Merge neighbouring inked cells of each row into runs, so fewer quads get
/// painted. `radius` is in cells: runs are cut to the corners of a rounded box.
pub fn cell_runs(cells: &[u8], width: usize, height: usize, radius: f32, out: &mut Vec<Run>) {
    out.clear();
    for y in 0..height {
        let inset = corner_inset(y, height, radius);
        let row = &cells[y * width..(y + 1) * width];
        let last = width.saturating_sub(inset);
        let mut x = inset;
        while x < last {
            if row[x] == 0 {
                x += 1;
                continue;
            }
            let start = x;
            while x < last && row[x] != 0 {
                x += 1;
            }
            out.push(Run { x: start as u32, y: y as u32, len: (x - start) as u32 });
        }
    }
}

fn corner_inset(y: usize, height: usize, radius: f32) -> usize {
    let from_edge = y.min(height - 1 - y) as f32 + 0.5;
    if radius <= 0.0 || from_edge >= radius {
        return 0;
    }
    let rise = radius - from_edge;
    (radius - (radius * radius - rise * rise).sqrt()).round() as usize
}
