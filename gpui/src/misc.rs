//! Smaller pieces: Swatches, Message, Thinking, Dots, CopyField, Avatar, EmptyState
//! and Uptime. The drawer is `Dialog::drawer`.

use crate::button::{Button, Look};
use crate::feedback::Loader;
use crate::generated::{DITHER_HOVER_CELL, RADIUS_CONTROL, RADIUS_PILL, WEIGHT_STRONG};
use crate::paint::{checker, focus_ring};
use crate::theme::{fade, Density, Theme};
use gpui::{
    div, prelude::*, px, App, ClipboardItem, ElementId, Entity, FocusHandle, FontWeight, IntoElement,
    KeyDownEvent, MouseButton, RenderOnce, Rgba, SharedString, Task, Timer, Window,
};
use std::rc::Rc;
use std::time::Duration;

type OnPick = Rc<dyn Fn(usize, &mut Window, &mut App)>;

struct PickState {
    focus: FocusHandle,
    selected: usize,
}

fn pick(state: &Entity<PickState>, index: usize, handler: Option<&OnPick>, window: &mut Window, cx: &mut App) {
    state.update(cx, |state, cx| {
        state.selected = index;
        cx.notify();
    });
    if let Some(handler) = handler {
        handler(index, window, cx);
    }
}

/// One color from a set. Arrow keys move the selection.
#[derive(IntoElement)]
pub struct Swatches {
    id: ElementId,
    colors: Vec<Rgba>,
    initial: usize,
    on_change: Option<OnPick>,
}

impl Swatches {
    pub fn new(id: impl Into<ElementId>, colors: impl IntoIterator<Item = Rgba>) -> Self {
        Self { id: id.into(), colors: colors.into_iter().collect(), initial: 0, on_change: None }
    }

    pub fn initial(mut self, index: usize) -> Self {
        self.initial = index;
        self
    }

    pub fn on_change(mut self, handler: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Swatches {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let (colors, keyboard) = (theme.colors(), theme.keyboard_input);
        let initial = self.initial;
        let state = window.use_keyed_state(self.id.clone(), cx, |_, cx| PickState { focus: cx.focus_handle().tab_stop(true), selected: initial });
        let (focus, selected) = {
            let state = state.read(cx);
            (state.focus.clone(), state.selected)
        };
        let ring = focus.is_focused(window) && keyboard;
        let count = self.colors.len();
        let on_change = self.on_change;
        let swatches: Vec<_> = self
            .colors
            .iter()
            .enumerate()
            .map(|(index, color)| {
                let clicked = state.clone();
                let handler = on_change.clone();
                div()
                    .id(("swatch", index))
                    .debug_selector(|| format!("swatch-{index}"))
                    .relative()
                    .size(px(22.))
                    .rounded(px(RADIUS_PILL))
                    .bg(*color)
                    .border_1()
                    .border_color(colors.line)
                    .cursor_pointer()
                    .on_click(move |_, window, cx| {
                        cx.global_mut::<Theme>().keyboard_input = false;
                        pick(&clicked, index, handler.as_ref(), window, cx);
                    })
                    .when(index == selected, |this| {
                        this.child(focus_ring(if ring { colors.focus } else { colors.ink }, RADIUS_PILL, 1.))
                    })
            })
            .collect();
        let keyed = state;
        div()
            .id(self.id)
            .track_focus(&focus)
            .flex()
            .flex_none()
            .gap(px(8.))
            .on_mouse_down(MouseButton::Left, |_, _, cx| {
                cx.global_mut::<Theme>().keyboard_input = false;
            })
            .on_key_down(move |event: &KeyDownEvent, window, cx| {
                let step = match event.keystroke.key.as_str() {
                    "right" | "down" => 1,
                    "left" | "up" => count - 1,
                    _ => return,
                };
                let next = (keyed.read(cx).selected + step) % count;
                cx.global_mut::<Theme>().keyboard_input = true;
                pick(&keyed, next, on_change.as_ref(), window, cx);
                cx.stop_propagation();
            })
            .children(swatches)
    }
}

/// A chat bubble. `mine` sits on the right in solid ink.
#[derive(IntoElement)]
pub struct Message {
    text: SharedString,
    mine: bool,
}

impl Message {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self { text: text.into(), mine: false }
    }

    pub fn mine(mut self) -> Self {
        self.mine = true;
        self
    }
}

impl RenderOnce for Message {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = Theme::get(cx).colors();
        div().flex().w_full().when(self.mine, |this| this.justify_end()).child(
            div()
                .max_w(px(420.))
                .px(px(12.))
                .py(px(8.))
                .rounded(px(RADIUS_CONTROL + 2.))
                .bg(if self.mine { colors.ink } else { colors.soft })
                .text_color(if self.mine { colors.on_solid } else { colors.ink })
                .text_size(px(13.))
                .line_height(px(18.))
                .child(self.text),
        )
    }
}

/// A label beside counting cells, for something that is working.
#[derive(IntoElement)]
pub struct Thinking {
    id: ElementId,
    label: SharedString,
}

impl Thinking {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self { id: id.into(), label: label.into() }
    }
}

