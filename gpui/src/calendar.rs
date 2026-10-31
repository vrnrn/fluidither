//! A month grid and a date picker. Arrow keys move by day and week, Page keys by month,
//! Home and End to the edges of the week, and Enter or Space choose. GPUI has no
//! accessibility API, so screen readers see nothing here.

use crate::dates::{month_grid, Date};
use crate::generated::{DITHER_HOVER_CELL, DITHER_HOVER_INK, GLYPH_ARROW, RADIUS_CONTROL, RADIUS_PILL, WEIGHT_STRONG};
use crate::paint::{checker, focus_ring, glyph};
use crate::theme::{cap_offset, fade, Density, Theme};
use gpui::{
    anchored, deferred, div, point, prelude::*, px, App, Bounds, Corner, ElementId, Entity, FocusHandle,
    FontWeight, IntoElement, KeyDownEvent, MouseButton, Pixels, RenderOnce, Window,
};
use std::rc::Rc;

const MONTHS: [&str; 12] = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];
const WEEKDAYS: [&str; 7] = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];
const PANEL_GAP: f32 = 6.0;

type OnChange = Rc<dyn Fn(Date, &mut Window, &mut App)>;

/// Where a key moves the focused day, if it moves it.
pub fn key_move(key: &str, from: Date) -> Option<Date> {
    match key {
        "left" => Some(from.plus_days(-1)),
        "right" => Some(from.plus_days(1)),
        "up" => Some(from.plus_days(-7)),
        "down" => Some(from.plus_days(7)),
        "home" => Some(from.plus_days(-i64::from(from.weekday()))),
        "end" => Some(from.plus_days(6 - i64::from(from.weekday()))),
        "pageup" => Some(from.plus_months(-1)),
        "pagedown" => Some(from.plus_months(1)),
        _ => None,
    }
}

struct CalendarState {
    focus: FocusHandle,
    cursor: Date,
    hovered: Option<Date>,
}

#[derive(IntoElement)]
pub struct Calendar {
    id: ElementId,
    value: Option<Date>,
    start: Date,
    on_change: Option<OnChange>,
}

impl Calendar {
    /// `start` is the month to show when nothing is selected.
    pub fn new(id: impl Into<ElementId>, start: Date) -> Self {
        Self { id: id.into(), value: None, start, on_change: None }
    }

    pub fn value(mut self, value: Date) -> Self {
        self.value = Some(value);
        self
    }

