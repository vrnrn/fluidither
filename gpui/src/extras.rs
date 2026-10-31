//! The last parts: Steps, QuestionCard, Sidebar, Compare, Marquee, FileCard, NotFound,
//! Bloom, Composer and ColorPicker.

use crate::button::Button;
use crate::choice::RadioGroup;
use crate::generated::{RADIUS_CONTROL, RADIUS_PILL, SETTLE, WEIGHT_STRONG};
use crate::paint::{checker, focus_ring};
use crate::slider::Slider;
use crate::spring::{Spring, LONGEST_STEP};
use crate::structure::Tone;
use crate::text_field::{TextField, TextFieldEvent};
use crate::theme::{fade, Theme};
use gpui::{
    canvas, div, fill, point, prelude::*, px, relative, size, AnyElement, App, Bounds, BoxShadow,
    Context, DispatchPhase, ElementId, Entity, EventEmitter, FocusHandle, Focusable, FontWeight, Hsla,
    IntoElement, KeyDownEvent, MouseButton, MouseMoveEvent, MouseUpEvent, ParentElement, Pixels, Render,
    RenderOnce, Rgba, SharedString, Task, Timer, Window,
};
use std::rc::Rc;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Done,
    Active,
    Waiting,
}

/// A list of stages, for work that takes a while.
#[derive(IntoElement)]
pub struct Steps {
    steps: Vec<(SharedString, Step)>,
}

impl Steps {
    pub fn new<T: Into<SharedString>>(steps: impl IntoIterator<Item = (T, Step)>) -> Self {
        Self { steps: steps.into_iter().map(|(label, step)| (label.into(), step)).collect() }
    }
}

impl RenderOnce for Steps {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = Theme::get(cx).colors();
        div().flex().flex_col().gap(px(6.)).text_size(px(13.)).line_height(px(18.)).children(self.steps.into_iter().map(|(label, step)| {
            let mark = div().relative().flex_none().size(px(8.));
            let mark = match step {
                Step::Done => mark.bg(colors.ink),
                Step::Active => mark.child(checker(colors.ink, 2.0, 0.0)),
                Step::Waiting => mark.border_1().border_color(colors.edge),
            };
            div()
                .flex()
                .items_center()
                .gap(px(10.))
                .text_color(if step == Step::Waiting { colors.ink_soft } else { colors.ink })
                .when(step == Step::Active, |this| this.font_weight(FontWeight(WEIGHT_STRONG)))
                .child(mark)
                .child(label)
        }))
    }
}

type OnAnswer = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// Asks one thing, with choices and an answer button.
#[derive(IntoElement)]
pub struct QuestionCard {
    id: ElementId,
    question: SharedString,
    options: Vec<SharedString>,
    on_answer: Option<OnAnswer>,
}

impl QuestionCard {
    pub fn new<T: Into<SharedString>>(id: impl Into<ElementId>, question: impl Into<SharedString>, options: impl IntoIterator<Item = T>) -> Self {
        Self { id: id.into(), question: question.into(), options: options.into_iter().map(Into::into).collect(), on_answer: None }
    }

    pub fn on_answer(mut self, handler: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_answer = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for QuestionCard {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let colors = theme.colors();
        let shadow = theme.shadow(0);
        let chosen = window.use_keyed_state(self.id.clone(), cx, |_, _| 0usize);
        let picked = chosen.clone();
        let handler = self.on_answer;
        div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .p(px(14.))
            .rounded(px(RADIUS_CONTROL + 2.))
            .bg(colors.surfaces[1])
            .shadow(shadow)
            .text_color(colors.ink)
            .text_size(px(13.))
            .child(div().font_weight(FontWeight(WEIGHT_STRONG)).child(self.question))
            .child(RadioGroup::new(("question-options", 0usize), self.options).on_change(move |index, _, cx| {
                picked.update(cx, |picked, _| *picked = index);
            }))
            .child(div().flex().child(Button::new(("question-answer", 0usize), "Answer").on_click(move |_, window, cx| {
                let index = *chosen.read(cx);
                if let Some(handler) = &handler {
                    handler(index, window, cx);
                }
            })))
    }
}

struct SidebarState {
    focus: FocusHandle,
    collapsed: bool,
    width: Spring,
    last_frame: Instant,
}

/// A panel that collapses to a thin strip. Add its content with `.child`.
#[derive(IntoElement)]
pub struct Sidebar {
    id: ElementId,
    width: f32,
    children: Vec<AnyElement>,
}

impl Sidebar {
    const THIN: f32 = 34.0;

