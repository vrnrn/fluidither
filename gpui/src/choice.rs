//! On-or-off controls: Checkbox, Switch and RadioGroup. Each keeps its own state
//! unless you pass the value in. GPUI has no accessibility API, so screen readers
//! see nothing here.

use crate::generated::{
    DITHER_HOVER_CELL, DITHER_HOVER_INK, GLYPH_DASH, GLYPH_TICK, QUICK, RADIUS_PILL, SETTLE,
};
use crate::paint::{checker, focus_ring, glyph};
use crate::spring::{Spring, LONGEST_STEP};
use crate::theme::{cap_offset, fade, mix, Density, Theme};
use gpui::{
    div, prelude::*, px, App, ElementId, Entity, FocusHandle, IntoElement, KeyDownEvent,
    MouseButton, Pixels, RenderOnce, Rgba, SharedString, Stateful, Window,
};
use std::rc::Rc;
use std::time::Instant;

type OnToggle = Rc<dyn Fn(bool, &mut Window, &mut App)>;

struct ToggleState {
    focus: FocusHandle,
    on: bool,
    pointer_over: bool,
    hover: Spring,
    flip: Spring,
    last_frame: Instant,
}

impl ToggleState {
    fn advance(&mut self, reduced: bool) -> bool {
        self.hover.target = f64::from(u8::from(self.pointer_over));
        self.flip.target = f64::from(u8::from(self.on));
        if reduced {
            self.hover.jump(self.hover.target);
            self.flip.jump(self.flip.target);
        }
        let dt = self.last_frame.elapsed().as_secs_f64().min(LONGEST_STEP);
        self.last_frame = Instant::now();
        self.hover.step(dt) & self.flip.step(dt)
    }
}

struct Toggle {
    state: Entity<ToggleState>,
    on: bool,
    hover: f32,
    flip: f32,
    ring: bool,
    nudge: Pixels,
}

fn use_toggle(id: &ElementId, given: Option<bool>, initial: bool, density: Density, window: &mut Window, cx: &mut App) -> Toggle {
    let theme = Theme::get(cx);
    let (reduced, keyboard) = (theme.reduced_motion, theme.keyboard_input);
    let state = window.use_keyed_state(id.clone(), cx, |_, cx| {
        let on = given.unwrap_or(initial);
        ToggleState {
            focus: cx.focus_handle().tab_stop(true),
            on,
            pointer_over: false,
            hover: Spring::new(QUICK, 0.0),
            flip: Spring::new(SETTLE, f64::from(u8::from(on))),
            last_frame: Instant::now(),
        }
    });
    let (on, hover, flip, focus, landed) = state.update(cx, |state, _| {
        if let Some(on) = given {
            state.on = on;
        }
        let landed = state.advance(reduced);
        (state.on, state.hover.position as f32, state.flip.position as f32, state.focus.clone(), landed)
    });
    if !landed {
        window.request_animation_frame();
    }
    let ring = focus.is_focused(window) && keyboard;
    let nudge = cap_offset(window, px(density.tokens().font_size));
    Toggle { state, on, hover, flip, ring, nudge }
}

fn flip_toggle(state: &Entity<ToggleState>, owned_outside: bool, handler: Option<&OnToggle>, window: &mut Window, cx: &mut App) {
    let next = state.update(cx, |state, cx| {
        let next = !state.on;
        if !owned_outside {
            state.on = next;
        }
        cx.notify();
        next
    });
    if let Some(handler) = handler {
        handler(next, window, cx);
    }
}