    pub fn on_change(mut self, handler: impl Fn(Date, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Calendar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let colors = theme.colors();
        let keyboard = theme.keyboard_input;
        let tokens = Density::default().tokens();
        let start = self.value.unwrap_or(self.start);
        let state = window.use_keyed_state(self.id.clone(), cx, |_, cx| CalendarState {
            focus: cx.focus_handle().tab_stop(true),
            cursor: start,
            hovered: None,
        });
        let (focus, cursor, hovered) = {
            let state = state.read(cx);
            (state.focus.clone(), state.cursor, state.hovered)
        };
        let ring = focus.is_focused(window) && keyboard;
        let nudge = cap_offset(window, px(tokens.font_size));
        let cell = px(tokens.segment);
        let value = self.value;
        let on_change = self.on_change;

        let days = month_grid(cursor.year, cursor.month);
        let weeks: Vec<_> = days
            .chunks(7)
            .enumerate()
            .map(|(week, dates)| {
                let row = dates.iter().enumerate().map(|(column, date)| {
                    let date = *date;
                    let chosen = value == Some(date);
                    let inside = date.month == cursor.month;
                    let picked = state.clone();
                    let entered = state.clone();
                    let handler = on_change.clone();
                    div()
                        .id(("day", week * 7 + column))
                        .debug_selector(|| format!("day-{}", date.iso()))
                        .relative()
                        .flex()
                        .items_center()
                        .justify_center()
                        .size(cell)
                        .rounded(px(RADIUS_PILL))
                        .cursor_pointer()
                        .text_color(if chosen { colors.on_solid } else if inside { colors.ink } else { colors.ink_soft })
                        .when(chosen, |this| this.bg(colors.ink).font_weight(FontWeight(WEIGHT_STRONG)))
                        .when(hovered == Some(date) && !chosen, |this| {
                            this.child(checker(fade(colors.ink, DITHER_HOVER_INK), DITHER_HOVER_CELL, RADIUS_PILL))
                        })
                        .when(ring && date == cursor, |this| this.child(focus_ring(colors.focus, RADIUS_PILL, 0.)))
                        .on_hover(move |over, _, cx| {
                            entered.update(cx, |state, cx| {
                                if *over {
                                    state.hovered = Some(date);
                                } else if state.hovered == Some(date) {
                                    state.hovered = None;
                                }
                                cx.notify();
                            });
                        })
                        .on_click(move |_, window, cx| {
                            cx.global_mut::<Theme>().keyboard_input = false;
                            picked.update(cx, |state, cx| {
                                state.cursor = date;
                                cx.notify();
                            });
                            if let Some(handler) = &handler {
                                handler(date, window, cx);
                            }
                        })
                        .child(div().relative().top(nudge).child(date.day.to_string()))
                });
                div().flex().gap(px(2.)).children(row)
            })
            .collect();

        let turn = |label: &'static str, step: i32, state: Entity<CalendarState>| {
            div()
                .id(label)
                .debug_selector(move || label.to_string())
                .flex()
                .items_center()
                .justify_center()
                .size(cell)
                .rounded(px(RADIUS_PILL))
                .cursor_pointer()
                .text_color(colors.ink_soft)
                .on_click(move |_, _, cx| {
                    state.update(cx, |state, cx| {
                        state.cursor = state.cursor.plus_months(step);
                        cx.notify();
                    });
                })
                .child(if step < 0 { "<" } else { ">" })
        };

        let pressed = state.clone();
        let handler = on_change;
        div()
            .id(self.id)
            .track_focus(&focus)
            .flex()
            .flex_col()
            .gap(px(2.))
            .flex_none()
            .text_color(colors.ink)
            .text_size(px(tokens.font_size))
            .line_height(px(tokens.line_height))
            .on_mouse_down(MouseButton::Left, |_, _, cx| {
                cx.global_mut::<Theme>().keyboard_input = false;
            })
            .on_key_down(move |event: &KeyDownEvent, window, cx| {
                let cursor = pressed.read(cx).cursor;
                let key = event.keystroke.key.as_str();
                if let Some(next) = key_move(key, cursor) {
                    cx.global_mut::<Theme>().keyboard_input = true;
                    pressed.update(cx, |state, cx| {
                        state.cursor = next;
                        cx.notify();
                    });
                } else if matches!(key, "enter" | "space") {
                    if let Some(handler) = &handler {
                        handler(cursor, window, cx);
                    }
                } else {
                    return;
                }
                cx.stop_propagation();
            })
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(turn("previous-month", -1, state.clone()))
                    .child(div().font_weight(FontWeight(WEIGHT_STRONG)).child(format!("{} {}", MONTHS[cursor.month as usize - 1], cursor.year)))
                    .child(turn("next-month", 1, state)),
            )
            .child(
                div().flex().gap(px(2.)).text_size(px(11.)).text_color(colors.ink_soft).children(
                    WEEKDAYS.map(|name| div().flex().items_center().justify_center().size(cell).child(name)),
                ),
            )
            .children(weeks)
    }
}

struct PickerState {
    focus: FocusHandle,
    open: bool,
    value: Option<Date>,
    trigger: Bounds<Pixels>,
}

/// A trigger that opens a calendar.
#[derive(IntoElement)]
pub struct DatePicker {
    id: ElementId,
    initial: Option<Date>,
    start: Date,
    placeholder: &'static str,
    on_change: Option<OnChange>,
}

impl DatePicker {
    pub fn new(id: impl Into<ElementId>, start: Date) -> Self {
        Self { id: id.into(), initial: None, start, placeholder: "Choose a date", on_change: None }
    }

    pub fn initial(mut self, value: Date) -> Self {
        self.initial = Some(value);
        self
    }

