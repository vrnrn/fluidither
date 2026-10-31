//! Overlays on GPUI's test platform: real layout, focus and input dispatch, but no
//! GPU, so nothing here proves how they look.
//!
//! `debug_bounds` says where something was drawn, but GPUI never clears it, so it
//! cannot show that something has gone. Those tests check behaviour instead.

use fluidither::list::{matching, step, Typeahead};
use fluidither::{
    Button, Combobox, ComboboxEvent, CommandEvent, CommandMenu, Dialog, Menu, Row, Select,
};
use gpui::{
    div, prelude::*, AnyElement, Context, Entity, FocusHandle, Focusable, Modifiers, Render,
    TestAppContext, VisualTestContext, Window,
};
use std::cell::RefCell;
use std::rc::Rc;

type Build = Rc<dyn Fn() -> AnyElement>;

struct Harness {
    build: Build,
}

impl Render for Harness {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full().flex().flex_col().items_start().child((self.build)())
    }
}

fn open<'a>(cx: &'a mut TestAppContext, build: impl Fn() -> AnyElement + 'static) -> &'a mut VisualTestContext {
    cx.update(fluidither::init);
    let build: Build = Rc::new(build);
    let (_, cx) = cx.add_window_view(move |_, _| Harness { build });
    cx.update(|window, _| window.activate_window());
    cx.run_until_parked();
    cx
}

fn log<T: 'static>() -> Rc<RefCell<Vec<T>>> {
    Rc::new(RefCell::new(Vec::new()))
}

fn rows() -> Vec<Row> {
    vec![Row::new("Rename"), Row::new("Duplicate"), Row::new("Archive").disabled(true), Row::new("Delete")]
}

#[test]
fn stepping_wraps_and_skips_disabled_rows() {
    let rows = rows();
    let shown: Vec<usize> = (0..rows.len()).collect();
    assert_eq!(step(&rows, &shown, None, 1), Some(0));
    assert_eq!(step(&rows, &shown, None, -1), Some(3));
    assert_eq!(step(&rows, &shown, Some(1), 1), Some(3));
    assert_eq!(step(&rows, &shown, Some(3), 1), Some(0));
    assert_eq!(step(&rows, &[2], None, 1), None);
}

#[test]
fn matching_ignores_case_and_finds_any_part() {
    assert_eq!(matching(&rows(), "DE"), vec![3]);
    assert_eq!(matching(&rows(), "e"), vec![0, 1, 2, 3]);
    assert_eq!(matching(&rows(), "zz"), Vec::<usize>::new());
}

#[test]
fn typeahead_cycles_on_one_letter_and_narrows_on_more() {
    let rows = vec![Row::new("Plum"), Row::new("Pear"), Row::new("Peach")];
    let mut typing = Typeahead::default();
    assert_eq!(typing.find(&rows, Some(0), "p"), Some(1));
    assert_eq!(typing.find(&rows, Some(1), "e"), Some(1));
    assert_eq!(typing.find(&rows, Some(1), "a"), Some(1));
    assert_eq!(typing.find(&rows, Some(1), "c"), Some(2));
}

#[gpui::test]
fn a_menu_opens_steps_and_picks_with_the_keyboard(cx: &mut TestAppContext) {
    let seen = log::<usize>();
    let record = seen.clone();
    let cx = open(cx, move || {
        let record = record.clone();
        Menu::new("menu", "Actions", rows()).on_pick(move |index, _, _| record.borrow_mut().push(index)).into_any_element()
    });
    cx.simulate_keystrokes("tab");
    cx.run_until_parked();
    cx.simulate_keystrokes("down");
    cx.simulate_keystrokes("down");
    cx.simulate_keystrokes("down");
    cx.simulate_keystrokes("enter");
    assert_eq!(*seen.borrow(), vec![3], "skips the disabled row");
}

#[gpui::test]
fn escape_closes_a_menu_without_picking(cx: &mut TestAppContext) {
    let seen = log::<usize>();
    let record = seen.clone();
    let cx = open(cx, move || {
        let record = record.clone();
        Menu::new("menu", "Actions", rows()).on_pick(move |index, _, _| record.borrow_mut().push(index)).into_any_element()
    });
    cx.simulate_keystrokes("tab");
    cx.run_until_parked();
    cx.simulate_keystrokes("down");
    cx.simulate_keystrokes("escape");
    assert!(seen.borrow().is_empty());
    // On a closed menu Enter opens it. On an open one it would pick the active row.
    cx.simulate_keystrokes("enter");
    assert!(seen.borrow().is_empty(), "the menu was closed");
    cx.simulate_keystrokes("enter");
    assert_eq!(*seen.borrow(), vec![0], "and now it is open");
}

