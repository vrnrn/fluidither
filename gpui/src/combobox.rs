//! A text field with a filtered list, and the command menu built from the same parts.
//! Both are views. Create one with `cx.new`, add it as a child, and subscribe to its
//! events. GPUI has no accessibility API, so screen readers see nothing here.

use crate::dialog::Dialog;
use crate::list::{matching, rows_element, step, surface, Mark, Panel, Row};
use crate::text_field::{TextField, TextFieldEvent};
use crate::theme::{Density, Theme};
use gpui::{
    anchored, deferred, div, point, prelude::*, px, Context, Corner, Entity, EventEmitter,
    FocusHandle, Focusable, IntoElement, KeyDownEvent, Render, SharedString, Window,
};
use std::rc::Rc;

const PANEL_GAP: f32 = 6.0;

/// The filtering list both views share.
struct Choosing {
    rows: Rc<Vec<Row>>,
    shown: Vec<usize>,
    active: Option<usize>,
    mark: Mark,
    empty: SharedString,
}

impl Choosing {
    fn new(rows: Vec<Row>) -> Self {
        let shown = (0..rows.len()).collect();
        Self { rows: Rc::new(rows), shown, active: None, mark: Mark::new(), empty: "No matches".into() }
    }

    fn filter(&mut self, query: &str) {
        self.shown = matching(&self.rows, query);
        self.active = step(&self.rows, &self.shown, None, 1);
    }

    fn move_by(&mut self, delta: isize) {
        self.active = step(&self.rows, &self.shown, self.active, delta);
    }

    /// The active row's position among the shown ones, for the mark.
    fn active_position(&self) -> Option<usize> {
        self.active.and_then(|index| self.shown.iter().position(|each| *each == index))
    }
}

