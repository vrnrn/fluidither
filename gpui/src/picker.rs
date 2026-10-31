//! Menu and Select: a trigger with a floating list. Focus stays on the trigger while
//! the list is open. GPUI has no accessibility API, so screen readers see nothing here.

use crate::generated::{GLYPH_ARROW, RADIUS_CONTROL, WEIGHT_STRONG};
use crate::list::{rows_element, step, surface, Mark, Panel, Row, Typeahead, MOST_ROWS};
use crate::paint::{focus_ring, glyph};
use crate::theme::{cap_offset, Density, Theme};
use gpui::{
    anchored, canvas, deferred, div, point, prelude::*, px, App, Bounds, Corner, ElementId, Entity,
    FocusHandle, FontWeight, IntoElement, KeyDownEvent, MouseButton, Pixels, RenderOnce,
    SharedString, Window,
};
use std::rc::Rc;

type OnPick = Rc<dyn Fn(usize, &mut Window, &mut App)>;

const PANEL_GAP: f32 = 6.0;
const EDGE: f32 = 8.0;

struct PickerState {
    focus: FocusHandle,
    open: bool,
    active: Option<usize>,
    selected: Option<usize>,
    mark: Mark,
    typing: Typeahead,
    trigger: Bounds<Pixels>,
}

#[derive(Clone)]
struct Picking {
    state: Entity<PickerState>,
    rows: Rc<Vec<Row>>,
    shown: Rc<Vec<usize>>,
    /// A select remembers its choice and fills it in when Tab leaves.
    keeps_choice: bool,
    owned_outside: bool,
    on_pick: Option<OnPick>,
}

impl Picking {
    fn set(&self, cx: &mut App, change: impl FnOnce(&mut PickerState)) {
        self.state.update(cx, |state, cx| {
            change(state);
            cx.notify();
        });
    }

    fn open_at(&self, row: Option<usize>, cx: &mut App) {
        self.set(cx, |state| {
            state.open = true;
            state.active = row;
        });
    }

    fn close(&self, cx: &mut App) {
        self.set(cx, |state| {
            state.open = false;
            state.active = None;
        });
    }

    fn choose(&self, index: usize, window: &mut Window, cx: &mut App) {
        let (keeps, outside) = (self.keeps_choice, self.owned_outside);
        self.set(cx, |state| {
            if keeps && !outside {
                state.selected = Some(index);
            }
            state.open = false;
            state.active = None;
        });
        if let Some(handler) = &self.on_pick {
            handler(index, window, cx);
        }
    }

    fn on_key(&self, event: &KeyDownEvent, window: &mut Window, cx: &mut App) {
        let keys = event.keystroke.modifiers;
        if keys.control || keys.platform || keys.alt {
            return;
        }
        let key = event.keystroke.key.as_str();
        let (open, active, selected) = {
            let state = self.state.read(cx);
            (state.open, state.active, state.selected)
        };
        let (rows, shown) = (&self.rows, &self.shown);
        let character = event.keystroke.key_char.as_deref().filter(|text| !text.trim().is_empty());
        let handled = if open {
            match key {
                "down" => self.move_to(step(rows, shown, active, 1), cx),
                "up" => self.move_to(step(rows, shown, active, -1), cx),
                "home" => self.move_to(step(rows, shown, None, 1), cx),
                "end" => self.move_to(step(rows, shown, None, -1), cx),
                "enter" | "space" => {
                    match active {
                        Some(index) => self.choose(index, window, cx),
                        None => self.close(cx),
                    }
                    true
                }
                "escape" => {
                    self.close(cx);
                    true
                }
                "tab" => {
                    match active.filter(|_| self.keeps_choice) {
                        Some(index) => self.choose(index, window, cx),
                        None => self.close(cx),
                    }
                    false
                }
                _ => self.type_ahead(character, active, cx),
            }
        } else {
            match key {
                "down" | "enter" | "space" => {
                    self.open_at(selected.or_else(|| step(rows, shown, None, 1)), cx);
                    true
                }
                "up" => {
                    self.open_at(selected.or_else(|| step(rows, shown, None, -1)), cx);
                    true
                }
                _ if self.keeps_choice && character.is_some() => {
                    self.set(cx, |state| state.open = true);
                    self.type_ahead(character, selected, cx)
                }
                _ => false,
            }
        };
        if handled {
            cx.global_mut::<Theme>().keyboard_input = true;
            cx.stop_propagation();
        }
    }

