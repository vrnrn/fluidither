//! Form controls on GPUI's test platform: real layout, focus and input dispatch,
//! but no GPU, so nothing here proves how they look.

use fluidither::slider::snap;
use fluidither::{Checkbox, Checked, RadioGroup, Slider, Switch, TextField, TextFieldEvent};
use gpui::{
    div, point, prelude::*, px, AnyElement, Context, Entity, Focusable, Modifiers, Render, TestAppContext,
    VisualTestContext, Window,
};
use std::cell::RefCell;
use std::rc::Rc;

type Build = Rc<dyn Fn() -> AnyElement>;

struct Harness {
    build: Build,
}

impl Render for Harness {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full().flex().flex_col().child((self.build)())
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

const TOP_LEFT: (f32, f32) = (6., 6.);

fn click_corner(cx: &mut VisualTestContext) {
    cx.simulate_click(point(px(TOP_LEFT.0), px(TOP_LEFT.1)), Modifiers::default());
    cx.run_until_parked();
}

#[gpui::test]
fn a_checkbox_toggles_with_the_mouse_and_with_space(cx: &mut TestAppContext) {
    let seen = log::<bool>();
    let record = seen.clone();
    let cx = open(cx, move || {
        let record = record.clone();
        Checkbox::new("box").label("Tick").on_change(move |on, _, _| record.borrow_mut().push(on)).into_any_element()
    });
    click_corner(cx);
    click_corner(cx);
    cx.simulate_keystrokes("tab");
    cx.run_until_parked();
    cx.simulate_keystrokes("space");
    assert_eq!(*seen.borrow(), vec![true, false, true]);
}

#[gpui::test]
fn a_mixed_checkbox_asks_for_on(cx: &mut TestAppContext) {
    let seen = log::<bool>();
    let record = seen.clone();
    let cx = open(cx, move || {
        let record = record.clone();
        Checkbox::new("box").checked(Checked::Mixed).on_change(move |on, _, _| record.borrow_mut().push(on)).into_any_element()
    });
    click_corner(cx);
    assert_eq!(*seen.borrow(), vec![true]);
}

#[gpui::test]
fn disabled_toggles_ignore_input(cx: &mut TestAppContext) {
    let seen = log::<bool>();
    let (box_log, switch_log) = (seen.clone(), seen.clone());
    let cx = open(cx, move || {
        let (box_log, switch_log) = (box_log.clone(), switch_log.clone());
        div()
            .child(Checkbox::new("box").disabled(true).on_change(move |on, _, _| box_log.borrow_mut().push(on)))
            .child(Switch::new("switch").disabled(true).on_change(move |on, _, _| switch_log.borrow_mut().push(on)))
            .into_any_element()
    });
    click_corner(cx);
    cx.simulate_keystrokes("tab");
    cx.run_until_parked();
    cx.simulate_keystrokes("space");
    assert!(seen.borrow().is_empty());
}

#[gpui::test]
fn a_switch_toggles_with_the_mouse_and_with_space(cx: &mut TestAppContext) {
    let seen = log::<bool>();
    let record = seen.clone();
    let cx = open(cx, move || {
        let record = record.clone();
        Switch::new("switch").initial(true).on_change(move |on, _, _| record.borrow_mut().push(on)).into_any_element()
    });
    click_corner(cx);
    cx.simulate_keystrokes("tab");
    cx.run_until_parked();
    cx.simulate_keystrokes("space");
    assert_eq!(*seen.borrow(), vec![false, true]);
}

#[gpui::test]
fn radio_arrows_wrap_and_skip_disabled_options(cx: &mut TestAppContext) {
    let seen = log::<usize>();
    let record = seen.clone();
    let cx = open(cx, move || {
        let record = record.clone();
        RadioGroup::new("plan", ["A", "B", "C"]).disable(1).on_change(move |index, _, _| record.borrow_mut().push(index)).into_any_element()
    });
    cx.simulate_keystrokes("tab");
    cx.run_until_parked();
    cx.simulate_keystrokes("right");
    cx.simulate_keystrokes("right");
    cx.simulate_keystrokes("left");
    assert_eq!(*seen.borrow(), vec![2, 0, 2]);
}

#[gpui::test]
fn a_disabled_radio_option_cannot_be_clicked(cx: &mut TestAppContext) {
    let seen = log::<usize>();
    let record = seen.clone();
    let cx = open(cx, move || {
        let record = record.clone();
        RadioGroup::new("plan", ["A", "B", "C"]).disable(1).on_change(move |index, _, _| record.borrow_mut().push(index)).into_any_element()
    });
    for name in ["radio-1", "radio-2"] {
        let spot = cx.debug_bounds(name).expect("drawn").center();
        cx.simulate_click(spot, Modifiers::default());
        cx.run_until_parked();
    }
    assert_eq!(*seen.borrow(), vec![2]);
}

#[test]
fn slider_values_snap_to_steps_and_stay_in_range() {
    assert_eq!(snap(47.4, 0.0, 100.0, 5.0), 45.0);
    assert_eq!(snap(47.6, 0.0, 100.0, 5.0), 50.0);
    assert_eq!(snap(120.0, 0.0, 100.0, 5.0), 100.0);
    assert_eq!(snap(-3.0, 0.0, 100.0, 1.0), 0.0);
    assert_eq!(snap(1.3, 1.0, 2.0, 0.25), 1.25);
}

#[gpui::test]
fn slider_keys_step_jump_and_stop_at_the_ends(cx: &mut TestAppContext) {
    let seen = log::<f64>();
    let record = seen.clone();
    let cx = open(cx, move || {
        let record = record.clone();
        Slider::new("volume").initial(50.0).step(5.0).on_change(move |value, _, _| record.borrow_mut().push(value)).into_any_element()
    });
    cx.simulate_keystrokes("tab");
    cx.run_until_parked();
    for keys in ["right", "left", "left", "pageup", "end", "right", "home"] {
        cx.simulate_keystrokes(keys);
    }
    assert_eq!(*seen.borrow(), vec![55.0, 50.0, 45.0, 95.0, 100.0, 0.0]);
}

#[gpui::test]
fn clicking_a_slider_moves_it_there(cx: &mut TestAppContext) {
    let seen = log::<f64>();
    let record = seen.clone();
    let cx = open(cx, move || {
        let record = record.clone();
        Slider::new("volume").on_change(move |value, _, _| record.borrow_mut().push(value)).into_any_element()
    });
    let width = f32::from(cx.window_bounds_width());
    cx.simulate_mouse_down(point(px(width / 2.0), px(14.)), gpui::MouseButton::Left, Modifiers::default());
    cx.run_until_parked();
    let value = *seen.borrow().last().expect("the click set a value");
    assert!((value - 50.0).abs() <= 1.0, "{value}");
}

trait WindowWidth {
    fn window_bounds_width(&mut self) -> gpui::Pixels;
}

impl WindowWidth for VisualTestContext {
    fn window_bounds_width(&mut self) -> gpui::Pixels {
        self.update(|window, _| window.bounds().size.width)
    }
}

struct FieldHarness {
    field: Entity<TextField>,
}

impl Render for FieldHarness {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(self.field.clone())
    }
}

