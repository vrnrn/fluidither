//! What an animated dither costs in a real GPUI window, for both paint paths:
//! `cargo run --release --example frame_bench`. Prints a table and quits.
//!
//! `compute` and `paint` are CPU time on the main thread. `paint` includes the
//! texture upload for images. GPU time is not measured directly: it only shows
//! up as a longer frame interval.

use fluidither::{DitherGradient, FrameStats, Gradient, PaintMode, Shape};
use gpui::{
    div, prelude::*, px, size, App, Application, Bounds, Context, Render, Timer, Window,
    WindowBounds, WindowOptions,
};
use std::cell::RefCell;
use std::process::Command;
use std::rc::Rc;
use std::time::{Duration, Instant};

const WARMUP: usize = 30;
const FRAMES: usize = 300;

struct Run {
    label: &'static str,
    width: f32,
    height: f32,
    cell: f32,
    mode: PaintMode,
}

const RUNS: &[Run] = &[
    Run { label: "band 848x120", width: 848., height: 120., cell: 3., mode: PaintMode::Quads },
    Run { label: "band 848x120", width: 848., height: 120., cell: 3., mode: PaintMode::Image },
    Run { label: "panel 1200x300", width: 1200., height: 300., cell: 3., mode: PaintMode::Quads },
    Run { label: "panel 1200x300", width: 1200., height: 300., cell: 3., mode: PaintMode::Image },
    Run { label: "large 1200x700", width: 1200., height: 700., cell: 3., mode: PaintMode::Quads },
    Run { label: "large 1200x700", width: 1200., height: 700., cell: 3., mode: PaintMode::Image },
    Run { label: "fine 1200x700 @1px", width: 1200., height: 700., cell: 1., mode: PaintMode::Quads },
    Run { label: "fine 1200x700 @1px", width: 1200., height: 700., cell: 1., mode: PaintMode::Image },
];

struct Bench {
    index: usize,
    stats: Rc<RefCell<FrameStats>>,
}

impl Render for Bench {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let run = &RUNS[self.index.min(RUNS.len() - 1)];
        div().size_full().bg(gpui::black()).child(
            DitherGradient::new(
                ("bench", self.index),
                Gradient {
                    shape: Shape::Linear { angle: 90.0 },
                    from: 0.0,
                    to: 0.75,
                    warp: 0.12,
                },
            )
            .speed(0.4)
            .fps(0.0)
            .cell(run.cell)
            .ink(gpui::white().into())
            .paint_mode(run.mode)
            .stats(self.stats.clone())
            .w(px(run.width))
            .h(px(run.height)),
        )
    }
}

fn resident_megabytes() -> f64 {
    let pid = std::process::id().to_string();
    let output = Command::new("ps").args(["-o", "rss=", "-p", &pid]).output().unwrap();
    String::from_utf8_lossy(&output.stdout).trim().parse::<f64>().unwrap_or(0.0) / 1024.0
}

fn percentile(values: &[f64], share: f64) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    sorted[((sorted.len() - 1) as f64 * share) as usize]
}

fn report(run: &Run, stats: &FrameStats, memory: (f64, f64)) {
    let us_to_ms = |values: &[u32]| values.iter().skip(WARMUP).map(|v| *v as f64 / 1000.0).collect::<Vec<_>>();
    let compute = us_to_ms(&stats.compute_us);
    let paint = us_to_ms(&stats.paint_us);
    let intervals: Vec<f64> = stats.frame_times.windows(2).skip(WARMUP)
        .map(|pair| (pair[1] - pair[0]).as_secs_f64() * 1000.0)
        .collect();
    let mode = match run.mode {
        PaintMode::Quads => "quads",
        PaintMode::Image => "image",
    };
    println!(
        "{:<20} {:<6} {:>8} {:>7} | {:>5.2} {:>5.2} | {:>5.2} {:>5.2} | {:>6.2} {:>6.2} {:>6.2} | {:>6.0} {:>6.0}",
        run.label, mode, stats.cells, stats.quads,
        percentile(&compute, 0.5), percentile(&compute, 0.95),
        percentile(&paint, 0.5), percentile(&paint, 0.95),
        percentile(&intervals, 0.5), percentile(&intervals, 0.95), percentile(&intervals, 0.99),
        memory.0, memory.1,
    );
}

fn main() {
    let launch = Instant::now();
    Application::new().run(move |cx: &mut App| {
        fluidither::init(cx);
        let stats = Rc::new(RefCell::new(FrameStats::default()));
        let bounds = Bounds::centered(None, size(px(1200.), px(700.)), cx);
        let window = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                {
                    let stats = stats.clone();
                    move |_, cx| cx.new(|_| Bench { index: 0, stats })
                },
            )
            .unwrap();
        cx.activate(true);
        println!("window open after {:?}", launch.elapsed());
        println!(
            "{:<20} {:<6} {:>8} {:>7} | {:>11} | {:>11} | {:>20} | {:>13}",
            "size", "paint", "cells", "quads", "compute ms", "paint ms", "frame interval ms", "memory MB"
        );
        println!(
            "{:<20} {:<6} {:>8} {:>7} | {:>5} {:>5} | {:>5} {:>5} | {:>6} {:>6} {:>6} | {:>6} {:>6}",
            "", "", "", "", "p50", "p95", "p50", "p95", "p50", "p95", "p99", "start", "end"
        );

        cx.spawn(async move |cx| {
            for index in 0..RUNS.len() {
                *stats.borrow_mut() = FrameStats::default();
                window
                    .update(cx, |bench, _, cx| {
                        bench.index = index;
                        cx.notify();
                    })
                    .ok();
                let start_memory = resident_megabytes();
                let started = Instant::now();
                while stats.borrow().frames < (WARMUP + FRAMES) as u64 && started.elapsed() < Duration::from_secs(30) {
                    Timer::after(Duration::from_millis(50)).await;
                }
                let stats = stats.borrow().clone();
                if stats.frames < (WARMUP + 2) as u64 {
                    println!("{}: only {} frames drew, window may be hidden", RUNS[index].label, stats.frames);
                    continue;
                }
                report(&RUNS[index], &stats, (start_memory, resident_megabytes()));
            }
            cx.update(|cx| cx.quit()).ok();
        })
        .detach();
    });
}
