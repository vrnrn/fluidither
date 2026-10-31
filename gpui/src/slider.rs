//! A slider. The filled part is solid and the rest is a dither. Dragging keeps
//! working when the pointer leaves the track. Arrow keys step, Page keys take ten
//! steps, and Home and End jump to the ends.

use crate::generated::RADIUS_CONTROL;
use crate::paint::{checker, focus_ring};
use crate::theme::{fade, Density, Theme};
use gpui::{
    canvas, div, prelude::*, px, relative, App, Bounds, DispatchPhase, ElementId, Entity,
    FocusHandle, IntoElement, KeyDownEvent, MouseButton, MouseMoveEvent, MouseUpEvent, Pixels,
    RenderOnce, Window,
};
use std::rc::Rc;

// Includes the 2px paper border drawn around the 14 by 18 thumb.
const THUMB_WIDTH: f32 = 18.0;
const THUMB_HEIGHT: f32 = 22.0;
const TRACK_HEIGHT: f32 = 6.0;

type OnChange = Rc<dyn Fn(f64, &mut Window, &mut App)>;

#[derive(IntoElement)]
pub struct Slider {
    id: ElementId,
    min: f64,
    max: f64,
    step: f64,
    value: Option<f64>,
    initial: f64,
    density: Density,
    disabled: bool,
    on_change: Option<OnChange>,
}

impl Slider {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            min: 0.0,
            max: 100.0,
            step: 1.0,
            value: None,
            initial: 0.0,
            density: Density::default(),
            disabled: false,
            on_change: None,
        }
    }

    pub fn range(mut self, min: f64, max: f64) -> Self {
        self.min = min;
        self.max = max;
        self
    }

    pub fn step(mut self, step: f64) -> Self {
        self.step = step;
        self
    }

    /// Own the value yourself. Without it the slider keeps its own.
    pub fn value(mut self, value: f64) -> Self {
        self.value = Some(value);
        self
    }

    pub fn initial(mut self, value: f64) -> Self {
        self.initial = value;
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

    pub fn on_change(mut self, handler: impl Fn(f64, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

struct SliderState {
    focus: FocusHandle,
    value: f64,
    track: Bounds<Pixels>,
    dragging: bool,
}

/// Rounds to the nearest step from `min` and keeps the result in range.
pub fn snap(value: f64, min: f64, max: f64, step: f64) -> f64 {
    let stepped = min + ((value - min) / step).round() * step;
    stepped.clamp(min, max)
}

#[derive(Clone)]
struct Scale {
    min: f64,
    max: f64,
    step: f64,
    owned_outside: bool,
    on_change: Option<OnChange>,
}

impl Scale {
    fn set(&self, state: &Entity<SliderState>, value: f64, window: &mut Window, cx: &mut App) {
        let value = snap(value, self.min, self.max, self.step);
        let changed = state.update(cx, |state, cx| {
            let changed = state.value != value;
            if !self.owned_outside {
                state.value = value;
            }
            cx.notify();
            changed
        });
        if let Some(handler) = self.on_change.as_ref().filter(|_| changed) {
            handler(value, window, cx);
        }
    }

    fn set_from_pointer(&self, state: &Entity<SliderState>, x: Pixels, window: &mut Window, cx: &mut App) {
        let track = state.read(cx).track;
        let travel = f32::from(track.size.width) - THUMB_WIDTH;
        if travel <= 0.0 {
            return;
        }
        let along = (f32::from(x - track.origin.x) - THUMB_WIDTH / 2.0) / travel;
        self.set(state, self.min + f64::from(along.clamp(0.0, 1.0)) * (self.max - self.min), window, cx);
    }
}

impl RenderOnce for Slider {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let colors = theme.colors();
        let keyboard = theme.keyboard_input;
        let initial = self.value.unwrap_or(self.initial);
        let state = window.use_keyed_state(self.id.clone(), cx, |_, cx| SliderState {
            focus: cx.focus_handle().tab_stop(true),
            value: initial,
            track: Bounds::default(),
            dragging: false,
        });
        let (focus, value, dragging) = state.update(cx, |state, _| {
            if let Some(value) = self.value {
                state.value = value;
            }
            (state.focus.clone(), state.value, state.dragging)
        });
        let ring = focus.is_focused(window) && keyboard && !self.disabled;
        let share = (((value - self.min) / (self.max - self.min)) as f32).clamp(0.0, 1.0);
        let scale = Scale {
            min: self.min,
            max: self.max,
            step: self.step,
            owned_outside: self.value.is_some(),
            on_change: self.on_change,
        };
        let active = !self.disabled;

        let measured = state.clone();
        let tracked = state.clone();
        let tracked_scale = scale.clone();
        let pressed = state.clone();
        let pressed_scale = scale.clone();
        let keyed = state;
        let keyed_scale = scale;
        let width_thumb = px(THUMB_WIDTH);

        div()
            .id(self.id)
            .track_focus(&focus.tab_stop(active))
            .relative()
            .flex()
            .items_center()
            .w_full()
            .h(px(self.density.tokens().segment))
            .when(self.disabled, |this| this.opacity(0.5))
            .when(active, |this| {
                this.cursor_pointer()
                    .on_mouse_down(MouseButton::Left, move |event, window, cx| {
                        cx.global_mut::<Theme>().keyboard_input = false;
                        pressed.update(cx, |state, _| state.dragging = true);
                        pressed_scale.set_from_pointer(&pressed, event.position.x, window, cx);
                    })
                    .on_key_down(move |event: &KeyDownEvent, window, cx| {
                        let current = keyed.read(cx).value;
                        let step = keyed_scale.step;
                        let next = match event.keystroke.key.as_str() {
                            "right" | "up" => current + step,
                            "left" | "down" => current - step,
                            "pageup" => current + step * 10.0,
                            "pagedown" => current - step * 10.0,
                            "home" => keyed_scale.min,
                            "end" => keyed_scale.max,
                            _ => return,
                        };
                        cx.global_mut::<Theme>().keyboard_input = true;
                        keyed_scale.set(&keyed, next, window, cx);
                        cx.stop_propagation();
                    })
            })
            .child(
                canvas(
                    move |bounds, _, cx| measured.update(cx, |state, _| state.track = bounds),
                    move |_, _, window, _| {
                        if !dragging {
                            return;
                        }
                        // Window-wide, so a drag survives leaving the track.
                        let moved = tracked.clone();
                        let scale = tracked_scale.clone();
                        window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
                            if phase == DispatchPhase::Bubble && event.pressed_button == Some(MouseButton::Left) {
                                scale.set_from_pointer(&moved, event.position.x, window, cx);
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
                .size_full(),
            )
            .child(
                div()
                    .relative()
                    .w_full()
                    .h(px(TRACK_HEIGHT))
                    .child(checker(fade(colors.ink, 0.45), 2.0, 0.0))
                    .child(div().absolute().h_full().w(relative(share)).rounded(px(1.)).bg(colors.ink)),
            )
            .child(
                div()
                    .absolute()
                    .top_0()
                    .bottom_0()
                    .left_0()
                    .right(width_thumb)
                    .child(
                        div()
                            .absolute()
                            .left(relative(share))
                            .top(px((self.density.tokens().segment - THUMB_HEIGHT) / 2.0))
                            .w(width_thumb)
                            .h(px(THUMB_HEIGHT))
                            .rounded(px(RADIUS_CONTROL - 2.))
                            .bg(colors.ink)
                            .border_2()
                            .border_color(colors.paper)
                            .when(ring, |this| this.child(focus_ring(colors.focus, 4., 2.))),
                    ),
            )
    }
}
