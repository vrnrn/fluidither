//! Text that arrives with some character. All three settle at once under reduced
//! motion. `web/src/core/text.js` is the same scramble.

use crate::dither::{dither_tones, fill_shimmer, Gradient, Matrix, Shape};
use crate::dither_gradient::DitherGradient;
use crate::generated::SETTLE;
use crate::spring::{Spring, LONGEST_STEP};
use crate::theme::Theme;
use gpui::{canvas, div, fill, point, prelude::*, px, size, AnyElement, App, Bounds, Context, ElementId, IntoElement, Pixels, RenderOnce, Rgba, SharedString, Task, Timer, Window};
use std::rc::Rc;
use std::time::{Duration, Instant};

const GLYPHS: &[u8] = b"#%&*+=/<>?";
const SCRAMBLE_SECONDS: f64 = 0.7;
const SCRAMBLE_STEP: Duration = Duration::from_millis(40);
const DISSOLVE_SECONDS: f64 = 0.5;
const DISSOLVE_STEP: Duration = Duration::from_millis(40);

/// `text` part way through a scramble: letters before `progress` (0 to 1) are settled
/// and the rest are stand-ins picked by `seed`. Spaces stay, so words keep their shape.
pub fn scrambled(text: &str, progress: f64, seed: u32) -> String {
    let count = text.chars().count();
    let settled = (count as f64 * progress).floor() as usize;
    text.chars()
        .enumerate()
        .map(|(index, letter)| {
            if index < settled || letter == ' ' {
                return letter;
            }
            let mixed = (index as i32 + 7).wrapping_mul(seed as i32 + 13) as u32;
            GLYPHS[mixed as usize % GLYPHS.len()] as char
        })
        .collect()
}

/// A steady timer that redraws while an effect runs, then stops by itself.
struct Run {
    started: Instant,
    ticks: u32,
    timer: Option<Task<()>>,
    waiting: bool,
}

impl Run {
    fn new() -> Self {
        Self { started: Instant::now(), ticks: 0, timer: None, waiting: false }
    }

    fn keep_alive(&mut self, every: Duration, seconds: f64, cx: &mut Context<Self>) {
        if self.timer.is_some() || self.started.elapsed().as_secs_f64() >= seconds {
            return;
        }
        self.timer = Some(cx.spawn(async move |this, cx| loop {
            Timer::after(every).await;
            let running = this.update(cx, |run, cx| {
                run.ticks += 1;
                cx.notify();
                run.started.elapsed().as_secs_f64() < seconds + 0.05
            });
            if !matches!(running, Ok(true)) {
                break;
            }
        }));
    }
}

/// Letters settle from stand-in glyphs.
#[derive(IntoElement)]
pub struct Scramble {
    id: ElementId,
    text: SharedString,
}

impl Scramble {
    pub fn new(id: impl Into<ElementId>, text: impl Into<SharedString>) -> Self {
        Self { id: id.into(), text: text.into() }
    }
}

impl RenderOnce for Scramble {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        if Theme::get(cx).reduced_motion {
            return div().child(self.text);
        }
        let run = window.use_keyed_state(self.id, cx, |_, _| Run::new());
        let (progress, seed) = run.update(cx, |run, cx| {
            run.keep_alive(SCRAMBLE_STEP, SCRAMBLE_SECONDS, cx);
            (run.started.elapsed().as_secs_f64() / SCRAMBLE_SECONDS, run.ticks)
        });
        if progress >= 1.0 {
            return div().child(self.text);
        }
        div().child(scrambled(&self.text, progress, seed))
    }
}

const SHIMMER_CELL: f32 = 2.0;
const SHIMMER_STEP: Duration = Duration::from_millis(33);
const SHIMMER_WORD_SECONDS: f64 = 3.2;

/// A clear band sweeps through a dither over the text, moving on to the next word each few
/// sweeps. `paper` should match what is behind it.
#[derive(IntoElement)]
pub struct Shimmer {
    id: ElementId,
    words: Vec<SharedString>,
    paper: Option<Rgba>,
}

impl Shimmer {
    pub fn new<T: Into<SharedString>>(id: impl Into<ElementId>, words: impl IntoIterator<Item = T>) -> Self {
        Self { id: id.into(), words: words.into_iter().map(Into::into).collect(), paper: None }
    }

    pub fn paper(mut self, paper: Rgba) -> Self {
        self.paper = Some(paper);
        self
    }
}

