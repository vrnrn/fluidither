//! A vertical list of destinations for a sidebar. A thumb marks the current one and
//! a dithered peek follows the pointer. Up and Down move the selection.

use crate::generated::{DITHER_HOVER_CELL, DITHER_HOVER_INK, GLIDE, RADIUS_PILL, WEIGHT_REGULAR, WEIGHT_STRONG};
use crate::list::{Mark, Row};
use crate::paint::{checker, focus_ring};
use crate::spring::{Spring, LONGEST_STEP};
use crate::theme::{cap_offset, fade, Density, Theme};
use gpui::{
    div, prelude::*, px, App, ElementId, Entity, FocusHandle, FontWeight, IntoElement, KeyDownEvent,
    MouseButton, RenderOnce, Window,
};
use std::rc::Rc;
use std::time::Instant;

type OnChange = Rc<dyn Fn(usize, &mut Window, &mut App)>;

#[derive(IntoElement)]
pub struct Nav {
    id: ElementId,
    rows: Vec<Row>,
    selected: Option<usize>,
    initial: usize,
    density: Density,
    on_change: Option<OnChange>,
}

impl Nav {
    pub fn new<T: Into<Row>>(id: impl Into<ElementId>, rows: impl IntoIterator<Item = T>) -> Self {
        Self { id: id.into(), rows: rows.into_iter().map(Into::into).collect(), selected: None, initial: 0, density: Density::default(), on_change: None }
    }

    /// Own the selection yourself. Without it the list keeps its own.
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

struct NavState {
    focus: FocusHandle,
    selected: usize,
    peeking: Option<usize>,
    thumb: Spring,
    peek: Mark,
    last_frame: Instant,
}

fn choose(state: &Entity<NavState>, index: usize, owned_outside: bool, handler: Option<&OnChange>, window: &mut Window, cx: &mut App) {
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

impl RenderOnce for Nav {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let colors = theme.colors();
        let (reduced, keyboard) = (theme.reduced_motion, theme.keyboard_input);
        let tokens = self.density.tokens();
        let row_height = tokens.segment;
        let start = self.selected.unwrap_or(self.initial);
        let owned_outside = self.selected.is_some();
        let state = window.use_keyed_state(self.id.clone(), cx, |_, cx| NavState {
            focus: cx.focus_handle().tab_stop(true),
            selected: start,
            peeking: None,
            thumb: Spring::new(GLIDE, f64::from(start as f32 * row_height)),
            peek: Mark::new(),
            last_frame: Instant::now(),
        });
        let (focus, selected, thumb_y, peek_y, peek_opacity, landed) = state.update(cx, |state, _| {
            if let Some(index) = self.selected {
                state.selected = index;
            }
            state.thumb.target = f64::from(state.selected as f32 * row_height);
            if reduced {
                state.thumb.jump(state.thumb.target);
            }
            let dt = state.last_frame.elapsed().as_secs_f64().min(LONGEST_STEP);
            state.last_frame = Instant::now();
            let thumb_landed = state.thumb.step(dt);
            let peek_row = state.peeking.filter(|index| *index != state.selected);
            let (y, opacity, peek_landed) = state.peek.advance(peek_row, row_height, reduced);
            (state.focus.clone(), state.selected, state.thumb.position as f32, y, opacity, thumb_landed && peek_landed)
        });
        if !landed {
            window.request_animation_frame();
        }
        let ring = focus.is_focused(window) && keyboard;
        let nudge = cap_offset(window, px(tokens.font_size));
        let on_change = self.on_change;

        let links: Vec<_> = self
            .rows
            .iter()
            .enumerate()
            .map(|(index, row)| {
                let clicked = state.clone();
                let handler = on_change.clone();
                let hovered = state.clone();
                let on = index == selected;
                let usable = !row.disabled;
                div()
                    .id(("nav", index))
                    .debug_selector(|| format!("nav-{index}"))
                    .relative()
                    .flex()
                    .items_center()
                    .h(px(row_height))
                    .px(px(tokens.tab_pad))
                    .text_color(if on { colors.ink } else { colors.ink_soft })
                    .font_weight(FontWeight(if on { WEIGHT_STRONG } else { WEIGHT_REGULAR }))
                    .when(row.disabled, |this| this.opacity(0.5))
                    .when(usable, |this| {
                        this.cursor_pointer()
                            .on_hover(move |over, _, cx| {
                                hovered.update(cx, |state, cx| {
                                    if *over {
                                        state.peeking = Some(index);
                                    } else if state.peeking == Some(index) {
                                        state.peeking = None;
                                    }
                                    cx.notify();
                                });
                            })
                            .on_click(move |_, window, cx| {
                                cx.global_mut::<Theme>().keyboard_input = false;
                                choose(&clicked, index, owned_outside, handler.as_ref(), window, cx);
                            })
                    })
                    .child(div().relative().top(nudge).child(row.label.clone()))
            })
            .collect();

        let pressed = state;
        let rows = self.rows;
        div()
            .id(self.id)
            .track_focus(&focus)
            .relative()
            .flex()
            .flex_col()
            .w_full()
            .text_size(px(tokens.font_size))
            .line_height(px(tokens.line_height))
            .on_mouse_down(MouseButton::Left, |_, _, cx| {
                cx.global_mut::<Theme>().keyboard_input = false;
            })
            .on_key_down(move |event: &KeyDownEvent, window, cx| {
                let usable: Vec<usize> = (0..rows.len()).filter(|index| !rows[*index].disabled).collect();
                let here = usable.iter().position(|index| *index == pressed.read(cx).selected).unwrap_or(0);
                let target = match event.keystroke.key.as_str() {
                    "down" => usable[(here + 1) % usable.len()],
                    "up" => usable[(here + usable.len() - 1) % usable.len()],
                    _ => return,
                };
                cx.global_mut::<Theme>().keyboard_input = true;
                choose(&pressed, target, owned_outside, on_change.as_ref(), window, cx);
                cx.stop_propagation();
            })
            .child(
                div()
                    .absolute()
                    .top(px(thumb_y))
                    .left_0()
                    .right_0()
                    .h(px(row_height))
                    .rounded(px(RADIUS_PILL))
                    .bg(colors.soft)
                    .when(ring, |this| this.child(focus_ring(colors.focus, RADIUS_PILL, 0.))),
            )
            .when(peek_opacity > 0.0, |this| {
                this.child(
                    div()
                        .absolute()
                        .top(px(peek_y))
                        .left_0()
                        .right_0()
                        .h(px(row_height))
                        .child(checker(fade(colors.ink, DITHER_HOVER_INK * peek_opacity), DITHER_HOVER_CELL, RADIUS_PILL)),
                )
            })
            .children(links)
    }
}
