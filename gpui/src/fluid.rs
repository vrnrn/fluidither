//! The motion-led parts: HoverList, CheckGroup, ThinkingSteps, Questions, Kbd, InputGroup and
//! Reveal. GPUI has no accessibility API, so screen readers see nothing here.

use crate::accordion::Accordion;
use crate::button::{Button, Look};
use crate::choice::{Checkbox, Checked};
use crate::extras::{Step, Steps};
use crate::generated::{
    DITHER_HOVER_CELL, DITHER_HOVER_INK, GLIDE, QUICK, QUICK_OUT, RADIUS_CONTROL, RADIUS_PILL, SETTLE, WEIGHT_STRONG,
};
use crate::paint::checker;
use crate::spring::{Spring, LONGEST_STEP};
use crate::structure::Badge;
use crate::text_field::TextField;
use crate::theme::{fade, Theme};
use gpui::{
    div, prelude::*, px, AnyElement, App, ElementId, Entity, FocusHandle, FontWeight, IntoElement,
    KeyDownEvent, MouseButton, RenderOnce, SharedString, Window,
};
use std::rc::Rc;
use std::time::{Duration, Instant};

struct HoverState {
    over: Option<usize>,
    place: Spring,
    shown: Spring,
    last_frame: Instant,
}

/// Rows of one height with a single dithered highlight that glides between them.
#[derive(IntoElement)]
pub struct HoverList {
    id: ElementId,
    rows: Vec<AnyElement>,
    row_height: f32,
}

impl HoverList {
    pub fn new(id: impl Into<ElementId>, row_height: f32) -> Self {
        Self { id: id.into(), rows: Vec::new(), row_height }
    }

    pub fn row(mut self, row: impl IntoElement) -> Self {
        self.rows.push(row.into_any_element());
        self
    }
}

impl RenderOnce for HoverList {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let ink = fade(theme.colors().ink, DITHER_HOVER_INK);
        let reduced = theme.reduced_motion;
        let height = self.row_height;
        let state = window.use_keyed_state(self.id.clone(), cx, |_, _| HoverState {
            over: None,
            place: Spring::new(GLIDE, 0.0),
            shown: Spring::new(QUICK, 0.0),
            last_frame: Instant::now(),
        });
        let (y, shown, landed) = state.update(cx, |state, _| {
            if let Some(row) = state.over {
                let target = row as f64 * f64::from(height);
                if state.shown.position == 0.0 {
                    state.place.jump(target);
                }
                state.place.target = target;
                state.shown.config = QUICK;
                state.shown.target = 1.0;
            } else {
                state.shown.config = QUICK_OUT;
                state.shown.target = 0.0;
            }
            if reduced {
                state.place.jump(state.place.target);
                state.shown.jump(state.shown.target);
            }
            let dt = state.last_frame.elapsed().as_secs_f64().min(LONGEST_STEP);
            state.last_frame = Instant::now();
            let landed = state.place.step(dt) & state.shown.step(dt);
            (state.place.position as f32, state.shown.position as f32, landed)
        });
        if !landed {
            window.request_animation_frame();
        }
        let left = state.clone();
        div()
            .id(self.id)
            .relative()
            .w_full()
            .on_hover(move |over, _, cx| {
                if !*over {
                    left.update(cx, |state, cx| {
                        state.over = None;
                        cx.notify();
                    });
                }
            })
            .child(
                div()
                    .absolute()
                    .left_0()
                    .right_0()
                    .top(px(y))
                    .h(px(height))
                    .opacity(shown)
                    .child(checker(ink, DITHER_HOVER_CELL, RADIUS_PILL)),
            )
            .children(self.rows.into_iter().enumerate().map(|(index, row)| {
                let state = state.clone();
                div()
                    .id(index)
                    .relative()
                    .flex()
                    .items_center()
                    .h(px(height))
                    .on_hover(move |over, _, cx| {
                        if *over {
                            state.update(cx, |state, cx| {
                                state.over = Some(index);
                                cx.notify();
                            });
                        }
                    })
                    .child(row)
            }))
    }
}

/// Which corners of each checked row stay square: `(top, bottom)`. Rows checked side by side
/// join into one shape.
pub fn joins(checked: &[bool]) -> Vec<(bool, bool)> {
    (0..checked.len())
        .map(|index| {
            let on = checked[index];
            (on && index > 0 && checked[index - 1], on && checked.get(index + 1).copied().unwrap_or(false))
        })
        .collect()
}