    fn move_to(&self, row: Option<usize>, cx: &mut App) -> bool {
        self.set(cx, |state| state.active = row);
        true
    }

    fn type_ahead(&self, character: Option<&str>, from: Option<usize>, cx: &mut App) -> bool {
        let Some(character) = character else { return false };
        let rows = self.rows.clone();
        self.set(cx, |state| {
            if let Some(found) = state.typing.find(&rows, from, character) {
                state.active = Some(found);
            }
        });
        true
    }
}

struct Trigger {
    label: SharedString,
    placeholder: bool,
    outline: bool,
}

fn picker(
    id: ElementId,
    rows: Vec<Row>,
    selected: Option<usize>,
    initial: Option<usize>,
    keeps_choice: bool,
    density: Density,
    on_pick: Option<OnPick>,
    trigger_for: impl FnOnce(Option<&Row>) -> Trigger,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    let theme = Theme::get(cx);
    let colors = theme.colors();
    let (reduced, keyboard) = (theme.reduced_motion, theme.keyboard_input);
    let tokens = density.tokens();
    let start = selected.or(initial);
    let state = window.use_keyed_state(id.clone(), cx, |_, cx| PickerState {
        focus: cx.focus_handle().tab_stop(true),
        open: false,
        active: None,
        selected: start,
        mark: Mark::new(),
        typing: Typeahead::default(),
        trigger: Bounds::default(),
    });
    let (focus, open, chosen, mark_y, mark_opacity, landed) = state.update(cx, |state, _| {
        if selected.is_some() {
            state.selected = selected;
        }
        let (y, opacity, landed) = state.mark.advance(state.active, tokens.segment, reduced);
        (state.focus.clone(), state.open, state.selected, y, opacity, landed)
    });
    if !landed {
        window.request_animation_frame();
    }

    let rows = Rc::new(rows);
    let shown = Rc::new((0..rows.len()).collect::<Vec<_>>());
    let picking = Picking { state: state.clone(), rows: rows.clone(), shown: shown.clone(), keeps_choice, owned_outside: selected.is_some(), on_pick };
    let trigger = trigger_for(chosen.map(|index| &rows[index]));
    let show_ring = focus.is_focused(window) && keyboard;
    let nudge = cap_offset(window, px(tokens.font_size));
    let trigger_bounds = state.read(cx).trigger;
    let panel_height = rows.len().min(MOST_ROWS) as f32 * tokens.segment + tokens.inset * 2.0;

    let clicked = picking.clone();
    let keyed = picking.clone();
    let measured = state.clone();
    let hover_state = state.clone();
    let pick = picking.clone();
    let outside = picking;
    let outside_state = state;

    let panel = open.then(|| {
        let content = rows_element(
            Panel {
                rows: &rows,
                shown: &shown,
                selected: chosen.filter(|_| keeps_choice),
                mark: (mark_y, mark_opacity),
                density,
                empty: None,
                on_hover: Rc::new(move |row, _, cx| {
                    hover_state.update(cx, |state, cx| {
                        if state.active != row {
                            state.active = row;
                            cx.notify();
                        }
                    });
                }),
                on_pick: Rc::new(move |index, window, cx| pick.choose(index, window, cx)),
            },
            window,
            cx,
        );
        let min_width = outside_state.read(cx).trigger.size.width;
        surface("fd-list", content, density, min_width.max(px(160.)), cx).on_mouse_down_out(move |event, _, cx| {
            // The trigger's own click toggles it, so leave that one alone.
            if !outside.state.read(cx).trigger.contains(&event.position) {
                outside.close(cx);
            }
        })
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
        .h(px(tokens.control))
        .px(px(tokens.tab_pad))
        .when(!trigger.outline, |this| this.min_w(px(180.)).bg(colors.surfaces[1]))
        .rounded(px(RADIUS_CONTROL))
        .border_1()
        .border_color(if show_ring && !trigger.outline { colors.focus } else if trigger.outline { colors.line } else { colors.edge })
        .text_size(px(tokens.font_size))
        .line_height(px(tokens.line_height))
        .text_color(if trigger.placeholder { colors.ink_soft } else { colors.ink })
        .when(trigger.outline, |this| this.font_weight(FontWeight(WEIGHT_STRONG)))
        .cursor_pointer()
        .on_mouse_down(MouseButton::Left, |_, _, cx| {
            cx.global_mut::<Theme>().keyboard_input = false;
        })
        .on_click(move |_, _, cx| {
            if clicked.state.read(cx).open {
                clicked.close(cx);
            } else {
                let selected = clicked.state.read(cx).selected;
                clicked.open_at(selected, cx);
            }
        })
        .on_key_down(move |event, window, cx| keyed.on_key(event, window, cx))
        .child(canvas(move |bounds, _, cx| measured.update(cx, |state, _| state.trigger = bounds), |_, _, _, _| {}).absolute().size_full())
        .child(div().relative().top(nudge).child(trigger.label))
        .when(!trigger.outline, |this| this.child(div().relative().flex_none().w(px(10.)).h(px(6.)).child(glyph(GLYPH_ARROW, colors.ink_soft))))
        .when(show_ring && trigger.outline, |this| this.child(focus_ring(colors.focus, RADIUS_CONTROL, 2.)))
        .when_some(panel, |this, panel| {
            this.child(deferred(place_list(trigger_bounds, panel_height, tokens.control, window).child(panel)).with_priority(1))
        })
}

/// Below the trigger, or above it when only that side has room.
fn place_list(trigger: Bounds<Pixels>, height: f32, control: f32, window: &Window) -> gpui::Anchored {
    let below = f32::from(window.viewport_size().height - trigger.bottom());
    let above = f32::from(trigger.top());
    let needed = height + PANEL_GAP + EDGE;
    let flip = below < needed && above > below;
    let (corner, offset) = if flip { (Corner::BottomLeft, -PANEL_GAP) } else { (Corner::TopLeft, control + PANEL_GAP) };
    anchored().anchor(corner).offset(point(px(0.), px(offset))).snap_to_window_with_margin(px(EDGE))
}

/// A button that opens a list of actions.
#[derive(IntoElement)]
pub struct Menu {
    id: ElementId,
    label: SharedString,
    rows: Vec<Row>,
    density: Density,
    on_pick: Option<OnPick>,
}

impl Menu {
    pub fn new<T: Into<Row>>(id: impl Into<ElementId>, label: impl Into<SharedString>, rows: impl IntoIterator<Item = T>) -> Self {
        Self { id: id.into(), label: label.into(), rows: rows.into_iter().map(Into::into).collect(), density: Density::default(), on_pick: None }
    }

    pub fn density(mut self, density: Density) -> Self {
        self.density = density;
        self
    }

    pub fn on_pick(mut self, handler: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_pick = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Menu {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let label = self.label;
        picker(self.id, self.rows, None, None, false, self.density, self.on_pick, |_| Trigger { label, placeholder: false, outline: true }, window, cx)
    }
}

/// One choice from a list. It keeps its own choice unless you pass one in.
#[derive(IntoElement)]
pub struct Select {
    id: ElementId,
    rows: Vec<Row>,
    selected: Option<usize>,
    initial: Option<usize>,
    placeholder: SharedString,
    density: Density,
    on_change: Option<OnPick>,
}

impl Select {
    pub fn new<T: Into<Row>>(id: impl Into<ElementId>, rows: impl IntoIterator<Item = T>) -> Self {
        Self { id: id.into(), rows: rows.into_iter().map(Into::into).collect(), selected: None, initial: None, placeholder: SharedString::default(), density: Density::default(), on_change: None }
    }

    pub fn selected(mut self, index: usize) -> Self {
        self.selected = Some(index);
        self
    }

    pub fn initial(mut self, index: usize) -> Self {
        self.initial = Some(index);
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
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

impl RenderOnce for Select {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let placeholder = self.placeholder;
        let trigger = move |chosen: Option<&Row>| Trigger {
            label: chosen.map_or(placeholder.clone(), |row| row.label.clone()),
            placeholder: chosen.is_none(),
            outline: false,
        };
        picker(self.id, self.rows, self.selected, self.initial, true, self.density, self.on_change, trigger, window, cx)
    }
}