impl RenderOnce for Shimmer {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let paper = self.paper.unwrap_or(theme.colors().paper);
        let reduced = theme.reduced_motion;
        let run = window.use_keyed_state(self.id, cx, |_, _| Run::new());
        let seconds = if reduced {
            0.0
        } else {
            run.update(cx, |run, cx| {
                run.keep_alive(SHIMMER_STEP, f64::INFINITY, cx);
                run.started.elapsed().as_secs_f64()
            })
        };
        let word = self.words[(seconds / SHIMMER_WORD_SECONDS) as usize % self.words.len()].clone();
        div().relative().child(word).child(
            canvas(
                |_, _, _| (),
                move |bounds: Bounds<Pixels>, _, window, _| {
                    if reduced {
                        return;
                    }
                    let columns = (f32::from(bounds.size.width) / SHIMMER_CELL).ceil() as usize;
                    let rows = (f32::from(bounds.size.height) / SHIMMER_CELL).ceil() as usize;
                    let mut tones = vec![0.0; columns * rows];
                    let mut cells = vec![0u8; columns * rows];
                    fill_shimmer(&mut tones, columns, rows, seconds);
                    dither_tones(&tones, columns, rows, Matrix::Bayer4.map(), &[0u8, 1], &mut cells);
                    for (index, _) in cells.iter().enumerate().filter(|(_, cell)| **cell == 1) {
                        let at = point(px((index % columns) as f32 * SHIMMER_CELL), px((index / columns) as f32 * SHIMMER_CELL));
                        window.paint_quad(fill(Bounds::new(bounds.origin + at, size(px(SHIMMER_CELL), px(SHIMMER_CELL))), paper));
                    }
                },
            )
            .absolute()
            .top_0()
            .left_0()
            .size_full(),
        )
    }
}

/// Content appears through a thinning dither. `paper` should match what is behind it.
#[derive(IntoElement)]
pub struct Dissolve {
    id: ElementId,
    paper: Option<Rgba>,
    content: AnyElement,
    hold: Option<(f64, f64)>,
    on_view: bool,
}

impl Dissolve {
    pub fn new(id: impl Into<ElementId>, content: impl IntoElement) -> Self {
        Self { id: id.into(), paper: None, content: content.into_any_element(), hold: None, on_view: false }
    }

    pub fn paper(mut self, paper: Rgba) -> Self {
        self.paper = Some(paper);
        self
    }

    /// Never clear. Keep a dither across the content, from `from` at the start to `to`
    /// at the end, for dithered headings and logos.
    pub fn hold(mut self, from: f64, to: f64) -> Self {
        self.hold = Some((from, to));
        self
    }

    /// Wait until it is inside the window.
    pub fn on_view(mut self) -> Self {
        self.on_view = true;
        self
    }
}

fn veil(id: ElementId, paper: Rgba, from: f64, to: f64) -> impl IntoElement {
    let tones = Gradient { shape: Shape::Linear { angle: 90.0 }, from, to, warp: 0.0 };
    div().absolute().top_0().left_0().size_full().child(DitherGradient::new(id, tones).matrix(Matrix::Bayer4).cell(2.0).ink(paper).size_full())
}

impl RenderOnce for Dissolve {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let paper = self.paper.unwrap_or(theme.colors().paper);
        let reduced = theme.reduced_motion;
        if let Some((from, to)) = self.hold {
            return div().relative().child(self.content).child(veil(self.id, paper, from, to)).into_any_element();
        }
        let waits = self.on_view;
        let run = window.use_keyed_state(self.id.clone(), cx, move |_, _| {
            let mut run = Run::new();
            run.waiting = waits;
            run
        });
        let left = run.update(cx, |run, cx| {
            if run.waiting {
                return 1.0;
            }
            run.keep_alive(DISSOLVE_STEP, DISSOLVE_SECONDS, cx);
            1.0 - run.started.elapsed().as_secs_f64() / DISSOLVE_SECONDS
        });
        let watch = run.clone();
        div()
            .relative()
            .child(self.content)
            .when(left > 0.0 && !reduced, |this| this.child(veil(self.id, paper, left, left)))
            .when(waits, |this| {
                this.child(
                    canvas(
                        move |bounds: Bounds<Pixels>, window, cx| {
                            let height = window.viewport_size().height;
                            if bounds.top() < height && bounds.bottom() > px(0.) {
                                watch.update(cx, |run, cx| {
                                    if run.waiting {
                                        run.waiting = false;
                                        run.started = Instant::now();
                                        cx.notify();
                                    }
                                });
                            }
                        },
                        |_, _, _, _| {},
                    )
                    .absolute()
                    .top_0()
                    .left_0()
                    .size_full(),
                )
            })
            .into_any_element()
    }
}

type Format = Rc<dyn Fn(f64) -> String>;

/// A number that springs to its value and keeps its speed when the value changes.
#[derive(IntoElement)]
pub struct Count {
    id: ElementId,
    value: f64,
    format: Format,
}

impl Count {
    pub fn new(id: impl Into<ElementId>, value: f64) -> Self {
        Self { id: id.into(), value, format: Rc::new(|value| format!("{}", value.round())) }
    }

    pub fn format(mut self, format: impl Fn(f64) -> String + 'static) -> Self {
        self.format = Rc::new(format);
        self
    }
}

struct CountState {
    spring: Spring,
    last_frame: Instant,
}

impl RenderOnce for Count {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let reduced = Theme::get(cx).reduced_motion;
        let value = self.value;
        let state = window.use_keyed_state(self.id, cx, |_, _| CountState { spring: Spring::new(SETTLE, 0.0), last_frame: Instant::now() });
        let (shown, landed) = state.update(cx, |state, _| {
            state.spring.target = value;
            if reduced {
                state.spring.jump(value);
            }
            let dt = state.last_frame.elapsed().as_secs_f64().min(LONGEST_STEP);
            state.last_frame = Instant::now();
            let landed = state.spring.step(dt);
            (state.spring.position, landed)
        });
        if !landed {
            window.request_animation_frame();
        }
        div().min_w(px(0.)).child((self.format)(shown))
    }
}