    pub fn new(id: impl Into<ElementId>) -> Self {
        Self { id: id.into(), width: 200.0, children: Vec::new() }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
}

impl ParentElement for Sidebar {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Sidebar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let colors = theme.colors();
        let (reduced, keyboard) = (theme.reduced_motion, theme.keyboard_input);
        let full = self.width;
        let state = window.use_keyed_state(self.id.clone(), cx, |_, cx| SidebarState {
            focus: cx.focus_handle().tab_stop(true),
            collapsed: false,
            width: Spring::new(SETTLE, f64::from(full)),
            last_frame: Instant::now(),
        });
        let (focus, collapsed, width, landed) = state.update(cx, |state, _| {
            state.width.target = f64::from(if state.collapsed { Self::THIN } else { full });
            if reduced {
                state.width.jump(state.width.target);
            }
            let dt = state.last_frame.elapsed().as_secs_f64().min(LONGEST_STEP);
            state.last_frame = Instant::now();
            let landed = state.width.step(dt);
            (state.focus.clone(), state.collapsed, state.width.position as f32, landed)
        });
        if !landed {
            window.request_animation_frame();
        }
        let ring = focus.is_focused(window) && keyboard;
        let clicked = state.clone();
        let keyed = state;
        let flip = move |state: &Entity<SidebarState>, cx: &mut App| {
            state.update(cx, |state, cx| {
                state.collapsed = !state.collapsed;
                cx.notify();
            });
        };
        div()
            .flex()
            .flex_col()
            .flex_none()
            .w(px(width))
            .overflow_hidden()
            .rounded(px(RADIUS_CONTROL + 2.))
            .bg(colors.surfaces[1])
            .border_1()
            .border_color(colors.line)
            .child(
                div().flex().justify_end().p(px(3.)).child(
                    div()
                        .id(self.id)
                        .debug_selector(|| "sidebar-toggle".into())
                        .track_focus(&focus)
                        .relative()
                        .flex()
                        .items_center()
                        .justify_center()
                        .size(px(26.))
                        .rounded(px(RADIUS_PILL))
                        .bg(colors.soft)
                        .text_color(colors.ink)
                        .text_size(px(12.))
                        .cursor_pointer()
                        .on_click(move |_, _, cx| {
                            cx.global_mut::<Theme>().keyboard_input = false;
                            flip(&clicked, cx);
                        })
                        .on_key_down(move |event: &KeyDownEvent, _, cx| {
                            if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                                flip(&keyed, cx);
                                cx.stop_propagation();
                            }
                        })
                        .child(if collapsed { ">" } else { "<" })
                        .when(ring, |this| this.child(focus_ring(colors.focus, RADIUS_PILL, 1.))),
                ),
            )
            .when(!collapsed, |this| this.child(div().w(px(full - 2.)).children(self.children)))
    }
}

struct CompareState {
    focus: FocusHandle,
    split: f32,
    dragging: bool,
    bounds: Bounds<Pixels>,
}

/// Two things side by side with a line to drag between them. Arrow keys move it too.
#[derive(IntoElement)]
pub struct Compare {
    id: ElementId,
    first: AnyElement,
    second: AnyElement,
    height: f32,
}

impl Compare {
    pub fn new(id: impl Into<ElementId>, first: impl IntoElement, second: impl IntoElement) -> Self {
        Self { id: id.into(), first: first.into_any_element(), second: second.into_any_element(), height: 160.0 }
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }
}

fn split_at(state: &Entity<CompareState>, x: Pixels, cx: &mut App) {
    state.update(cx, |state, cx| {
        let share = f32::from(x - state.bounds.left()) / f32::from(state.bounds.size.width).max(1.0);
        state.split = share.clamp(0.0, 1.0);
        cx.notify();
    });
}