fn open_field<'a>(
    cx: &'a mut TestAppContext,
    start: &'static str,
) -> (Entity<TextField>, Rc<RefCell<Vec<TextFieldEvent>>>, &'a mut VisualTestContext) {
    cx.update(fluidither::init);
    let events = log::<TextFieldEvent>();
    let (field, cx) = cx.add_window_view(|_, cx| {
        let field = cx.new(|cx| TextField::new(cx).with_text(start));
        FieldHarness { field }
    });
    let field = field.read_with(cx, |harness, _| harness.field.clone());
    let record = events.clone();
    cx.update(|window, cx| {
        window.activate_window();
        cx.subscribe(&field, move |_, event, _| record.borrow_mut().push(event.clone())).detach();
        let handle = field.read(cx).focus_handle(cx);
        window.focus(&handle);
    });
    cx.run_until_parked();
    (field, events, cx)
}

fn text(field: &Entity<TextField>, cx: &mut VisualTestContext) -> String {
    field.read_with(cx, |field, _| field.text().to_string())
}

#[gpui::test]
fn typing_inserts_at_the_caret(cx: &mut TestAppContext) {
    let (field, events, cx) = open_field(cx, "");
    cx.simulate_input("helo");
    cx.simulate_keystrokes("left");
    cx.simulate_input("l");
    assert_eq!(text(&field, cx), "hello");
    assert_eq!(events.borrow().last(), Some(&TextFieldEvent::Changed("hello".into())));
}

