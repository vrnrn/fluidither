//! Dithered charts as a pure function: data in, cells out. A cell holds 0 for empty or
//! a series number from 1. `web/src/core/charts.js` is the same code, and
//! `fixtures/charts.json` holds both to the same cells.

use crate::dither::ThresholdMap;
use std::f64::consts::{PI, TAU};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ChartKind {
    #[default]
    Bar,
    Area,
    Line,
    Donut,
    Radar,
    Heatmap,
    Gauge,
}

struct Canvas<'a> {
    out: &'a mut [u8],
    width: usize,
    map: &'a ThresholdMap,
}

impl Canvas<'_> {
    #[inline]
    fn inked(&mut self, x: usize, y: usize, which: usize, tone: f64) {
        let wrap = self.map.size - 1;
        if tone > self.map.thresholds[(y & wrap) * self.map.size + (x & wrap)] {
            self.out[y * self.width + x] = which as u8 + 1;
        }
    }
}

fn top_of(column: usize, width: usize, height: usize, values: &[f64], top: f64) -> f64 {
    let span = values.len().saturating_sub(1).max(1);
    let along = (column as f64 + 0.5) / width as f64 * span as f64;
    let left = (along.floor() as usize).min(span - 1);
    let right = (left + 1).min(values.len() - 1);
    let value = values[left] + (values[right] - values[left]) * (along - left as f64);
    (1.0 - value / top) * (height as f64 - 1.0)
}

fn largest(series: &[Vec<f64>]) -> f64 {
    let top = series.iter().flatten().copied().fold(0.0, f64::max);
    if top > 0.0 { top } else { 1.0 }
}

fn bars(series: &[Vec<f64>], width: usize, height: usize, top: f64, canvas: &mut Canvas) {
    let groups = series[0].len();
    let rows = height as f64;
    let group_width = width as f64 / groups as f64;
    let bar_width = group_width * 0.78 / series.len() as f64;
    for x in 0..width {
        let group = ((x as f64 / group_width).floor() as usize).min(groups - 1);
        let inset = x as f64 + 0.5 - group as f64 * group_width - group_width * 0.11;
        let which = (inset / bar_width).floor();
        if inset < 0.0 || which >= series.len() as f64 {
            continue;
        }
        if inset - which * bar_width > bar_width * 0.88 {
            continue;
        }
        let which = which as usize;
        let tall = series[which][group] / top * rows;
        let mut y = height as f64 - 1.0;
        while y >= rows - tall {
            let near_top = (y - (rows - tall)) / tall < 0.5;
            canvas.inked(x, y as usize, which, 0.55 + 0.45 * if near_top { 1.0 } else { 0.6 });
            y -= 1.0;
        }
    }
}

fn area(series: &[Vec<f64>], width: usize, height: usize, top: f64, canvas: &mut Canvas) {
    for (which, values) in series.iter().enumerate() {
        for x in 0..width {
            let line = top_of(x, width, height, values, top);
            for y in (line.ceil() as usize)..height {
                let below = y as f64 - line;
                let tone = if below < 1.0 { 1.0 } else { 0.7 * (1.0 - below / (height as f64 - line)) };
                canvas.inked(x, y, which, tone);
            }
        }
    }
}

fn line(series: &[Vec<f64>], width: usize, height: usize, top: f64, canvas: &mut Canvas) {
    for (which, values) in series.iter().enumerate() {
        let mut previous = top_of(0, width, height, values, top);
        for x in 0..width {
            let here = top_of(x, width, height, values, top);
            let from = previous.min(here).floor() as usize;
            let to = previous.max(here).ceil() as usize;
            for y in from..=to.min(height - 1) {
                canvas.inked(x, y, which, 1.0);
            }
            previous = here;
        }
    }
}

fn clockwise_turn(dx: f64, dy: f64) -> f64 {
    let turn = dx.atan2(-dy) / TAU;
    if turn < 0.0 { turn + 1.0 } else { turn }
}

fn donut(series: &[Vec<f64>], width: usize, height: usize, canvas: &mut Canvas) {
    let values = &series[0];
    let sum: f64 = values.iter().sum();
    let total = if sum != 0.0 { sum } else { 1.0 };
    let radius = (width.min(height) as f64) / 2.0 - 0.5;
    let hole = radius * 0.62;
    for y in 0..height {
        for x in 0..width {
            let dx = x as f64 + 0.5 - width as f64 / 2.0;
            let dy = y as f64 + 0.5 - height as f64 / 2.0;
            let distance = (dx * dx + dy * dy).sqrt();
            if distance > radius || distance < hole {
                continue;
            }
            let turn = clockwise_turn(dx, dy);
            let mut running = 0.0;
            let mut slice = 0;
            while slice < values.len() - 1 {
                running += values[slice] / total;
                if turn < running {
                    break;
                }
                slice += 1;
            }
            canvas.inked(x, y, slice, 0.5 + 0.5 * ((distance - hole) / (radius - hole)));
        }
    }
}

