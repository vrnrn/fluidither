//! A dithered gradient element. Cells come from `crate::dither` on the CPU and
//! reach the screen one of two ways, because GPUI has no public shader API:
//!
//! - `PaintMode::Image`, the default, expands cells to device pixels and uploads
//!   a texture. It stays cheap at any size and costs nothing while it is still.
//! - `PaintMode::Quads` paints one quad per run of inked cells. GPUI's cost per
//!   quad grows with their number, so this is only for small areas.
//!
//! `examples/frame_bench.rs` measures both.

use crate::dither::{cell_runs, dither_tones, fill_gradient, Gradient, Matrix, Run};
use crate::fields::{fill_field, Field};
use crate::world::{fill_globe, fill_map};
use crate::generated::{DITHER_CELL, DITHER_FPS};
use crate::theme::Theme;
use gpui::{
    canvas, div, fill, point, prelude::*, px, size, App, Background, Bounds, ContentMask, Corners,
    ElementId, Entity, Global, Hsla, IntoElement, Pixels, RenderImage, RenderOnce, Rgba,
    StyleRefinement, Styled, Timer, WeakEntity, Window,
};
use image::{Frame, RgbaImage};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PaintMode {
    #[default]
    Image,
    Quads,
}

/// What the last frames cost, for measuring. Times are in microseconds.
#[derive(Clone, Debug, Default)]
pub struct FrameStats {
    pub frames: u64,
    pub compute_us: Vec<u32>,
    pub paint_us: Vec<u32>,
    pub frame_times: Vec<Instant>,
    pub quads: usize,
    pub cells: usize,
}

const GLOBE_DEGREES_PER_SECOND: f64 = 12.0;

/// Where a dither's tones come from.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Source {
    Gradient(Gradient),
    Field(Field),
    Map,
    Globe,
}

#[derive(IntoElement)]
pub struct DitherGradient {
    id: ElementId,
    source: Source,
    matrix: Matrix,
    cell: f32,
    speed: f64,
    fps: f32,
    ink: Option<Rgba>,
    radius: f32,
    mode: PaintMode,
    stats: Option<Rc<RefCell<FrameStats>>>,
    style: StyleRefinement,
}

impl DitherGradient {
    pub fn new(id: impl Into<ElementId>, gradient: Gradient) -> Self {
        Self {
            id: id.into(),
            source: Source::Gradient(gradient),
            matrix: Matrix::default(),
            cell: DITHER_CELL,
            speed: 0.0,
            fps: DITHER_FPS,
            ink: None,
            radius: 0.0,
            mode: PaintMode::default(),
            stats: None,
            style: StyleRefinement::default(),
        }
    }

    /// An animated field instead of a gradient. It moves at speed 1 unless you say.
    pub fn field(id: impl Into<ElementId>, field: Field) -> Self {
        let mut this = Self::new(id, Gradient::default());
        this.source = Source::Field(field);
        this.speed = 1.0;
        this
    }

    /// A dotted world map, from a coarse sketch of the land.
    pub fn map(id: impl Into<ElementId>) -> Self {
        let mut this = Self::new(id, Gradient::default());
        this.source = Source::Map;
        this.cell = 2.0;
        this
    }

    /// A turning globe, from the same sketch.
    pub fn globe(id: impl Into<ElementId>) -> Self {
        let mut this = Self::new(id, Gradient::default());
        this.source = Source::Globe;
        this.cell = 2.0;
        this.speed = GLOBE_DEGREES_PER_SECOND;
        this
    }

    pub fn matrix(mut self, matrix: Matrix) -> Self {
        self.matrix = matrix;
        self
    }

    /// Cell size in logical pixels. It snaps to whole device pixels.
    pub fn cell(mut self, cell: f32) -> Self {
        self.cell = cell;
        self
    }

    /// How fast the noise drifts. 0, the default, draws one still frame.
    pub fn speed(mut self, speed: f64) -> Self {
        self.speed = speed;
        self
    }

    /// Frame rate cap. 0 repaints on every window frame, for measuring.
    pub fn fps(mut self, fps: f32) -> Self {
        self.fps = fps;
        self
    }