#[gpui::test]
fn clicking_rows_works_and_skips_disabled_ones(cx: &mut TestAppContext) {
    let seen = log::<usize>();
    let record = seen.clone();
    let cx = open(cx, move || {
        let record = record.clone();
        Menu::new("menu", "Actions", rows()).on_pick(move |index, _, _| record.borrow_mut().push(index)).into_any_element()
    });
    cx.simulate_keystrokes("tab");
    cx.run_until_parked();
    cx.simulate_keystrokes("down");
    cx.run_until_parked();
    let disabled = cx.debug_bounds("row-2").expect("drawn").center();
    cx.simulate_click(disabled, Modifiers::default());
    cx.run_until_parked();
    let usable = cx.debug_bounds("row-1").expect("still open").center();
    cx.simulate_click(usable, Modifiers::default());
    assert_eq!(*seen.borrow(), vec![1]);
}

#[gpui::test]
fn clicking_outside_closes_a_menu(cx: &mut TestAppContext) {
    let seen = log::<usize>();
    let record = seen.clone();
    let cx = open(cx, move || {
        let record = record.clone();
        Menu::new("menu", "Actions", rows()).on_pick(move |index, _, _| record.borrow_mut().push(index)).into_any_element()
    });
    cx.simulate_keystrokes("tab");
    cx.run_until_parked();
    cx.simulate_keystrokes("down");
    cx.run_until_parked();
    let row = cx.debug_bounds("row-0").expect("drawn").center();
    cx.simulate_click(gpui::point(gpui::px(900.), gpui::px(700.)), Modifiers::default());
    cx.run_until_parked();
    cx.simulate_click(row, Modifiers::default());
    cx.run_until_parked();
    assert!(seen.borrow().is_empty(), "nothing was left to click");
}

#[gpui::test]
fn a_select_opens_on_its_choice_and_typing_jumps(cx: &mut TestAppContext) {
    let seen = log::<usize>();
    let record = seen.clone();
    let cx = open(cx, move || {
        let record = record.clone();
        Select::new("select", ["Apple", "Banana", "Pear"]).initial(2).on_change(move |index, _, _| record.borrow_mut().push(index)).into_any_element()
    });
    cx.simulate_keystrokes("tab");
    cx.run_until_parked();
    cx.simulate_keystrokes("enter");
    cx.simulate_keystrokes("up");
    cx.simulate_keystrokes("enter");
    assert_eq!(*seen.borrow(), vec![1]);
    cx.run_until_parked();
    cx.simulate_keystrokes("a");
    cx.simulate_keystrokes("enter");
    assert_eq!(*seen.borrow(), vec![1, 0]);
}

struct DialogHarness {
    open: bool,
    closes: Rc<RefCell<u32>>,
    behind_clicks: Rc<RefCell<u32>>,
    focus: FocusHandle,
}

impl Render for DialogHarness {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let closes = self.closes.clone();
        let behind = self.behind_clicks.clone();
        let view = cx.entity();
        div()
            .size_full()
            .track_focus(&self.focus)
            .child(Button::new("behind", "Behind").on_click(move |_, _, _| *behind.borrow_mut() += 1))
            .child(
                Dialog::new("dialog", self.open)
                    .title("Sure?")
                    .action(Button::new("no", "No"))
                    .action(Button::new("yes", "Yes"))
                    .on_close(move |_, cx| {
                        *closes.borrow_mut() += 1;
                        view.update(cx, |this, cx| {
                            this.open = false;
                            cx.notify();
                        });
                    }),
            )
    }
}

fn open_dialog(cx: &mut TestAppContext) -> (Entity<DialogHarness>, Rc<RefCell<u32>>, Rc<RefCell<u32>>, &mut VisualTestContext) {
    cx.update(fluidither::init);
    let closes = Rc::new(RefCell::new(0));
    let behind = Rc::new(RefCell::new(0));
    let (close_count, behind_count) = (closes.clone(), behind.clone());
    let (view, cx) = cx.add_window_view(move |_, cx| DialogHarness {
        open: false,
        closes: close_count,
        behind_clicks: behind_count,
        focus: cx.focus_handle(),
    });
    cx.update(|window, _| window.activate_window());
    cx.run_until_parked();
    view.update_in(cx, |this, _, cx| {
        this.open = true;
        cx.notify();
    });
    cx.run_until_parked();
    (view, closes, behind, cx)
}

#[gpui::test]
fn escape_closes_a_dialog(cx: &mut TestAppContext) {
    let (_, closes, _, cx) = open_dialog(cx);
    cx.simulate_keystrokes("escape");
    assert_eq!(*closes.borrow(), 1);
}

#[gpui::test]
fn tab_stays_inside_a_dialog_and_the_page_behind_is_blocked(cx: &mut TestAppContext) {
    let (_, _, behind, cx) = open_dialog(cx);
    for _ in 0..7 {
        cx.simulate_keystrokes("tab");
        cx.run_until_parked();
        cx.simulate_keystrokes("enter");
    }
    assert_eq!(*behind.borrow(), 0, "Enter never reached the button behind");
    cx.simulate_click(gpui::point(gpui::px(10.), gpui::px(10.)), Modifiers::default());
    cx.run_until_parked();
    assert_eq!(*behind.borrow(), 0, "the click hit the backdrop");
}