    pub fn on_change(mut self, handler: impl Fn(Date, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for DatePicker {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let colors = theme.colors();
        let keyboard = theme.keyboard_input;
        let panel_shadow = theme.shadow(2);
        let tokens = Density::default().tokens();
        let initial = self.initial;
        let state = window.use_keyed_state(self.id.clone(), cx, |_, cx| PickerState {
            focus: cx.focus_handle().tab_stop(true),
            open: false,
            value: initial,
            trigger: Bounds::default(),
        });
        let (focus, open, value, trigger) = {
            let state = state.read(cx);
            (state.focus.clone(), state.open, state.value, state.trigger)
        };
        let ring = focus.is_focused(window) && keyboard;
        let nudge = cap_offset(window, px(tokens.font_size));
        let label = value.map_or(self.placeholder.to_string(), |date| format!("{} {}, {}", &MONTHS[date.month as usize - 1][..3], date.day, date.year));

        let clicked = state.clone();
        let keyed = state.clone();
        let measured = state.clone();
        let picked = state.clone();
        let outside = state;
        let on_change = self.on_change;
        let start = value.unwrap_or(self.start);
        let id = self.id.clone();

        let panel = open.then(|| {
            let mut calendar = Calendar::new("fd-date-calendar", start).on_change(move |date, window, cx| {
                picked.update(cx, |state, cx| {
                    state.value = Some(date);
                    state.open = false;
                    cx.notify();
                });
                if let Some(handler) = &on_change {
                    handler(date, window, cx);
                }
            });
            if let Some(value) = value {
                calendar = calendar.value(value);
            }
            let height = 7.0 * tokens.segment + 40.0;
            let below = f32::from(window.viewport_size().height - trigger.bottom());
            let flip = below < height + PANEL_GAP + 8.0 && f32::from(trigger.top()) > below;
            let (corner, offset) = if flip { (Corner::BottomLeft, -PANEL_GAP) } else { (Corner::TopLeft, tokens.control + PANEL_GAP) };
            deferred(
                anchored().anchor(corner).offset(point(px(0.), px(offset))).snap_to_window_with_margin(px(8.)).child(
                    div()
                        .id("fd-date-panel")
                        .occlude()
                        .p(px(10.))
                        .rounded(px(RADIUS_PILL + tokens.inset))
                        .bg(colors.surfaces[3])
                        .shadow(panel_shadow)
                        .on_mouse_down_out(move |event, _, cx| {
                            if !outside.read(cx).trigger.contains(&event.position) {
                                outside.update(cx, |state, cx| {
                                    state.open = false;
                                    cx.notify();
                                });
                            }
                        })
                        .child(calendar),
                ),
            )
            .with_priority(1)
        });

        div()
            .id(id)
            .track_focus(&focus)
            .relative()
            .flex()
            .flex_none()
            .items_center()
            .justify_between()
            .gap(px(tokens.gap))
            .min_w(px(180.))
            .h(px(tokens.control))
            .px(px(tokens.tab_pad))
            .rounded(px(RADIUS_CONTROL))
            .border_1()
            .border_color(if ring { colors.focus } else { colors.edge })
            .bg(colors.surfaces[1])
            .text_size(px(tokens.font_size))
            .line_height(px(tokens.line_height))
            .text_color(if value.is_some() { colors.ink } else { colors.ink_soft })
            .cursor_pointer()
            .on_mouse_down(MouseButton::Left, |_, _, cx| {
                cx.global_mut::<Theme>().keyboard_input = false;
            })
            .on_click(move |_, _, cx| {
                clicked.update(cx, |state, cx| {
                    state.open = !state.open;
                    cx.notify();
                });
            })
            .on_key_down(move |event: &KeyDownEvent, _, cx| {
                let key = event.keystroke.key.as_str();
                let open = keyed.read(cx).open;
                let next = match key {
                    "enter" | "space" | "down" if !open => true,
                    "escape" if open => false,
                    _ => return,
                };
                keyed.update(cx, |state, cx| {
                    state.open = next;
                    cx.notify();
                });
                cx.stop_propagation();
            })
            .child(
                gpui::canvas(move |bounds, _, cx| measured.update(cx, |state, _| state.trigger = bounds), |_, _, _, _| {})
                    .absolute()
                    .top_0()
                    .left_0()
                    .size_full(),
            )
            .child(div().relative().top(nudge).child(label))
            .child(div().relative().flex_none().w(px(10.)).h(px(6.)).child(glyph(GLYPH_ARROW, colors.ink_soft)))
            .children(panel)
    }
}
