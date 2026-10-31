//! Drives the components through GPUI's test platform: real layout, focus and
//! input dispatch, but no GPU, so nothing here proves how they look.

use fluidither::{Button, Tab, Tabs};
use gpui::{
    div, point, prelude::*, px, Context, Modifiers, Render, TestAppContext, VisualTestContext, Window,
};
use std::cell::RefCell;
use std::rc::Rc;

struct TabsHarness {
    changes: Rc<RefCell<Vec<usize>>>,
}

impl Render for TabsHarness {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let changes = self.changes.clone();
        div().size_full().flex().child(
            Tabs::new(
                "tabs",
                [
                    Tab::new("Day"),
                    Tab::new("Week"),
                    Tab::new("Year").disabled(true),
                    Tab::new("All"),
                ],
            )
            .on_change(move |index, _, _| changes.borrow_mut().push(index)),
        )
    }
}

fn open_tabs(cx: &mut TestAppContext) -> (Rc<RefCell<Vec<usize>>>, &mut VisualTestContext) {
    cx.update(fluidither::init);
    let changes = Rc::new(RefCell::new(Vec::new()));
    let harness = changes.clone();
    let (_, cx) = cx.add_window_view(move |_, _| TabsHarness { changes: harness });
    cx.update(|window, _| window.activate_window());
    cx.run_until_parked();
    (changes, cx)
}

#[gpui::test]
fn arrow_keys_select_and_skip_disabled_tabs(cx: &mut TestAppContext) {
    let (changes, cx) = open_tabs(cx);
    cx.simulate_keystrokes("tab");
    cx.run_until_parked();
    cx.simulate_keystrokes("right");
    cx.simulate_keystrokes("right");
    assert_eq!(*changes.borrow(), vec![1, 3], "skips the disabled third tab");
}

#[gpui::test]
fn arrow_keys_wrap_at_both_ends(cx: &mut TestAppContext) {
    let (changes, cx) = open_tabs(cx);
    cx.simulate_keystrokes("tab");
    cx.run_until_parked();
    cx.simulate_keystrokes("left");
    cx.simulate_keystrokes("right");
    assert_eq!(*changes.borrow(), vec![3, 0]);
}

#[gpui::test]
fn home_and_end_jump_to_the_first_and_last_enabled_tab(cx: &mut TestAppContext) {
    let (changes, cx) = open_tabs(cx);
    cx.simulate_keystrokes("tab");
    cx.run_until_parked();
    cx.simulate_keystrokes("end");
    cx.simulate_keystrokes("home");
    assert_eq!(*changes.borrow(), vec![3, 0]);
}

#[gpui::test]
fn keys_do_nothing_without_focus(cx: &mut TestAppContext) {
    let (changes, cx) = open_tabs(cx);
    cx.simulate_keystrokes("right");
    assert!(changes.borrow().is_empty());
}

#[gpui::test]
fn modified_arrows_are_left_alone(cx: &mut TestAppContext) {
    let (changes, cx) = open_tabs(cx);
    cx.simulate_keystrokes("tab");
    cx.run_until_parked();
    cx.simulate_keystrokes("cmd-right");
    assert!(changes.borrow().is_empty());
}

#[gpui::test]
fn clicking_selects_a_tab_but_not_a_disabled_one(cx: &mut TestAppContext) {
    let (changes, cx) = open_tabs(cx);
    let center = |cx: &mut VisualTestContext, name: &'static str| {
        cx.debug_bounds(name).unwrap_or_else(|| panic!("{name} was not drawn")).center()
    };
    for name in ["tab-1", "tab-2", "tab-3"] {
        let position = center(cx, name);
        cx.simulate_click(position, Modifiers::default());
        cx.run_until_parked();
    }
    assert_eq!(*changes.borrow(), vec![1, 3], "the disabled tab never fires");
}

struct ButtonHarness {
    clicks: Rc<RefCell<u32>>,
    disabled: bool,
    loading: bool,
}

impl Render for ButtonHarness {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let clicks = self.clicks.clone();
        div().size_full().child(
            div().flex().child(
                Button::new("button", "Save")
                    .disabled(self.disabled)
                    .busy(self.loading)
                    .on_click(move |_, _, _| *clicks.borrow_mut() += 1),
            ),
        )
    }
}

fn open_button(cx: &mut TestAppContext, disabled: bool, loading: bool) -> (Rc<RefCell<u32>>, &mut VisualTestContext) {
    cx.update(fluidither::init);
    let clicks = Rc::new(RefCell::new(0));
    let harness = clicks.clone();
    let (_, cx) = cx.add_window_view(move |_, _| ButtonHarness { clicks: harness, disabled, loading });
    cx.update(|window, _| window.activate_window());
    cx.run_until_parked();
    (clicks, cx)
}

#[gpui::test]
fn a_button_clicks_with_the_mouse(cx: &mut TestAppContext) {
    let (clicks, cx) = open_button(cx, false, false);
    cx.simulate_click(point(px(10.), px(10.)), Modifiers::default());
    assert_eq!(*clicks.borrow(), 1);
}

#[gpui::test]
fn a_focused_button_clicks_with_enter_and_space(cx: &mut TestAppContext) {
    let (clicks, cx) = open_button(cx, false, false);
    cx.simulate_keystrokes("tab");
    cx.run_until_parked();
    cx.simulate_keystrokes("enter");
    cx.simulate_keystrokes("space");
    assert_eq!(*clicks.borrow(), 2);
}

#[gpui::test]
fn disabled_and_loading_buttons_ignore_input(cx: &mut TestAppContext) {
    for (disabled, loading) in [(true, false), (false, true)] {
        let (clicks, cx) = open_button(cx, disabled, loading);
        cx.simulate_click(point(px(10.), px(10.)), Modifiers::default());
        cx.simulate_keystrokes("tab");
        cx.run_until_parked();
        cx.simulate_keystrokes("enter");
        assert_eq!(*clicks.borrow(), 0);
    }
}