    /// The ink color. Defaults to the theme's.
    pub fn ink(mut self, ink: Rgba) -> Self {
        self.ink = Some(ink);
        self
    }

    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    pub fn paint_mode(mut self, mode: PaintMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn stats(mut self, stats: Rc<RefCell<FrameStats>>) -> Self {
        self.stats = Some(stats);
        self
    }
}

impl Styled for DitherGradient {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

#[derive(Clone, Copy, PartialEq)]
struct Key {
    columns: usize,
    rows: usize,
    device_cell: usize,
    device_width: usize,
    device_height: usize,
    source: Source,
    matrix: Matrix,
    frame: u64,
    ink: u32,
    mode: PaintMode,
}

struct GradientState {
    started: Instant,
    key: Option<Key>,
    tones: Vec<f64>,
    cells: Vec<u8>,
    runs: Vec<Run>,
    image: Option<Arc<RenderImage>>,
    retired: Vec<Arc<RenderImage>>,
}

/// One timer for every animated gradient, so GPUI redraws once per tick rather
/// than once per gradient.
#[derive(Default)]
struct DitherClock {
    watching: Vec<WeakEntity<GradientState>>,
    running: bool,
}

impl Global for DitherClock {}

fn tick_with_clock(state: &Entity<GradientState>, fps: f32, cx: &mut App) {
    let clock = cx.default_global::<DitherClock>();
    let weak = state.downgrade();
    if !clock.watching.contains(&weak) {
        clock.watching.push(weak);
    }
    if clock.running {
        return;
    }
    clock.running = true;
    let interval = Duration::from_secs_f32(1.0 / fps);
    cx.spawn(async move |cx| loop {
        Timer::after(interval).await;
        let keep_going = cx
            .update(|cx| {
                let watching = std::mem::take(&mut cx.global_mut::<DitherClock>().watching);
                let alive: Vec<_> = watching
                    .into_iter()
                    .filter(|weak| weak.update(cx, |_, cx| cx.notify()).is_ok())
                    .collect();
                let clock = cx.global_mut::<DitherClock>();
                clock.running = !alive.is_empty();
                clock.watching = alive;
                clock.running
            })
            .unwrap_or(false);
        if !keep_going {
            break;
        }
    })
    .detach();
}

impl GradientState {
    fn new() -> Self {
        Self {
            started: Instant::now(),
            key: None,
            tones: Vec::new(),
            cells: Vec::new(),
            runs: Vec::new(),
            image: None,
            retired: Vec::new(),
        }
    }

    fn compute(&mut self, key: Key, t: f64, radius_cells: f32, ink: Rgba, stats: Option<&Rc<RefCell<FrameStats>>>) {
        let started = Instant::now();
        let Key { columns, rows, source, matrix, mode, .. } = key;
        let count = columns * rows;
        self.tones.resize(count, 0.0);
        self.cells.resize(count, 0);
        match source {
            Source::Gradient(gradient) => fill_gradient(&mut self.tones, columns, rows, &gradient, t),
            Source::Field(field) => fill_field(&mut self.tones, columns, rows, field, t),
            Source::Map => fill_map(&mut self.tones, columns, rows, 0.0),
            Source::Globe => fill_globe(&mut self.tones, columns, rows, t),
        }
        dither_tones(&self.tones, columns, rows, matrix.map(), &[0u8, 1u8], &mut self.cells);
        self.retired.extend(self.image.take());
        match mode {
            PaintMode::Quads => cell_runs(&self.cells, columns, rows, radius_cells, &mut self.runs),
            PaintMode::Image => {
                self.runs.clear();
                self.image = Some(Arc::new(expand_to_image(&self.cells, columns, key.device_cell, key.device_width, key.device_height, ink)));
            }
        }
        self.key = Some(key);
        if let Some(stats) = stats {
            let mut stats = stats.borrow_mut();
            stats.compute_us.push(started.elapsed().as_micros() as u32);
            stats.quads = self.runs.len();
            stats.cells = count;
        }
    }
}

// GPUI samples images linearly, so a cell-sized image scaled up would blur.
// Each cell becomes `device_cell` real pixels instead. The image is the exact
// size of the box, so the last cells are cut rather than sticking out past it.
pub(crate) fn expand_to_image(cells: &[u8], columns: usize, device_cell: usize, width: usize, height: usize, ink: Rgba) -> RenderImage {
    expand_palette(cells, columns, device_cell, width, height, &[ink])
}

/// As above, with cell value `n` taking palette entry `n - 1` and 0 left empty.
pub(crate) fn expand_palette(cells: &[u8], columns: usize, device_cell: usize, width: usize, height: usize, palette: &[Rgba]) -> RenderImage {
    let to_byte = |value: f32| (value * 255.0).round() as u8;
    let colors: Vec<[u8; 4]> = palette.iter().map(|ink| [to_byte(ink.b), to_byte(ink.g), to_byte(ink.r), to_byte(ink.a)]).collect();
    let stride = width * 4;
    let mut data = vec![0u8; stride * height];
    for (row, block) in data.chunks_mut(stride * device_cell).enumerate() {
        let (line, rest) = block.split_at_mut(stride);
        for (part, cell) in line.chunks_mut(device_cell * 4).zip(&cells[row * columns..]) {
            if *cell != 0 {
                let color = colors[(*cell as usize - 1) % colors.len()];
                for pixel in part.chunks_exact_mut(4) {
                    pixel.copy_from_slice(&color);
                }
            }
        }
        for copy in rest.chunks_exact_mut(stride) {
            copy.copy_from_slice(line);
        }
    }
    let buffer = RgbaImage::from_raw(width as u32, height as u32, data).expect("buffer matches its size");
    RenderImage::new(vec![Frame::new(buffer)])
}

impl RenderOnce for DitherGradient {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let reduced = theme.reduced_motion;
        let ink = self.ink.unwrap_or(theme.colors().ink);
        let animated = self.speed > 0.0 && !reduced;
        let state = window.use_keyed_state(self.id.clone(), cx, |_, cx| {
            cx.on_release(|state: &mut GradientState, cx| {
                for image in state.retired.drain(..).chain(state.image.take()) {
                    cx.drop_image(image, None);
                }
            })
            .detach();
            GradientState::new()
        });