type OnChecks = Rc<dyn Fn(&[bool], &mut Window, &mut App)>;

/// Checkbox rows whose backgrounds merge where neighbours are both checked.
#[derive(IntoElement)]
pub struct CheckGroup {
    name: SharedString,
    rows: Vec<SharedString>,
    initial: Vec<usize>,
    on_change: Option<OnChecks>,
}

impl CheckGroup {
    pub fn new<T: Into<SharedString>>(name: impl Into<SharedString>, rows: impl IntoIterator<Item = T>) -> Self {
        Self { name: name.into(), rows: rows.into_iter().map(Into::into).collect(), initial: Vec::new(), on_change: None }
    }

    /// The rows checked at the start.
    pub fn initial(mut self, rows: impl IntoIterator<Item = usize>) -> Self {
        self.initial = rows.into_iter().collect();
        self
    }

    pub fn on_change(mut self, handler: impl Fn(&[bool], &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for CheckGroup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = Theme::get(cx).colors();
        let count = self.rows.len();
        let initial = self.initial;
        let state = window.use_keyed_state(ElementId::Name(self.name.clone()), cx, |_, _| {
            (0..count).map(|row| initial.contains(&row)).collect::<Vec<bool>>()
        });
        let checked = state.read(cx).clone();
        let shapes = joins(&checked);
        let radius = px(RADIUS_CONTROL);
        div().flex().flex_col().w_full().children(self.rows.into_iter().enumerate().map(|(index, label)| {
            let (joined_top, joined_bottom) = shapes[index];
            let state = state.clone();
            let handler = self.on_change.clone();
            let id = ElementId::NamedInteger(self.name.clone(), index as u64);
            div()
                .flex()
                .items_center()
                .px(px(10.))
                .py(px(7.))
                .when(checked[index], |this| this.bg(colors.soft))
                .when(!joined_top, |this| this.rounded_t(radius))
                .when(!joined_bottom, |this| this.rounded_b(radius))
                .child(
                    Checkbox::new(id)
                        .label(label)
                        .checked(if checked[index] { Checked::On } else { Checked::Off })
                        .on_change(move |on, window, cx| {
                            let rows = state.update(cx, |rows, cx| {
                                rows[index] = on;
                                cx.notify();
                                rows.clone()
                            });
                            if let Some(handler) = &handler {
                                handler(&rows, window, cx);
                            }
                        }),
                )
        }))
    }
}

/// A run of steps folded under a line of status, with the sources it used.
#[derive(IntoElement)]
pub struct ThinkingSteps {
    id: ElementId,
    title: SharedString,
    steps: Vec<(SharedString, Step)>,
    sources: Vec<SharedString>,
}

impl ThinkingSteps {
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self { id: id.into(), title: title.into(), steps: Vec::new(), sources: Vec::new() }
    }

    pub fn step(mut self, label: impl Into<SharedString>, step: Step) -> Self {
        self.steps.push((label.into(), step));
        self
    }

    pub fn source(mut self, name: impl Into<SharedString>) -> Self {
        self.sources.push(name.into());
        self
    }
}

impl RenderOnce for ThinkingSteps {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let body = div()
            .flex()
            .flex_col()
            .gap(px(8.))
            .child(Steps::new(self.steps))
            .child(div().flex().flex_wrap().gap(px(4.)).children(self.sources.into_iter().map(Badge::new)));
        Accordion::new(self.id).section(self.title, body).initial(0)
    }
}

/// One entry per question: the rows chosen, in order, and empty if it was skipped.
type OnDone = Rc<dyn Fn(&[Vec<usize>], &mut Window, &mut App)>;

struct Question {
    title: SharedString,
    options: Vec<SharedString>,
    several: bool,
}

struct QuestionsState {
    focus: FocusHandle,
    at: usize,
    answers: Vec<Vec<usize>>,
    finished: bool,
}

impl QuestionsState {
    fn toggle(&mut self, row: usize) {
        let chosen = &mut self.answers[self.at];
        match chosen.iter().position(|each| *each == row) {
            Some(place) => {
                chosen.remove(place);
            }
            None => chosen.push(row),
        }
        chosen.sort_unstable();
    }

    /// Move on. Returns true when that was the last question.
    fn advance(&mut self) -> bool {
        if self.at + 1 < self.answers.len() {
            self.at += 1;
            return false;
        }
        self.finished = true;
        true
    }
}