impl RenderOnce for Compare {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let colors = theme.colors();
        let keyboard = theme.keyboard_input;
        let state = window.use_keyed_state(self.id.clone(), cx, |_, cx| CompareState {
            focus: cx.focus_handle().tab_stop(true),
            split: 0.5,
            dragging: false,
            bounds: Bounds::default(),
        });
        let (focus, split, dragging, width) = {
            let state = state.read(cx);
            (state.focus.clone(), state.split, state.dragging, state.bounds.size.width)
        };
        let ring = focus.is_focused(window) && keyboard;
        let measured = state.clone();
        let tracked = state.clone();
        let pressed = state.clone();
        let keyed = state;
        div()
            .id(self.id)
            .track_focus(&focus)
            .relative()
            .w_full()
            .h(px(self.height))
            .overflow_hidden()
            .cursor_pointer()
            .on_mouse_down(MouseButton::Left, move |event, _, cx| {
                cx.global_mut::<Theme>().keyboard_input = false;
                pressed.update(cx, |state, _| state.dragging = true);
                split_at(&pressed, event.position.x, cx);
            })
            .on_key_down(move |event: &KeyDownEvent, _, cx| {
                let step = match event.keystroke.key.as_str() {
                    "left" => -0.01,
                    "right" => 0.01,
                    _ => return,
                };
                cx.global_mut::<Theme>().keyboard_input = true;
                keyed.update(cx, |state, cx| {
                    state.split = (state.split + step).clamp(0.0, 1.0);
                    cx.notify();
                });
                cx.stop_propagation();
            })
            .child(div().absolute().top_0().left_0().size_full().child(self.first))
            .child(
                div()
                    .absolute()
                    .top_0()
                    .bottom_0()
                    .right_0()
                    .left(relative(split))
                    .overflow_hidden()
                    .child(div().absolute().top_0().bottom_0().right_0().w(width).child(self.second)),
            )
            .child(div().absolute().top_0().bottom_0().left(relative(split)).w(px(2.)).bg(colors.focus))
            .when(ring, |this| this.border_1().border_color(colors.focus))
            .child(
                canvas(
                    move |bounds, _, cx| {
                        measured.update(cx, |state, cx| {
                            if state.bounds != bounds {
                                state.bounds = bounds;
                                cx.notify();
                            }
                        })
                    },
                    move |_, _, window, _| {
                        if !dragging {
                            return;
                        }
                        // Window-wide, so a drag survives leaving the box.
                        let moved = tracked.clone();
                        window.on_mouse_event(move |event: &MouseMoveEvent, phase, _, cx| {
                            if phase == DispatchPhase::Bubble && event.pressed_button == Some(MouseButton::Left) {
                                split_at(&moved, event.position.x, cx);
                            }
                        });
                        let released = tracked.clone();
                        window.on_mouse_event(move |_: &MouseUpEvent, phase, _, cx| {
                            if phase == DispatchPhase::Bubble {
                                released.update(cx, |state, cx| {
                                    state.dragging = false;
                                    cx.notify();
                                });
                            }
                        });
                    },
                )
                .absolute()
                .top_0()
                .left_0()
                .size_full(),
            )
    }
}

const MARQUEE_STEP: Duration = Duration::from_millis(33);
const MARQUEE_SPEED: f32 = 30.0;

struct MarqueeState {
    started: Instant,
    held: f32,
    paused: bool,
    width: f32,
    timer: Option<Task<()>>,
}

/// Text scrolling sideways for ever. It stops while hovered and under reduced motion.
#[derive(IntoElement)]
pub struct Marquee {
    id: ElementId,
    items: Vec<SharedString>,
}

impl Marquee {
    pub fn new<T: Into<SharedString>>(id: impl Into<ElementId>, items: impl IntoIterator<Item = T>) -> Self {
        Self { id: id.into(), items: items.into_iter().map(Into::into).collect() }
    }
}