fn panel_for<V: 'static>(
    choosing: &mut Choosing,
    selected: Option<usize>,
    density: Density,
    window: &mut Window,
    cx: &mut Context<V>,
    on_hover: impl Fn(&mut V, Option<usize>, &mut Context<V>) + 'static,
    on_pick: impl Fn(&mut V, usize, &mut Window, &mut Context<V>) + 'static,
) -> gpui::Stateful<gpui::Div> {
    let reduced = Theme::get(cx).reduced_motion;
    let (y, opacity, landed) = choosing.mark.advance(choosing.active_position(), density.tokens().segment, reduced);
    if !landed {
        window.request_animation_frame();
    }
    let view = cx.entity().downgrade();
    let picked = view.clone();
    let empty = choosing.shown.is_empty().then(|| choosing.empty.clone());
    rows_element(
        Panel {
            rows: &choosing.rows,
            shown: &choosing.shown,
            selected,
            mark: (y, opacity),
            density,
            empty,
            on_hover: Rc::new(move |row, _, cx| {
                view.update(cx, |this, cx| on_hover(this, row, cx)).ok();
            }),
            on_pick: Rc::new(move |index, window, cx| {
                picked.update(cx, |this, cx| on_pick(this, index, window, cx)).ok();
            }),
        },
        window,
        cx,
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ComboboxEvent {
    Chosen(usize),
}

pub struct Combobox {
    field: Entity<TextField>,
    choosing: Choosing,
    open: bool,
    selected: Option<usize>,
    density: Density,
}

impl EventEmitter<ComboboxEvent> for Combobox {}

impl Focusable for Combobox {
    fn focus_handle(&self, cx: &gpui::App) -> FocusHandle {
        self.field.read(cx).focus_handle(cx)
    }
}

impl Combobox {
    pub fn new<T: Into<Row>>(rows: impl IntoIterator<Item = T>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let field = cx.new(TextField::new);
        cx.subscribe(&field, |this, _, event, cx| {
            if let TextFieldEvent::Changed(text) = event {
                this.choosing.filter(text);
                this.open = true;
                cx.notify();
            }
        })
        .detach();
        let focus = field.read(cx).focus_handle(cx);
        cx.on_blur(&focus, window, |this, _, cx| {
            this.open = false;
            this.choosing.active = None;
            cx.notify();
        })
        .detach();
        Self {
            field,
            choosing: Choosing::new(rows.into_iter().map(Into::into).collect()),
            open: false,
            selected: None,
            density: Density::default(),
        }
    }

    pub fn placeholder(self, placeholder: impl Into<SharedString>, cx: &mut Context<Self>) -> Self {
        let placeholder = placeholder.into();
        self.field.update(cx, |field, cx| {
            field.set_placeholder(placeholder);
            cx.notify();
        });
        self
    }

    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    fn choose(&mut self, index: usize, cx: &mut Context<Self>) {
        self.selected = Some(index);
        self.open = false;
        self.choosing.active = None;
        let label = self.choosing.rows[index].label.to_string();
        self.field.update(cx, |field, cx| field.set_text(label, cx));
        cx.emit(ComboboxEvent::Chosen(index));
        cx.notify();
    }

    fn on_key(&mut self, event: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        match event.keystroke.key.as_str() {
            "down" | "up" => {
                if !self.open {
                    self.open = true;
                    let query = self.field.read(cx).text().to_string();
                    self.choosing.shown = matching(&self.choosing.rows, &query);
                }
                self.choosing.move_by(if event.keystroke.key == "down" { 1 } else { -1 });
            }
            "enter" if self.open => {
                if let Some(index) = self.choosing.active {
                    self.choose(index, cx);
                }
            }
            "escape" if self.open => {
                self.open = false;
                self.choosing.active = None;
            }
            _ => return,
        }
        cx.notify();
        cx.stop_propagation();
    }
}

impl Render for Combobox {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tokens = self.density.tokens();
        let density = self.density;
        let panel = self.open.then(|| {
            let content = panel_for(
                &mut self.choosing,
                self.selected,
                density,
                window,
                cx,
                |this, row, cx| {
                    if row.is_some() && this.choosing.active != row {
                        this.choosing.active = row;
                        cx.notify();
                    }
                },
                |this, index, _, cx| this.choose(index, cx),
            );
            surface("fd-combobox-list", content, density, px(240.), cx).on_mouse_down_out(cx.listener(|this, _, _, cx| {
                this.open = false;
                cx.notify();
            }))
        });
        div()
            .relative()
            .w_full()
            .capture_key_down(cx.listener(Self::on_key))
            .child(self.field.clone())
            .when_some(panel, |this, panel| {
                this.child(
                    deferred(
                        anchored()
                            .anchor(Corner::TopLeft)
                            .offset(point(px(0.), px(tokens.control + PANEL_GAP)))
                            .snap_to_window_with_margin(px(8.))
                            .child(panel),
                    )
                    .with_priority(1),
                )
            })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommandEvent {
    Picked(usize),
}

/// A searchable list of actions in a dialog. Call `show` from your own shortcut.
pub struct CommandMenu {
    field: Entity<TextField>,
    choosing: Choosing,
    open: bool,
}

impl EventEmitter<CommandEvent> for CommandMenu {}

impl CommandMenu {
    pub fn new<T: Into<Row>>(rows: impl IntoIterator<Item = T>, cx: &mut Context<Self>) -> Self {
        let field = cx.new(TextField::new);
        cx.subscribe(&field, |this, _, event, cx| {
            if let TextFieldEvent::Changed(text) = event {
                this.choosing.filter(text);
                cx.notify();
            }
        })
        .detach();
        Self { field, choosing: Choosing::new(rows.into_iter().map(Into::into).collect()), open: false }
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn show(&mut self, cx: &mut Context<Self>) {
        self.open = true;
        self.field.update(cx, |field, cx| field.set_text("", cx));
        self.choosing.filter("");
        cx.notify();
    }

    pub fn close(&mut self, cx: &mut Context<Self>) {
        self.open = false;
        cx.notify();
    }

    fn pick(&mut self, index: usize, cx: &mut Context<Self>) {
        self.open = false;
        cx.emit(CommandEvent::Picked(index));
        cx.notify();
    }

    fn on_key(&mut self, event: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        match event.keystroke.key.as_str() {
            "down" => self.choosing.move_by(1),
            "up" => self.choosing.move_by(-1),
            "enter" => {
                if let Some(index) = self.choosing.active {
                    self.pick(index, cx);
                }
            }
            _ => return,
        }
        cx.notify();
        cx.stop_propagation();
    }
}

impl Render for CommandMenu {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity().downgrade();
        let start = self.field.read(cx).focus_handle(cx);
        let list = self.open.then(|| {
            panel_for(
                &mut self.choosing,
                None,
                Density::default(),
                window,
                cx,
                |this, row, cx| {
                    if row.is_some() && this.choosing.active != row {
                        this.choosing.active = row;
                        cx.notify();
                    }
                },
                |this, index, _, cx| this.pick(index, cx),
            )
        });
        div().child(
            Dialog::new("fd-command", self.open)
                .title("Commands")
                .start_focus(start)
                .on_close(move |_, cx| {
                    view.update(cx, |this, cx| this.close(cx)).ok();
                })
                .body(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(8.))
                        .mt(px(4.))
                        .capture_key_down(cx.listener(Self::on_key))
                        .child(self.field.clone())
                        .children(list),
                ),
        )
    }
}