fn advance(state: &Entity<QuestionsState>, handler: Option<&OnDone>, window: &mut Window, cx: &mut App) {
    let finished = state.update(cx, |state, cx| {
        cx.notify();
        state.advance()
    });
    if let Some(handler) = handler.filter(|_| finished) {
        let answers = state.read(cx).answers.clone();
        handler(&answers, window, cx);
    }
}

fn choose(state: &Entity<QuestionsState>, row: usize, several: bool, handler: Option<&OnDone>, window: &mut Window, cx: &mut App) {
    state.update(cx, |state, cx| {
        if several {
            state.toggle(row);
        } else {
            state.answers[state.at] = vec![row];
        }
        cx.notify();
    });
    if !several {
        advance(state, handler, window, cx);
    }
}

/// One question at a time. A click or a number key answers and moves on. With `several`, they
/// toggle rows and Next moves on.
#[derive(IntoElement)]
pub struct Questions {
    id: ElementId,
    name: SharedString,
    questions: Vec<Question>,
    closing: SharedString,
    on_done: Option<OnDone>,
}

impl Questions {
    pub fn new(name: impl Into<SharedString>) -> Self {
        let name = name.into();
        Self { id: ElementId::Name(name.clone()), name, questions: Vec::new(), closing: "Thanks.".into(), on_done: None }
    }

    pub fn question<T: Into<SharedString>>(mut self, title: impl Into<SharedString>, options: impl IntoIterator<Item = T>) -> Self {
        self.questions.push(Question { title: title.into(), options: options.into_iter().map(Into::into).collect(), several: false });
        self
    }

    /// A question that takes more than one answer.
    pub fn several<T: Into<SharedString>>(mut self, title: impl Into<SharedString>, options: impl IntoIterator<Item = T>) -> Self {
        self.questions.push(Question { title: title.into(), options: options.into_iter().map(Into::into).collect(), several: true });
        self
    }

    /// What it says once every question is done.
    pub fn closing(mut self, text: impl Into<SharedString>) -> Self {
        self.closing = text.into();
        self
    }

    pub fn on_done(mut self, handler: impl Fn(&[Vec<usize>], &mut Window, &mut App) + 'static) -> Self {
        self.on_done = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Questions {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = Theme::get(cx).colors();
        let count = self.questions.len();
        let state = window.use_keyed_state(self.id.clone(), cx, |_, cx| QuestionsState {
            focus: cx.focus_handle().tab_stop(true),
            at: 0,
            answers: vec![Vec::new(); count],
            finished: false,
        });
        let (focus, at, finished, chosen) = {
            let read = state.read(cx);
            (read.focus.clone(), read.at, read.finished, read.answers.get(read.at).cloned().unwrap_or_default())
        };
        let shell = div().id(self.id).flex().flex_col().gap(px(10.)).w_full().text_size(px(13.)).line_height(px(18.)).text_color(colors.ink);
        if finished {
            return shell.child(div().text_color(colors.ink_soft).child(self.closing));
        }
        let Question { title, options, several } = self.questions.into_iter().nth(at).expect("a question at this step");
        let option_count = options.len();
        let keyed = state.clone();
        let handler = self.on_done;
        let on_key = handler.clone();
        let on_skip = handler.clone();
        let on_next = handler.clone();
        let (skipped, back, next) = (state.clone(), state.clone(), state.clone());
        let mut rows = HoverList::new(ElementId::NamedInteger(self.name.clone(), at as u64), 30.);
        for (index, label) in options.into_iter().enumerate() {
            let state = state.clone();
            let handler = handler.clone();
            rows = rows.row(
                div()
                    .id(index)
                    .flex()
                    .items_center()
                    .justify_between()
                    .w_full()
                    .px(px(8.))
                    .cursor_pointer()
                    .when(chosen.contains(&index), |this| this.font_weight(FontWeight(WEIGHT_STRONG)))
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.global_mut::<Theme>().keyboard_input = false)
                    .on_click(move |_, window, cx| choose(&state, index, several, handler.as_ref(), window, cx))
                    .child(label)
                    .child(Kbd::new((index + 1).to_string())),
            );
        }
        shell
            .track_focus(&focus)
            .on_key_down(move |event: &KeyDownEvent, window, cx| {
                let digit = event.keystroke.key.parse::<usize>().unwrap_or(0);
                if (1..=option_count).contains(&digit) {
                    choose(&keyed, digit - 1, several, on_key.as_ref(), window, cx);
                    cx.stop_propagation();
                } else if several && event.keystroke.key == "enter" {
                    advance(&keyed, on_key.as_ref(), window, cx);
                    cx.stop_propagation();
                }
            })
            .child(div().text_color(colors.ink_soft).child(format!("Question {} of {count}", at + 1)))
            .child(div().font_weight(FontWeight(WEIGHT_STRONG)).child(title))
            .child(rows)
            .child(
                div()
                    .flex()
                    .justify_end()
                    .gap(px(6.))
                    .when(at > 0, |this| {
                        this.child(Button::new("fd-questions-back", "Back").look(Look::Ghost).on_click(move |_, _, cx| {
                            back.update(cx, |state, cx| {
                                state.at -= 1;
                                cx.notify();
                            });
                        }))
                    })
                    .child(Button::new("fd-questions-skip", "Skip").look(Look::Soft).on_click(move |_, window, cx| {
                        skipped.update(cx, |state, _| {
                            let at = state.at;
                            state.answers[at].clear();
                        });
                        advance(&skipped, on_skip.as_ref(), window, cx);
                    }))
                    .when(several, |this| {
                        this.child(Button::new("fd-questions-next", "Next").on_click(move |_, window, cx| advance(&next, on_next.as_ref(), window, cx)))
                    }),
            )
    }
}

