//! Structure components on GPUI's test platform: real layout, focus and input
//! dispatch, but no GPU, so nothing here proves how they look.
//!
//! `debug_bounds` is never cleared by GPUI, so it cannot show that something has
//! gone. Accordion bodies count their own paints instead.

use fluidither::scroll_area::bar_span;
use fluidither::{Accordion, Badge, Banner, Card, Nav, Row, ScrollArea, Table, Tone};
use gpui::{canvas, div, prelude::*, px, AnyElement, Context, Modifiers, Render, TestAppContext, VisualTestContext, Window};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

struct Harness {
    build: Rc<dyn Fn() -> AnyElement>,
}

impl Render for Harness {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full().flex().flex_col().child((self.build)())
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

type Paints = Rc<[Cell<u32>; 2]>;

// A body that counts how often it is painted.
fn counted(paints: &Paints, index: usize) -> impl IntoElement {
    let paints = paints.clone();
    canvas(|_, _, _| (), move |_, _, _, _| paints[index].set(paints[index].get() + 1)).w(px(100.)).h(px(20.))
}

fn accordion(paints: &Paints, multiple: bool) -> AnyElement {
    let built = Accordion::new("faq").section("First", counted(paints, 0)).section("Second", counted(paints, 1)).initial(0);
    (if multiple { built.multiple() } else { built }).into_any_element()
}

// Springs run on real time, so draw frames until they have landed.
fn settle(cx: &mut VisualTestContext) {
    for _ in 0..30 {
        std::thread::sleep(std::time::Duration::from_millis(16));
        cx.update(|window, _| window.refresh());
        cx.run_until_parked();
    }
}

/// Which bodies paint in one fresh frame.
fn showing(paints: &Paints, cx: &mut VisualTestContext) -> [bool; 2] {
    paints[0].set(0);
    paints[1].set(0);
    cx.update(|window, _| window.refresh());
    cx.run_until_parked();
    [paints[0].get() > 0, paints[1].get() > 0]
}

fn paints() -> Paints {
    Rc::new([Cell::new(0), Cell::new(0)])
}

#[gpui::test]
fn opening_one_section_closes_the_other(cx: &mut TestAppContext) {
    let counts = paints();
    let build = counts.clone();
    let cx = open(cx, move || accordion(&build, false));
    assert_eq!(showing(&counts, cx), [true, false]);
    let header = cx.debug_bounds("section-1").expect("drawn").center();
    cx.simulate_click(header, Modifiers::default());
    settle(cx);
    assert_eq!(showing(&counts, cx), [false, true]);
}

#[gpui::test]
fn multiple_sections_can_stay_open(cx: &mut TestAppContext) {
    let counts = paints();
    let build = counts.clone();
    let cx = open(cx, move || accordion(&build, true));
    let header = cx.debug_bounds("section-1").expect("drawn").center();
    cx.simulate_click(header, Modifiers::default());
    settle(cx);
    assert_eq!(showing(&counts, cx), [true, true]);
}

#[gpui::test]
fn enter_and_space_toggle_a_focused_section(cx: &mut TestAppContext) {
    let counts = paints();
    let build = counts.clone();
    let cx = open(cx, move || accordion(&build, false));
    cx.simulate_keystrokes("tab");
    cx.simulate_keystrokes("tab");
    cx.run_until_parked();
    cx.simulate_keystrokes("enter");
    settle(cx);
    assert_eq!(showing(&counts, cx), [false, true]);
    cx.simulate_keystrokes("space");
    settle(cx);
    assert_eq!(showing(&counts, cx), [false, false]);
}

#[gpui::test]
fn nav_moves_with_keys_and_clicks_and_skips_disabled_rows(cx: &mut TestAppContext) {
    let seen = Rc::new(RefCell::new(Vec::new()));
    let record = seen.clone();
    let cx = open(cx, move || {
        let record = record.clone();
        Nav::new("side", [Row::new("Overview"), Row::new("Billing").disabled(true), Row::new("Settings")])
            .on_change(move |index, _, _| record.borrow_mut().push(index))
            .into_any_element()
    });
    cx.simulate_keystrokes("tab");
    cx.run_until_parked();
    cx.simulate_keystrokes("down");
    cx.simulate_keystrokes("down");
    let disabled = cx.debug_bounds("nav-1").expect("drawn").center();
    cx.simulate_click(disabled, Modifiers::default());
    cx.run_until_parked();
    let last = cx.debug_bounds("nav-2").expect("drawn").center();
    cx.simulate_click(last, Modifiers::default());
    assert_eq!(*seen.borrow(), vec![2, 0, 2], "down, down wraps, then the click");
}

#[test]
fn the_scroll_bar_tracks_position_and_hides_when_it_fits() {
    assert_eq!(bar_span(100.0, 100.0, 0.0), None);
    assert_eq!(bar_span(100.0, 400.0, 0.0), Some((0.0, 25.0)));
    assert_eq!(bar_span(100.0, 400.0, 300.0), Some((75.0, 25.0)));
    assert_eq!(bar_span(100.0, 10_000.0, 0.0).map(|span| span.1), Some(24.0), "stays grabbable");
}

#[gpui::test]
fn everything_static_draws_without_trouble(cx: &mut TestAppContext) {
    let cx = open(cx, || {
        div()
            .child(Badge::new("Live").tone(Tone::Green))
            .child(Banner::new("Hello").tone(Tone::Orange).action(div().child("Act")))
            .child(Card::new().title("Title").text("Text").raised().child(div().child("More")))
            .child(Table::new("table").column("Name").number_column("Amount").row(["A", "1"]).row(["B", "2"]))
            .child(ScrollArea::new("area").h(px(40.)).child(div().h(px(400.))))
            .into_any_element()
    });
    cx.run_until_parked();
}