impl RenderOnce for Marquee {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let colors = theme.colors();
        let reduced = theme.reduced_motion;
        let state = window.use_keyed_state(self.id.clone(), cx, |_, _| MarqueeState {
            started: Instant::now(),
            held: 0.0,
            paused: false,
            width: 0.0,
            timer: None,
        });
        let shift = state.update(cx, |state, cx| {
            if !reduced && state.timer.is_none() {
                state.timer = Some(cx.spawn(async move |this, cx| loop {
                    Timer::after(MARQUEE_STEP).await;
                    let moved = this.update(cx, |state, cx| {
                        if !state.paused {
                            cx.notify();
                        }
                    });
                    if moved.is_err() {
                        break;
                    }
                }));
            }
            let moved = if state.paused || reduced { state.held } else { state.held + state.started.elapsed().as_secs_f32() * MARQUEE_SPEED };
            if state.width > 0.0 { moved % state.width } else { 0.0 }
        });
        let track = |items: &[SharedString]| div().flex().flex_none().gap(px(32.)).pr(px(32.)).children(items.iter().cloned().map(|item| div().flex_none().child(item)));
        let hovered = state.clone();
        let measured = state;
        div()
            .id(self.id)
            .relative()
            .w_full()
            .h(px(18.))
            .overflow_hidden()
            .text_size(px(13.))
            .line_height(px(18.))
            .font_weight(FontWeight(WEIGHT_STRONG))
            .text_color(colors.ink_soft)
            .on_hover(move |over, _, cx| {
                hovered.update(cx, |state, _| {
                    if *over {
                        state.held += state.started.elapsed().as_secs_f32() * MARQUEE_SPEED;
                    } else {
                        state.started = Instant::now();
                    }
                    state.paused = *over;
                });
            })
            .child(
                div()
                    .absolute()
                    .top_0()
                    .left(px(-shift))
                    .flex()
                    .child(track(&self.items))
                    .child(track(&self.items))
                    .on_children_prepainted(move |bounds, _, cx| {
                        let Some(first) = bounds.first() else { return };
                        let width = f32::from(first.size.width);
                        measured.update(cx, |state, _| state.width = width);
                    }),
            )
    }
}

/// A file with its type and size. It shows no preview: rendering a PDF needs a renderer
/// that this library doesn't carry.
#[derive(IntoElement)]
pub struct FileCard {
    name: SharedString,
    size: SharedString,
}

impl FileCard {
    pub fn new(name: impl Into<SharedString>, size: impl Into<SharedString>) -> Self {
        Self { name: name.into(), size: size.into() }
    }
}

impl RenderOnce for FileCard {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let colors = theme.colors();
        let kind = self.name.rsplit('.').next().unwrap_or("").to_uppercase();
        div()
            .flex()
            .flex_none()
            .items_center()
            .gap(px(10.))
            .p(px(8.))
            .pr(px(12.))
            .rounded(px(RADIUS_CONTROL))
            .bg(colors.surfaces[1])
            .shadow(theme.shadow(0))
            .text_color(colors.ink)
            .text_size(px(13.))
            .line_height(px(18.))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(px(34.))
                    .rounded(px(RADIUS_PILL))
                    .bg(colors.soft)
                    .text_size(px(10.))
                    .font_weight(FontWeight(WEIGHT_STRONG))
                    .child(kind),
            )
            .child(div().flex().flex_col().child(self.name).child(div().text_size(px(12.)).text_color(colors.ink_soft).child(self.size)))
    }
}

/// A page for something that isn't there.
#[derive(IntoElement)]
pub struct NotFound {
    code: SharedString,
    text: SharedString,
    action: Option<AnyElement>,
}

impl NotFound {
    pub fn new(code: impl Into<SharedString>, text: impl Into<SharedString>) -> Self {
        Self { code: code.into(), text: text.into(), action: None }
    }

    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.action = Some(action.into_any_element());
        self
    }
}

impl RenderOnce for NotFound {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = Theme::get(cx).colors();
        div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(6.))
            .py(px(40.))
            .text_size(px(13.))
            .text_color(colors.ink_soft)
            .child(
                crate::text_effects::Dissolve::new("fd-not-found", div().text_size(px(64.)).line_height(px(64.)).font_weight(FontWeight(700.)).text_color(colors.ink).child(self.code))
                    .hold(0.1, 0.7),
            )
            .child(self.text)
            .children(self.action)
    }
}

/// A soft glow around its content, in a tone.
#[derive(IntoElement)]
pub struct Bloom {
    tone: Tone,
    content: AnyElement,
}

impl Bloom {
    pub fn new(tone: Tone, content: impl IntoElement) -> Self {
        Self { tone, content: content.into_any_element() }
    }
}