/// One key or shortcut.
#[derive(IntoElement)]
pub struct Kbd {
    keys: SharedString,
}

impl Kbd {
    pub fn new(keys: impl Into<SharedString>) -> Self {
        Self { keys: keys.into() }
    }
}

impl RenderOnce for Kbd {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = Theme::get(cx).colors();
        div().flex_none().px(px(5.)).rounded(px(RADIUS_PILL)).bg(colors.soft).text_color(colors.ink_soft).text_size(px(11.)).line_height(px(16.)).child(self.keys)
    }
}

/// Labelled text fields stacked into one block. Create the fields once and keep the entities.
#[derive(IntoElement, Default)]
pub struct InputGroup {
    rows: Vec<(SharedString, Entity<TextField>)>,
}

impl InputGroup {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn field(mut self, label: impl Into<SharedString>, field: Entity<TextField>) -> Self {
        self.rows.push((label.into(), field));
        self
    }
}

impl RenderOnce for InputGroup {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = Theme::get(cx).colors();
        div().flex().flex_col().gap(px(6.)).w_full().children(self.rows.into_iter().map(|(label, field)| {
            div()
                .flex()
                .items_center()
                .text_size(px(13.))
                .text_color(colors.ink_soft)
                .child(div().flex_none().w(px(96.)).child(label))
                .child(div().flex_1().child(field))
        }))
    }
}

struct RevealState {
    started: Instant,
    shown: Spring,
    last_frame: Instant,
}

/// Rises into place once. Pass `delay` to stagger several.
#[derive(IntoElement)]
pub struct Reveal {
    id: ElementId,
    content: AnyElement,
    delay: Duration,
    rise: f32,
}

impl Reveal {
    pub fn new(id: impl Into<ElementId>, content: impl IntoElement) -> Self {
        Self { id: id.into(), content: content.into_any_element(), delay: Duration::ZERO, rise: 14. }
    }

    pub fn delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    /// A smaller move.
    pub fn subtle(mut self) -> Self {
        self.rise = 5.;
        self
    }
}

impl RenderOnce for Reveal {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let reduced = Theme::get(cx).reduced_motion;
        let state = window.use_keyed_state(self.id, cx, |_, _| RevealState {
            started: Instant::now(),
            shown: Spring::new(SETTLE, 0.0),
            last_frame: Instant::now(),
        });
        let delay = self.delay;
        let (shown, landed) = state.update(cx, |state, _| {
            if state.started.elapsed() >= delay {
                state.shown.target = 1.0;
            }
            if reduced {
                state.shown.jump(1.0);
            }
            let dt = state.last_frame.elapsed().as_secs_f64().min(LONGEST_STEP);
            state.last_frame = Instant::now();
            let landed = state.shown.step(dt) && state.shown.target == 1.0;
            (state.shown.position as f32, landed)
        });
        if !landed {
            window.request_animation_frame();
        }
        div().relative().top(px(self.rise * (1.0 - shown))).opacity(shown.clamp(0.0, 1.0)).child(self.content)
    }
}