/// The interactive row both toggles share. Each adds its control and label.
fn toggle_row(
    id: ElementId,
    toggle: &Toggle,
    density: Density,
    disabled: bool,
    owned_outside: bool,
    on_change: Option<OnToggle>,
    cx: &App,
) -> Stateful<gpui::Div> {
    let tokens = density.tokens();
    let focus = toggle.state.read(cx).focus.clone();
    let hovered = toggle.state.clone();
    let clicked = toggle.state.clone();
    let keyed = toggle.state.clone();
    let on_click = on_change.clone();
    div()
        .id(id)
        .track_focus(&focus.tab_stop(!disabled))
        .flex()
        .flex_none()
        .items_center()
        .gap(px(tokens.gap))
        .text_size(px(tokens.font_size))
        .line_height(px(tokens.line_height))
        .when(disabled, |this| this.opacity(0.5))
        .when(!disabled, |this| {
            this.cursor_pointer()
                .on_hover(move |over, _, cx| {
                    hovered.update(cx, |state, cx| {
                        state.pointer_over = *over;
                        cx.notify();
                    });
                })
                .on_mouse_down(MouseButton::Left, |_, _, cx| {
                    cx.global_mut::<Theme>().keyboard_input = false;
                })
                .on_click(move |_, window, cx| {
                    flip_toggle(&clicked, owned_outside, on_click.as_ref(), window, cx);
                })
                .on_key_down(move |event: &KeyDownEvent, window, cx| {
                    if event.keystroke.key == "space" {
                        cx.global_mut::<Theme>().keyboard_input = true;
                        flip_toggle(&keyed, owned_outside, on_change.as_ref(), window, cx);
                        cx.stop_propagation();
                    }
                })
        })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Checked {
    Off,
    On,
    Mixed,
}

#[derive(IntoElement)]
pub struct Checkbox {
    id: ElementId,
    label: Option<SharedString>,
    checked: Option<Checked>,
    initial: bool,
    density: Density,
    disabled: bool,
    on_change: Option<OnToggle>,
}

impl Checkbox {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self { id: id.into(), label: None, checked: None, initial: false, density: Density::default(), disabled: false, on_change: None }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Own the value yourself. Without it the checkbox keeps its own.
    pub fn checked(mut self, checked: Checked) -> Self {
        self.checked = Some(checked);
        self
    }

    pub fn initial(mut self, on: bool) -> Self {
        self.initial = on;
        self
    }

    pub fn density(mut self, density: Density) -> Self {
        self.density = density;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_change(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Checkbox {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = Theme::get(cx).colors();
        let given = self.checked.map(|checked| checked == Checked::On);
        let toggle = use_toggle(&self.id, given, self.initial, self.density, window, cx);
        let mark = match self.checked {
            Some(Checked::Mixed) => Some(GLYPH_DASH),
            _ => toggle.on.then_some(GLYPH_TICK),
        };
        let filled = mark.is_some();
        let hover = if self.disabled { 0.0 } else { toggle.hover };
        let side = px(self.density.tokens().icon + 1.);

        let square = div()
            .relative()
            .flex_none()
            .size(side)
            .rounded(px(3.))
            .border_1()
            .border_color(if filled { colors.ink } else { colors.edge })
            .bg(if filled { colors.ink } else { colors.surfaces[1] })
            .when(!filled && hover > 0.0, |this| {
                this.child(checker(fade(colors.ink, DITHER_HOVER_INK * hover), DITHER_HOVER_CELL, 3.))
            })
            .when_some(mark, |this, rows| this.child(glyph(rows, colors.on_solid)))
            .when(toggle.ring, |this| this.child(focus_ring(colors.focus, 3., 2.)));

        let owned_outside = self.checked.is_some();
        toggle_row(self.id, &toggle, self.density, self.disabled, owned_outside, self.on_change, cx)
            .child(square)
            .when_some(self.label, |this, label| this.child(div().relative().top(toggle.nudge).child(label)))
    }
}

#[derive(IntoElement)]
pub struct Switch {
    id: ElementId,
    label: Option<SharedString>,
    on: Option<bool>,
    initial: bool,
    density: Density,
    disabled: bool,
    on_change: Option<OnToggle>,
}

impl Switch {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self { id: id.into(), label: None, on: None, initial: false, density: Density::default(), disabled: false, on_change: None }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Own the value yourself. Without it the switch keeps its own.
    pub fn on(mut self, on: bool) -> Self {
        self.on = Some(on);
        self
    }

    pub fn initial(mut self, on: bool) -> Self {
        self.initial = on;
        self
    }

    pub fn density(mut self, density: Density) -> Self {
        self.density = density;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_change(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Switch {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = Theme::get(cx).colors();
        let toggle = use_toggle(&self.id, self.on, self.initial, self.density, window, cx);
        let height = self.density.tokens().icon + 3.;
        let travel = height * 0.75;
        let flip = toggle.flip;

        let track = div()
            .relative()
            .flex_none()
            .w(px(height + travel))
            .h(px(height))
            .rounded(px(RADIUS_PILL))
            .border_1()
            .border_color(mix(colors.edge, colors.ink, flip.clamp(0.0, 1.0)))
            .bg(mix(colors.wash, colors.ink, flip.clamp(0.0, 1.0)))
            .child(
                div()
                    .absolute()
                    .top(px(2.))
                    .left(px(2. + travel * flip))
                    .size(px(height - 6.))
                    .rounded(px(3.))
                    .bg(mix(colors.ink_soft, colors.on_solid, flip.clamp(0.0, 1.0))),
            )
            .when(toggle.ring, |this| this.child(focus_ring(colors.focus, RADIUS_PILL, 2.)));

        let owned_outside = self.on.is_some();
        toggle_row(self.id, &toggle, self.density, self.disabled, owned_outside, self.on_change, cx)
            .child(track)
            .when_some(self.label, |this, label| this.child(div().relative().top(toggle.nudge).child(label)))
    }
}

type OnPick = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// One choice from a few. Arrow keys move the selection and skip disabled options.
#[derive(IntoElement)]
pub struct RadioGroup {
    id: ElementId,
    options: Vec<(SharedString, bool)>,
    selected: Option<usize>,
    initial: usize,
    density: Density,
    on_change: Option<OnPick>,
}

impl RadioGroup {
    pub fn new<T: Into<SharedString>>(id: impl Into<ElementId>, options: impl IntoIterator<Item = T>) -> Self {
        Self {
            id: id.into(),
            options: options.into_iter().map(|label| (label.into(), false)).collect(),
            selected: None,
            initial: 0,
            density: Density::default(),
            on_change: None,
        }
    }

    pub fn disable(mut self, index: usize) -> Self {
        self.options[index].1 = true;
        self
    }

    /// Own the selection yourself. Without it the group keeps its own.
    pub fn selected(mut self, index: usize) -> Self {
        self.selected = Some(index);
        self
    }

    pub fn initial(mut self, index: usize) -> Self {
        self.initial = index;
        self
    }

    pub fn density(mut self, density: Density) -> Self {
        self.density = density;
        self
    }

    pub fn on_change(mut self, handler: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

struct RadioState {
    focus: FocusHandle,
    selected: usize,
}

fn pick(state: &Entity<RadioState>, index: usize, owned_outside: bool, handler: Option<&OnPick>, window: &mut Window, cx: &mut App) {
    let changed = state.update(cx, |state, cx| {
        let changed = state.selected != index;
        if !owned_outside {
            state.selected = index;
        }
        cx.notify();
        changed
    });
    if let Some(handler) = handler.filter(|_| changed) {
        handler(index, window, cx);
    }
}

fn dot(on: bool, side: Pixels, colors: (Rgba, Rgba, Rgba)) -> impl IntoElement {
    let (ink, surface, edge) = colors;
    div()
        .relative()
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .size(side)
        .rounded_full()
        .border_1()
        .border_color(if on { ink } else { edge })
        .bg(if on { ink } else { surface })
        .when(on, |this| this.child(div().size(side * 0.4).rounded_full().bg(surface)))
}

impl RenderOnce for RadioGroup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let colors = theme.colors();
        let keyboard = theme.keyboard_input;
        let tokens = self.density.tokens();
        let start = self.selected.unwrap_or(self.initial);
        let owned_outside = self.selected.is_some();
        let state = window.use_keyed_state(self.id.clone(), cx, |_, cx| RadioState {
            focus: cx.focus_handle().tab_stop(true),
            selected: start,
        });
        let (focus, selected) = state.update(cx, |state, _| {
            if let Some(index) = self.selected {
                state.selected = index;
            }
            (state.focus.clone(), state.selected)
        });
        let ring = focus.is_focused(window) && keyboard;
        let nudge = cap_offset(window, px(tokens.font_size));
        let side = px(tokens.icon + 1.);
        let on_change = self.on_change;

        let rows: Vec<_> = self
            .options
            .iter()
            .enumerate()
            .map(|(index, (label, disabled))| {
                let clicked = state.clone();
                let handler = on_change.clone();
                let usable = !*disabled;
                div()
                    .id(index)
                    .debug_selector(|| format!("radio-{index}"))
                    .relative()
                    .flex()
                    .flex_none()
                    .items_center()
                    .gap(px(tokens.gap))
                    .when(*disabled, |this| this.opacity(0.5))
                    .when(usable, |this| {
                        this.cursor_pointer().on_click(move |_, window, cx| {
                            cx.global_mut::<Theme>().keyboard_input = false;
                            pick(&clicked, index, owned_outside, handler.as_ref(), window, cx);
                        })
                    })
                    .child(
                        div()
                            .relative()
                            .child(dot(index == selected, side, (colors.ink, colors.surfaces[1], colors.edge)))
                            .when(ring && index == selected, |this| this.child(focus_ring(colors.focus, 8., 2.))),
                    )
                    .child(div().relative().top(nudge).child(label.clone()))
            })
            .collect();

        let pressed = state;
        let options = self.options;
        div()
            .id(self.id)
            .track_focus(&focus)
            .flex()
            .flex_none()
            .items_center()
            .gap(px(tokens.pad))
            .text_size(px(tokens.font_size))
            .line_height(px(tokens.line_height))
            .on_mouse_down(MouseButton::Left, |_, _, cx| {
                cx.global_mut::<Theme>().keyboard_input = false;
            })
            .on_key_down(move |event: &KeyDownEvent, window, cx| {
                let step: isize = match event.keystroke.key.as_str() {
                    "right" | "down" => 1,
                    "left" | "up" => -1,
                    _ => return,
                };
                let usable: Vec<usize> = (0..options.len()).filter(|index| !options[*index].1).collect();
                let from = pressed.read(cx).selected;
                let here = usable.iter().position(|index| *index == from).unwrap_or(0) as isize;
                let count = usable.len() as isize;
                let index = usable[((here + step + count) % count) as usize];
                cx.global_mut::<Theme>().keyboard_input = true;
                pick(&pressed, index, owned_outside, on_change.as_ref(), window, cx);
                cx.stop_propagation();
            })
            .children(rows)
    }
}