impl RenderOnce for Bloom {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = Theme::get(cx).colors();
        let color = match self.tone {
            Tone::Neutral => colors.ink_soft,
            Tone::Red => colors.red,
            Tone::Orange => colors.orange,
            Tone::Green => colors.green,
            Tone::Blue => colors.blue,
            Tone::Violet => colors.violet,
            Tone::Pink => colors.pink,
        };
        let layer = |alpha: f32, blur: f32| BoxShadow {
            color: Hsla::from(fade(color, alpha)),
            offset: point(px(0.), px(0.)),
            blur_radius: px(blur),
            spread_radius: px(0.),
        };
        div().flex().flex_none().rounded(px(RADIUS_PILL)).shadow(vec![layer(0.6, 3.), layer(0.45, 14.)]).child(self.content)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ComposerEvent {
    Sent(String),
}

/// A message box with a send button. Enter or the button sends and clears it. With a
/// suggestion, Tab takes it into an empty box.
pub struct Composer {
    field: Entity<TextField>,
    suggestion: Option<SharedString>,
}

impl EventEmitter<ComposerEvent> for Composer {}

impl Focusable for Composer {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.field.read(cx).focus_handle(cx)
    }
}

impl Composer {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let field = cx.new(|cx| TextField::new(cx).placeholder("Write a message"));
        cx.subscribe(&field, |this, _, event, cx| {
            if matches!(event, TextFieldEvent::Submitted(_)) {
                this.send(cx);
            }
        })
        .detach();
        Self { field, suggestion: None }
    }

    /// Offer a message, shown as the placeholder.
    pub fn suggestion(mut self, text: impl Into<SharedString>, cx: &mut Context<Self>) -> Self {
        let text = text.into();
        self.field.update(cx, |field, _| field.set_placeholder(text.clone()));
        self.suggestion = Some(text);
        self
    }

    fn take_suggestion(&mut self, cx: &mut Context<Self>) {
        match self.suggestion.clone().filter(|_| self.field.read(cx).text().is_empty()) {
            Some(text) => self.field.update(cx, |field, cx| field.set_text(text.to_string(), cx)),
            None => cx.propagate(),
        }
    }

    fn send(&mut self, cx: &mut Context<Self>) {
        let text = self.field.read(cx).text().trim().to_string();
        if !text.is_empty() {
            self.field.update(cx, |field, cx| field.set_text("", cx));
            cx.emit(ComposerEvent::Sent(text));
        }
    }
}

impl Render for Composer {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity();
        let take = cx.listener(|this, _: &crate::FocusNext, _, cx| this.take_suggestion(cx));
        div().flex().gap(px(6.)).w_full().on_action(take).child(div().flex_1().child(self.field.clone())).child(Button::new("fd-composer-send", "Send").on_click(move |_, _, cx| {
            view.update(cx, |this, cx| this.send(cx));
        }))
    }
}

/// Hue 0 to 360, saturation and value 0 to 1.
pub fn hsv_to_rgb(hue: f32, saturation: f32, value: f32) -> Rgba {
    let turn = hue.rem_euclid(360.0) / 60.0;
    let chroma = value * saturation;
    let rising = chroma * (1.0 - (turn % 2.0 - 1.0).abs());
    let (r, g, b) = match turn as u32 {
        0 => (chroma, rising, 0.0),
        1 => (rising, chroma, 0.0),
        2 => (0.0, chroma, rising),
        3 => (0.0, rising, chroma),
        4 => (rising, 0.0, chroma),
        _ => (chroma, 0.0, rising),
    };
    let base = value - chroma;
    Rgba { r: r + base, g: g + base, b: b + base, a: 1.0 }
}

const PICKER_COLUMNS: usize = 32;
const PICKER_ROWS: usize = 18;
const PICKER_CELL: f32 = 8.0;

type OnColor = Rc<dyn Fn(Rgba, &mut Window, &mut App)>;

struct PickerState {
    focus: FocusHandle,
    origin: gpui::Point<Pixels>,
    hue: f32,
    saturation: f32,
    value: f32,
}

/// A saturation and value field in cells, above a hue slider. Arrow keys move the mark.
#[derive(IntoElement)]
pub struct ColorPicker {
    id: ElementId,
    on_change: Option<OnColor>,
}