#[gpui::test]
fn backspace_removes_a_whole_emoji(cx: &mut TestAppContext) {
    let (field, _, cx) = open_field(cx, "ab👩‍👩‍👧");
    cx.simulate_keystrokes("backspace");
    assert_eq!(text(&field, cx), "ab");
    cx.simulate_keystrokes("backspace");
    assert_eq!(text(&field, cx), "a");
}

#[gpui::test]
fn delete_removes_forward_and_arrows_move_by_character(cx: &mut TestAppContext) {
    let (field, _, cx) = open_field(cx, "héllo");
    cx.simulate_keystrokes("home");
    cx.simulate_keystrokes("right");
    cx.simulate_keystrokes("delete");
    assert_eq!(text(&field, cx), "hllo");
}

#[gpui::test]
fn shift_selects_and_typing_replaces_the_selection(cx: &mut TestAppContext) {
    let (field, _, cx) = open_field(cx, "hello world");
    cx.simulate_keystrokes("shift-left");
    cx.simulate_keystrokes("shift-left");
    cx.simulate_input("LD");
    assert_eq!(text(&field, cx), "hello worLD");
}

#[gpui::test]
fn word_and_line_keys_jump(cx: &mut TestAppContext) {
    let (field, _, cx) = open_field(cx, "one two three");
    cx.simulate_keystrokes("alt-left");
    cx.simulate_input("X");
    assert_eq!(text(&field, cx), "one two Xthree");
    cx.simulate_keystrokes("cmd-left");
    cx.simulate_input("Y");
    assert_eq!(text(&field, cx), "Yone two Xthree");
    cx.simulate_keystrokes("alt-right");
    cx.simulate_keystrokes("alt-backspace");
    assert_eq!(text(&field, cx), " two Xthree");
}

#[gpui::test]
fn select_all_copy_cut_and_paste_use_the_clipboard(cx: &mut TestAppContext) {
    let (field, _, cx) = open_field(cx, "copy me");
    cx.simulate_keystrokes("cmd-a");
    cx.simulate_keystrokes("cmd-c");
    cx.simulate_keystrokes("right");
    cx.simulate_keystrokes("cmd-v");
    assert_eq!(text(&field, cx), "copy mecopy me");
    cx.simulate_keystrokes("cmd-a");
    cx.simulate_keystrokes("cmd-x");
    assert_eq!(text(&field, cx), "");
    cx.simulate_keystrokes("cmd-v");
    assert_eq!(text(&field, cx), "copy mecopy me");
}

#[gpui::test]
fn enter_submits(cx: &mut TestAppContext) {
    let (_, events, cx) = open_field(cx, "done");
    cx.simulate_keystrokes("enter");
    assert_eq!(events.borrow().last(), Some(&TextFieldEvent::Submitted("done".into())));
}

#[gpui::test]
fn tab_leaves_the_field_alone(cx: &mut TestAppContext) {
    let (field, _, cx) = open_field(cx, "stay");
    cx.simulate_keystrokes("tab");
    assert_eq!(text(&field, cx), "stay");
}