impl RenderOnce for Thinking {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = Theme::get(cx).colors();
        div().flex().items_center().gap(px(10.)).text_size(px(13.)).text_color(colors.ink_soft).child(Loader::new(self.id)).child(self.label)
    }
}

/// Dots for a carousel.
#[derive(IntoElement)]
pub struct Dots {
    id: ElementId,
    count: usize,
    current: usize,
    on_change: Option<OnPick>,
}

impl Dots {
    pub fn new(id: impl Into<ElementId>, count: usize, current: usize) -> Self {
        Self { id: id.into(), count, current, on_change: None }
    }

    pub fn on_change(mut self, handler: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Dots {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = Theme::get(cx).colors();
        let on_change = self.on_change;
        let current = self.current;
        div().id(self.id).flex().gap(px(6.)).children((0..self.count).map(|index| {
            let handler = on_change.clone();
            div()
                .id(("dot", index))
                .debug_selector(|| format!("dot-{index}"))
                .size(px(8.))
                .bg(if index == current { colors.ink } else { colors.edge })
                .cursor_pointer()
                .on_click(move |_, window, cx| {
                    if let Some(handler) = &handler {
                        handler(index, window, cx);
                    }
                })
        }))
    }
}

struct CopyState {
    copied: bool,
    reset: Option<Task<()>>,
}

/// Read-only text with a copy button that reports success.
#[derive(IntoElement)]
pub struct CopyField {
    id: ElementId,
    text: SharedString,
}

impl CopyField {
    pub fn new(id: impl Into<ElementId>, text: impl Into<SharedString>) -> Self {
        Self { id: id.into(), text: text.into() }
    }
}

impl RenderOnce for CopyField {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = Theme::get(cx).colors();
        let tokens = Density::default().tokens();
        let state = window.use_keyed_state(self.id.clone(), cx, |_, _| CopyState { copied: false, reset: None });
        let copied = state.read(cx).copied;
        let text = self.text.clone();
        div()
            .flex()
            .gap(px(tokens.gap))
            .w_full()
            .child(
                div()
                    .flex()
                    .flex_1()
                    .items_center()
                    .h(px(tokens.control))
                    .px(px(tokens.tab_pad))
                    .rounded(px(RADIUS_CONTROL))
                    .border_1()
                    .border_color(colors.edge)
                    .bg(colors.surfaces[1])
                    .text_size(px(tokens.font_size))
                    .text_color(colors.ink)
                    .child(self.text),
            )
            .child(Button::new(self.id, if copied { "Copied" } else { "Copy" }).look(Look::Soft).on_click(move |_, _, cx| {
                cx.write_to_clipboard(ClipboardItem::new_string(text.to_string()));
                state.update(cx, |state, cx| {
                    state.copied = true;
                    state.reset = Some(cx.spawn(async move |this, cx| {
                        Timer::after(Duration::from_millis(1500)).await;
                        this.update(cx, |state, cx| {
                            state.copied = false;
                            cx.notify();
                        })
                        .ok();
                    }));
                    cx.notify();
                });
            }))
    }
}

/// Initials on a dithered square.
#[derive(IntoElement)]
pub struct Avatar {
    initials: SharedString,
}

impl Avatar {
    pub fn new(initials: impl Into<SharedString>) -> Self {
        Self { initials: initials.into() }
    }
}

impl RenderOnce for Avatar {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = Theme::get(cx).colors();
        div()
            .relative()
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .size(px(34.))
            .child(checker(fade(colors.ink_soft, 0.4), DITHER_HOVER_CELL, RADIUS_CONTROL))
            .child(
                div()
                    .relative()
                    .px(px(3.))
                    .rounded(px(3.))
                    .bg(colors.paper)
                    .text_color(colors.ink)
                    .text_size(px(13.))
                    .font_weight(FontWeight(WEIGHT_STRONG))
                    .child(self.initials),
            )
    }
}

/// What to show when there is nothing to show.
#[derive(IntoElement)]
pub struct EmptyState {
    title: SharedString,
    text: SharedString,
}

impl EmptyState {
    pub fn new(title: impl Into<SharedString>, text: impl Into<SharedString>) -> Self {
        Self { title: title.into(), text: text.into() }
    }
}

impl RenderOnce for EmptyState {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = Theme::get(cx).colors();
        div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(4.))
            .py(px(32.))
            .text_size(px(13.))
            .text_color(colors.ink_soft)
            .child(div().text_size(px(14.)).font_weight(FontWeight(WEIGHT_STRONG)).text_color(colors.ink).child(self.title))
            .child(self.text)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Up,
    Partial,
    Down,
}

/// One block per period.
#[derive(IntoElement)]
pub struct Uptime {
    periods: Vec<Status>,
}

impl Uptime {
    pub fn new(periods: impl IntoIterator<Item = Status>) -> Self {
        Self { periods: periods.into_iter().collect() }
    }
}

impl RenderOnce for Uptime {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = Theme::get(cx).colors();
        div().flex().gap(px(2.)).h(px(24.)).w_full().children(self.periods.into_iter().map(|status| {
            let block = div().relative().flex_1().h_full();
            match status {
                Status::Up => block.bg(colors.green),
                Status::Down => block.bg(colors.red),
                Status::Partial => block.child(checker(fade(colors.orange, 0.7), 2.0, 0.0)),
            }
        }))
    }
}