/// The straight edge of a regular polygon, as a share of its corner radius.
fn polygon_edge(share: f64, count: usize) -> f64 {
    let half = PI / count as f64;
    half.cos() / ((share * 2.0 - 1.0) * half).cos()
}

fn radar(series: &[Vec<f64>], width: usize, height: usize, top: f64, canvas: &mut Canvas) {
    let radius = (width.min(height) as f64) / 2.0 - 1.0;
    // A faint web first, so the shapes have something to be read against.
    let corners = series[0].len();
    for y in 0..height {
        for x in 0..width {
            let dx = x as f64 + 0.5 - width as f64 / 2.0;
            let dy = y as f64 + 0.5 - height as f64 / 2.0;
            let along = clockwise_turn(dx, dy) * corners as f64;
            let edge = radius * polygon_edge(along - along.floor(), corners);
            let distance = (dx * dx + dy * dy).sqrt();
            if (distance - edge).abs() < 0.7 || (distance - edge / 2.0).abs() < 0.7 {
                canvas.inked(x, y, 0, 0.3);
            }
        }
    }
    for (which, values) in series.iter().enumerate() {
        let count = values.len();
        for y in 0..height {
            for x in 0..width {
                let dx = x as f64 + 0.5 - width as f64 / 2.0;
                let dy = y as f64 + 0.5 - height as f64 / 2.0;
                let distance = (dx * dx + dy * dy).sqrt();
                let along = clockwise_turn(dx, dy) * count as f64;
                let left = (along.floor() as usize) % count;
                let share = along - along.floor();
                let a = values[left] / top;
                let b = values[(left + 1) % count] / top;
                // Straight edges between spokes, which a lerp of the reach would bow outward.
                let reach = radius * (a + (b - a) * share) * polygon_edge(share, count);
                if distance > reach {
                    continue;
                }
                canvas.inked(x, y, which, if reach - distance < 1.0 { 1.0 } else { 0.4 });
            }
        }
    }
}

fn heatmap(series: &[Vec<f64>], width: usize, height: usize, top: f64, canvas: &mut Canvas) {
    let rows = series.len();
    let columns = series[0].len();
    let cell_width = width as f64 / columns as f64;
    let cell_height = height as f64 / rows as f64;
    for y in 0..height {
        let row = ((y as f64 / cell_height).floor() as usize).min(rows - 1);
        if (y + 1) as f64 - row as f64 * cell_height >= cell_height {
            continue;
        }
        for x in 0..width {
            let column = ((x as f64 / cell_width).floor() as usize).min(columns - 1);
            if (x + 1) as f64 - column as f64 * cell_width >= cell_width {
                continue;
            }
            canvas.inked(x, y, 0, series[row][column] / top);
        }
    }
}

fn gauge(series: &[Vec<f64>], width: usize, height: usize, top: f64, canvas: &mut Canvas) {
    let share = (series[0][0] / series[0].get(1).copied().unwrap_or(top)).min(1.0);
    let radius = (width as f64 / 2.0).min(height as f64) - 1.0;
    let hole = radius * 0.7;
    let center_y = height as f64 - 0.5;
    for y in 0..height {
        for x in 0..width {
            let dx = x as f64 + 0.5 - width as f64 / 2.0;
            let dy = y as f64 + 0.5 - center_y;
            let distance = (dx * dx + dy * dy).sqrt();
            if distance > radius || distance < hole || dy > 0.0 {
                continue;
            }
            let turn = dy.atan2(dx) / PI + 1.0;
            canvas.inked(x, y, 0, if turn <= share { 1.0 } else { 0.18 });
        }
    }
}

/// Fill `out` (width * height) for a chart. A gauge takes one list of `[value, max]`.
/// A heatmap reads its lists as rows. A donut's slices are its first list, and each
/// slice counts as a series for color.
pub fn chart_cells(kind: ChartKind, series: &[Vec<f64>], width: usize, height: usize, map: &ThresholdMap, out: &mut [u8]) {
    out.fill(0);
    let top = largest(series);
    let mut canvas = Canvas { out, width, map };
    match kind {
        ChartKind::Bar => bars(series, width, height, top, &mut canvas),
        ChartKind::Area => area(series, width, height, top, &mut canvas),
        ChartKind::Line => line(series, width, height, top, &mut canvas),
        ChartKind::Donut => donut(series, width, height, &mut canvas),
        ChartKind::Radar => radar(series, width, height, top, &mut canvas),
        ChartKind::Heatmap => heatmap(series, width, height, top, &mut canvas),
        ChartKind::Gauge => gauge(series, width, height, top, &mut canvas),
    }
}