impl ColorPicker {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self { id: id.into(), on_change: None }
    }

    pub fn on_change(mut self, handler: impl Fn(Rgba, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

fn report(state: &Entity<PickerState>, handler: Option<&OnColor>, window: &mut Window, cx: &mut App) {
    let picked = state.read(cx);
    let color = hsv_to_rgb(picked.hue, picked.saturation, picked.value);
    if let Some(handler) = handler {
        handler(color, window, cx);
    }
}

impl RenderOnce for ColorPicker {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let colors = theme.colors();
        let keyboard = theme.keyboard_input;
        let state = window.use_keyed_state(self.id.clone(), cx, |_, cx| PickerState {
            focus: cx.focus_handle().tab_stop(true),
            origin: gpui::Point::default(),
            hue: 210.0,
            saturation: 0.7,
            value: 0.8,
        });
        let (focus, hue, saturation, value) = {
            let state = state.read(cx);
            (state.focus.clone(), state.hue, state.saturation, state.value)
        };
        let ring = focus.is_focused(window) && keyboard;
        let (wide, tall) = (PICKER_COLUMNS as f32 * PICKER_CELL, PICKER_ROWS as f32 * PICKER_CELL);
        let on_change = self.on_change;
        let keyed = state.clone();
        let key_handler = on_change.clone();
        let clicked = state.clone();
        let click_handler = on_change.clone();
        let placed = state.clone();
        let hue_state = state;
        let hue_handler = on_change;
        let field_id = ElementId::from(("fd-color-field", 0usize));
        div()
            .flex()
            .flex_col()
            .gap(px(8.))
            .w(px(wide))
            .child(
                div()
                    .id(field_id)
                    .debug_selector(|| "color-field".into())
                    .track_focus(&focus)
                    .relative()
                    .w(px(wide))
                    .h(px(tall))
                    .cursor_pointer()
                    .on_mouse_down(MouseButton::Left, move |event, window, cx| {
                        cx.global_mut::<Theme>().keyboard_input = false;
                        clicked.update(cx, |state, cx| {
                            let at = event.position - state.origin;
                            state.saturation = (f32::from(at.x) / wide).clamp(0.0, 1.0);
                            state.value = 1.0 - (f32::from(at.y) / tall).clamp(0.0, 1.0);
                            cx.notify();
                        });
                        report(&clicked, click_handler.as_ref(), window, cx);
                    })
                    .on_key_down(move |event: &KeyDownEvent, window, cx| {
                        let (across, down) = match event.keystroke.key.as_str() {
                            "left" => (-1.0, 0.0),
                            "right" => (1.0, 0.0),
                            "up" => (0.0, -1.0),
                            "down" => (0.0, 1.0),
                            _ => return,
                        };
                        cx.global_mut::<Theme>().keyboard_input = true;
                        keyed.update(cx, |state, cx| {
                            state.saturation = (state.saturation + across / PICKER_COLUMNS as f32).clamp(0.0, 1.0);
                            state.value = (state.value - down / PICKER_ROWS as f32).clamp(0.0, 1.0);
                            cx.notify();
                        });
                        report(&keyed, key_handler.as_ref(), window, cx);
                        cx.stop_propagation();
                    })
                    .child(
                        canvas(
                            move |bounds: Bounds<Pixels>, _, cx| placed.update(cx, |state, _| state.origin = bounds.origin),
                            move |bounds, _, window, _| {
                                for row in 0..PICKER_ROWS {
                                    for column in 0..PICKER_COLUMNS {
                                        let s = column as f32 / (PICKER_COLUMNS - 1) as f32;
                                        let v = 1.0 - row as f32 / (PICKER_ROWS - 1) as f32;
                                        let origin = bounds.origin + point(px(column as f32 * PICKER_CELL), px(row as f32 * PICKER_CELL));
                                        window.paint_quad(fill(Bounds::new(origin, size(px(PICKER_CELL), px(PICKER_CELL))), hsv_to_rgb(hue, s, v)));
                                    }
                                }
                            },
                        )
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full(),
                    )
                    .child(
                        div()
                            .absolute()
                            .left(px(saturation * (wide - PICKER_CELL)))
                            .top(px((1.0 - value) * (tall - PICKER_CELL)))
                            .size(px(PICKER_CELL))
                            .border_1()
                            .border_color(colors.paper),
                    )
                    .when(ring, |this| this.child(focus_ring(colors.focus, 0.0, 2.))),
            )
            .child(
                Slider::new("fd-color-hue").range(0.0, 360.0).value(f64::from(hue)).on_change(move |hue, window, cx| {
                    hue_state.update(cx, |state, cx| {
                        state.hue = hue as f32;
                        cx.notify();
                    });
                    report(&hue_state, hue_handler.as_ref(), window, cx);
                }),
            )
    }
}