#[gpui::test]
fn focus_returns_when_a_dialog_closes(cx: &mut TestAppContext) {
    cx.update(fluidither::init);
    let (closes, behind) = (Rc::new(RefCell::new(0)), Rc::new(RefCell::new(0)));
    let (view, cx) = cx.add_window_view(move |_, cx| DialogHarness {
        open: false,
        closes,
        behind_clicks: behind,
        focus: cx.focus_handle(),
    });
    let page = view.read_with(cx, |harness, _| harness.focus.clone());
    cx.update(|window, _| {
        window.activate_window();
        window.focus(&page);
    });
    cx.run_until_parked();
    view.update_in(cx, |this, _, cx| {
        this.open = true;
        cx.notify();
    });
    cx.run_until_parked();
    assert!(!cx.update(|window, _| page.is_focused(window)), "the dialog took focus");
    cx.simulate_keystrokes("escape");
    cx.run_until_parked();
    assert!(cx.update(|window, _| page.is_focused(window)), "focus came back");
    cx.update(|_, cx| assert!(fluidither::Theme::get(cx).modal.is_none()));
}

struct ComboHarness {
    combo: Entity<Combobox>,
}

impl Render for ComboHarness {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(self.combo.clone())
    }
}

#[gpui::test]
fn a_combobox_filters_and_chooses(cx: &mut TestAppContext) {
    cx.update(fluidither::init);
    let events = log::<ComboboxEvent>();
    let (view, cx) = cx.add_window_view(|window, cx| ComboHarness {
        combo: cx.new(|cx| Combobox::new(["Lisbon", "London", "Oslo"], window, cx)),
    });
    let combo = view.read_with(cx, |harness, _| harness.combo.clone());
    let record = events.clone();
    cx.update(|window, cx| {
        window.activate_window();
        cx.subscribe(&combo, move |_, event, _| record.borrow_mut().push(event.clone())).detach();
        let handle = combo.read(cx).focus_handle(cx);
        window.focus(&handle);
    });
    cx.run_until_parked();
    cx.simulate_input("lo");
    cx.run_until_parked();
    // Rows have not been drawn before this, so a missing one was filtered out.
    assert!(cx.debug_bounds("row-1").is_some());
    assert!(cx.debug_bounds("row-0").is_none(), "Lisbon has no 'lo'");
    cx.simulate_keystrokes("down");
    cx.simulate_keystrokes("enter");
    assert_eq!(*events.borrow(), vec![ComboboxEvent::Chosen(2)]);
}

#[gpui::test]
fn a_combobox_opens_with_arrows_and_escape_closes(cx: &mut TestAppContext) {
    cx.update(fluidither::init);
    let (view, cx) = cx.add_window_view(|window, cx| ComboHarness {
        combo: cx.new(|cx| Combobox::new(["Lisbon", "London"], window, cx)),
    });
    let combo = view.read_with(cx, |harness, _| harness.combo.clone());
    cx.update(|window, cx| {
        window.activate_window();
        let handle = combo.read(cx).focus_handle(cx);
        window.focus(&handle);
    });
    cx.run_until_parked();
    let is_open = |cx: &mut VisualTestContext| combo.read_with(cx, |combo, _| combo.is_open());
    assert!(!is_open(cx));
    cx.simulate_keystrokes("down");
    assert!(is_open(cx));
    cx.simulate_keystrokes("escape");
    assert!(!is_open(cx));
}

struct CommandHarness {
    menu: Entity<CommandMenu>,
}

impl Render for CommandHarness {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(self.menu.clone())
    }
}

#[gpui::test]
fn the_command_menu_searches_and_picks(cx: &mut TestAppContext) {
    cx.update(fluidither::init);
    let events = log::<CommandEvent>();
    let (view, cx) = cx.add_window_view(|_, cx| CommandHarness {
        menu: cx.new(|cx| CommandMenu::new(["New report", "Open recent", "Switch theme"], cx)),
    });
    let menu = view.read_with(cx, |harness, _| harness.menu.clone());
    let record = events.clone();
    cx.update(|window, cx| {
        window.activate_window();
        cx.subscribe(&menu, move |_, event, _| record.borrow_mut().push(event.clone())).detach();
        menu.update(cx, |menu, cx| menu.show(cx));
    });
    cx.run_until_parked();
    cx.simulate_input("th");
    cx.run_until_parked();
    cx.simulate_keystrokes("enter");
    assert_eq!(*events.borrow(), vec![CommandEvent::Picked(2)]);
    cx.run_until_parked();
    assert!(!menu.read_with(cx, |menu, _| menu.is_open()));
}
