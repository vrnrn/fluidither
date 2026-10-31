//! What one dither frame costs on the CPU, with no window involved:
//! `cargo run --release --example dither_cost`.

use fluidither::dither::{cell_runs, dither_tones, fill_gradient, Gradient, Matrix, Run, Shape};
use std::time::Instant;

fn percentile(sorted: &[f64], share: f64) -> f64 {
    sorted[((sorted.len() - 1) as f64 * share) as usize]
}

fn time(mut work: impl FnMut(usize)) -> (f64, f64) {
    let mut samples: Vec<f64> = (0..300)
        .map(|frame| {
            let started = Instant::now();
            work(frame);
            started.elapsed().as_secs_f64() * 1000.0
        })
        .collect();
    samples.sort_by(f64::total_cmp);
    (percentile(&samples, 0.5), percentile(&samples, 0.95))
}

fn main() {
    let band = Gradient {
        shape: Shape::Linear { angle: 90.0 },
        from: 0.0,
        to: 0.75,
        warp: 0.12,
    };
    let sizes = [
        ("band 880x120 @3px", 293, 40),
        ("panel 1200x300 @3px", 400, 100),
        ("full 1920x1080 @3px", 640, 360),
        ("full 1920x1080 @1px", 1920, 1080),
    ];
    let map = Matrix::Bayer4.map();
    println!("{:<26} {:>9} {:>14} {:>14} {:>14}", "size", "cells", "noise p50/p95", "dither p50/p95", "runs p50/p95");
    for (name, width, height) in sizes {
        let mut tones = vec![0.0; width * height];
        let mut cells = vec![0u8; width * height];
        let mut runs: Vec<Run> = Vec::new();
        let tone = time(|frame| fill_gradient(&mut tones, width, height, &band, frame as f64 * 0.013));
        let dither = time(|_| dither_tones(&tones, width, height, map, &[0u8, 1], &mut cells));
        let run = time(|_| cell_runs(&cells, width, height, 0.0, &mut runs));
        println!(
            "{name:<26} {:>9} {:>6.2}/{:<7.2} {:>6.2}/{:<7.2} {:>6.2}/{:<7.2} ({} runs)",
            width * height,
            tone.0,
            tone.1,
            dither.0,
            dither.1,
            run.0,
            run.1,
            runs.len()
        );
    }
}