        if animated && self.fps > 0.0 {
            tick_with_clock(&state, self.fps, cx);
        }

        let Self { source, matrix, cell, speed, fps, radius, mode, stats, mut style, .. } = self;
        let mut element = div();
        *element.style() = std::mem::take(&mut style);
        let every_frame = animated && fps <= 0.0;
        let prepaint_state = state.clone();
        let paint_state = state;
        let paint_stats = stats.clone();

        element.child(
            canvas(
                move |bounds: Bounds<Pixels>, window, cx| {
                    let scale = window.scale_factor();
                    let device_cell = ((cell * scale).round() as usize).max(1);
                    let cell_px = device_cell as f32 / scale;
                    let columns = (f32::from(bounds.size.width) / cell_px).ceil() as usize;
                    let rows = (f32::from(bounds.size.height) / cell_px).ceil() as usize;
                    let device_width = (f32::from(bounds.size.width) * scale).ceil() as usize;
                    let device_height = (f32::from(bounds.size.height) * scale).ceil() as usize;
                    if columns == 0 || rows == 0 {
                        return cell_px;
                    }
                    prepaint_state.update(cx, |state, _| {
                        let elapsed = if animated { state.started.elapsed().as_secs_f64() } else { 0.0 };
                        let tick = if every_frame { DITHER_FPS as f64 * 4.0 } else { fps.max(1.0) as f64 };
                        let frame = (elapsed * tick) as u64;
                        let key = Key {
                            columns,
                            rows,
                            device_cell,
                            device_width,
                            device_height,
                            source,
                            matrix,
                            frame: if animated { frame } else { 0 },
                            ink: u32::from(ink),
                            mode,
                        };
                        if state.key != Some(key) {
                            let t = frame as f64 / tick * speed;
                            state.compute(key, t, radius / cell_px, ink, stats.as_ref());
                        }
                    });
                    cell_px
                },
                move |bounds, cell_px, window, cx| {
                    let started = Instant::now();
                    let color: Background = Hsla::from(ink).into();
                    let origin = bounds.origin;
                    let (image, retired) = paint_state.update(cx, |state, _| {
                        let mask = ContentMask { bounds };
                        window.with_content_mask(Some(mask), |window| {
                            for run in &state.runs {
                                let run_origin = point(
                                    origin.x + px(run.x as f32 * cell_px),
                                    origin.y + px(run.y as f32 * cell_px),
                                );
                                let run_size = size(px(run.len as f32 * cell_px), px(cell_px));
                                window.paint_quad(fill(Bounds::new(run_origin, run_size), color));
                            }
                        });
                        (state.image.clone(), std::mem::take(&mut state.retired))
                    });
                    if let Some(image) = image {
                        let size_in_pixels = image.size(0).map(|side| px(side.0 as f32 / window.scale_factor()));
                        let image_bounds = Bounds::new(origin, size_in_pixels);
                        window
                            .paint_image(image_bounds, Corners::all(px(radius)), image, 0, false)
                            .ok();
                    }
                    // Dropped after the replacement is in, or the atlas keeps every frame.
                    for image in retired {
                        window.drop_image(image).ok();
                    }
                    if let Some(stats) = &paint_stats {
                        let mut stats = stats.borrow_mut();
                        stats.frames += 1;
                        stats.paint_us.push(started.elapsed().as_micros() as u32);
                        stats.frame_times.push(Instant::now());
                    }
                    if every_frame {
                        window.request_animation_frame();
                    }
                },
            )
            .size_full(),
        )
    }
}
