//! Specialised components on GPUI's test platform: real layout, focus and input
//! dispatch, but no GPU, so nothing here proves how they look.

use fluidither::calendar::key_move;
use fluidither::{Calendar, Date, DatePicker, Dots, Swatches};
use gpui::{div, prelude::*, rgb, AnyElement, Context, Modifiers, Render, Rgba, TestAppContext, VisualTestContext, Window};
use std::cell::RefCell;
use std::rc::Rc;

struct Harness {
    build: Rc<dyn Fn() -> AnyElement>,
}

impl Render for Harness {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full().flex().flex_col().items_start().child((self.build)())
    }
}

fn open<'a>(cx: &'a mut TestAppContext, build: impl Fn() -> AnyElement + 'static) -> &'a mut VisualTestContext {
    cx.update(fluidither::init);
    let build: Rc<dyn Fn() -> AnyElement> = Rc::new(build);
    let (_, cx) = cx.add_window_view(move |_, _| Harness { build });
    cx.update(|window, _| window.activate_window());
    cx.run_until_parked();
    cx
}

const DAY: Date = Date { year: 2026, month: 10, day: 14 };

#[test]
fn keys_map_to_the_right_days() {
    assert_eq!(key_move("right", DAY).unwrap().iso(), "2026-10-15");
    assert_eq!(key_move("up", DAY).unwrap().iso(), "2026-10-07");
    assert_eq!(key_move("home", DAY).unwrap().iso(), "2026-10-12");
    assert_eq!(key_move("end", DAY).unwrap().iso(), "2026-10-18");
    assert_eq!(key_move("pagedown", DAY).unwrap().iso(), "2026-11-14");
    assert_eq!(key_move("x", DAY), None);
}

#[gpui::test]
fn the_keyboard_chooses_a_day(cx: &mut TestAppContext) {
    let seen = Rc::new(RefCell::new(Vec::new()));
    let record = seen.clone();
    let cx = open(cx, move || {
        let record = record.clone();
        Calendar::new("calendar", DAY).on_change(move |date, _, _| record.borrow_mut().push(date.iso())).into_any_element()
    });
    cx.simulate_keystrokes("tab");
    cx.run_until_parked();
    cx.simulate_keystrokes("right");
    cx.simulate_keystrokes("down");
    cx.simulate_keystrokes("enter");
    cx.simulate_keystrokes("pagedown");
    cx.simulate_keystrokes("space");
    assert_eq!(*seen.borrow(), vec!["2026-10-22", "2026-11-22"]);
}

#[gpui::test]
fn clicking_a_day_chooses_it_and_arrows_turn_the_month(cx: &mut TestAppContext) {
    let seen = Rc::new(RefCell::new(Vec::new()));
    let record = seen.clone();
    let cx = open(cx, move || {
        let record = record.clone();
        Calendar::new("calendar", DAY).on_change(move |date, _, _| record.borrow_mut().push(date.iso())).into_any_element()
    });
    let day = cx.debug_bounds("day-2026-10-20").expect("drawn").center();
    cx.simulate_click(day, Modifiers::default());
    cx.run_until_parked();
    let next = cx.debug_bounds("next-month").expect("drawn").center();
    cx.simulate_click(next, Modifiers::default());
    cx.run_until_parked();
    let day = cx.debug_bounds("day-2026-11-03").expect("the next month is drawn").center();
    cx.simulate_click(day, Modifiers::default());
    assert_eq!(*seen.borrow(), vec!["2026-10-20", "2026-11-03"]);
}

#[gpui::test]
fn a_date_picker_opens_picks_and_closes(cx: &mut TestAppContext) {
    let seen = Rc::new(RefCell::new(Vec::new()));
    let record = seen.clone();
    let cx = open(cx, move || {
        let record = record.clone();
        DatePicker::new("due", DAY).initial(DAY).on_change(move |date, _, _| record.borrow_mut().push(date.iso())).into_any_element()
    });
    cx.simulate_keystrokes("tab");
    cx.run_until_parked();
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();
    let day = cx.debug_bounds("day-2026-10-21").expect("the calendar opened").center();
    cx.simulate_click(day, Modifiers::default());
    assert_eq!(*seen.borrow(), vec!["2026-10-21"]);
    cx.run_until_parked();
    cx.simulate_click(day, Modifiers::default());
    assert_eq!(seen.borrow().len(), 1, "it closed, so nothing was left to click");
}

fn colors() -> Vec<Rgba> {
    vec![rgb(0xbc4945), rgb(0xb25900), rgb(0x007dc2)]
}

#[gpui::test]
fn swatches_move_with_arrows_and_clicks(cx: &mut TestAppContext) {
    let seen = Rc::new(RefCell::new(Vec::new()));
    let record = seen.clone();
    let cx = open(cx, move || {
        let record = record.clone();
        Swatches::new("accent", colors()).on_change(move |index, _, _| record.borrow_mut().push(index)).into_any_element()
    });
    cx.simulate_keystrokes("tab");
    cx.run_until_parked();
    cx.simulate_keystrokes("left");
    cx.simulate_keystrokes("right");
    let spot = cx.debug_bounds("swatch-1").expect("drawn").center();
    cx.simulate_click(spot, Modifiers::default());
    assert_eq!(*seen.borrow(), vec![2, 0, 1]);
}

#[gpui::test]
fn dots_report_clicks(cx: &mut TestAppContext) {
    let seen = Rc::new(RefCell::new(Vec::new()));
    let record = seen.clone();
    let cx = open(cx, move || {
        let record = record.clone();
        Dots::new("slides", 3, 0).on_change(move |index, _, _| record.borrow_mut().push(index)).into_any_element()
    });
    let spot = cx.debug_bounds("dot-2").expect("drawn").center();
    cx.simulate_click(spot, Modifiers::default());
    assert_eq!(*seen.borrow(), vec![2]);
}
