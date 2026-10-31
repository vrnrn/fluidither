//! The last parts on GPUI's test platform: real layout, focus and input dispatch, but no
//! GPU, so nothing here proves how they look.

use fluidither::export::save_png;
use fluidither::extras::hsv_to_rgb;
use fluidither::{
    ColorPicker, Compare, Composer, ComposerEvent, Dialog, Marquee, NotFound, QuestionCard, Sidebar,
    Step, Steps,
};
use gpui::{div, prelude::*, px, rgb, AnyElement, Context, Entity, Focusable, Modifiers, Render, Rgba, TestAppContext, VisualTestContext, Window};
use std::cell::{Cell, RefCell};
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

fn close(a: Rgba, b: Rgba) -> bool {
    (a.r - b.r).abs() < 0.01 && (a.g - b.g).abs() < 0.01 && (a.b - b.b).abs() < 0.01
}

#[test]
fn hsv_gives_the_colors_you_would_expect() {
    assert!(close(hsv_to_rgb(0.0, 1.0, 1.0), rgb(0xff0000)));
    assert!(close(hsv_to_rgb(120.0, 1.0, 1.0), rgb(0x00ff00)));
    assert!(close(hsv_to_rgb(240.0, 1.0, 1.0), rgb(0x0000ff)));
    assert!(close(hsv_to_rgb(60.0, 1.0, 1.0), rgb(0xffff00)));
    assert!(close(hsv_to_rgb(200.0, 0.0, 0.5), rgb(0x808080)));
    assert!(close(hsv_to_rgb(360.0, 1.0, 1.0), rgb(0xff0000)));
}

#[test]
fn a_png_holds_the_cells_at_full_size() {
    let path = std::env::temp_dir().join("fluidither-export-test.png");
    save_png(&path, &[1, 0, 0, 1], 2, 2, 3, rgb(0xff0000)).unwrap();
    let saved = image::open(&path).unwrap().to_rgba8();
    assert_eq!(saved.dimensions(), (6, 6));
    assert_eq!(saved.get_pixel(1, 1).0, [255, 0, 0, 255]);
    assert_eq!(saved.get_pixel(4, 1).0[3], 0);
    std::fs::remove_file(path).ok();
}

#[gpui::test]
fn a_question_reports_the_chosen_answer(cx: &mut TestAppContext) {
    let seen = Rc::new(RefCell::new(Vec::new()));
    let record = seen.clone();
    let cx = open(cx, move || {
        let record = record.clone();
        QuestionCard::new("question", "Which?", ["One", "Two"]).on_answer(move |index, _, _| record.borrow_mut().push(index)).into_any_element()
    });
    cx.simulate_keystrokes("tab");
    cx.run_until_parked();
    cx.simulate_keystrokes("right");
    cx.run_until_parked();
    cx.simulate_keystrokes("tab");
    cx.run_until_parked();
    cx.simulate_keystrokes("enter");
    assert_eq!(*seen.borrow(), vec![1]);
}

fn paint_counter(count: Rc<Cell<u32>>) -> impl IntoElement {
    gpui::canvas(|_, _, _| (), move |_, _, _, _| count.set(count.get() + 1)).w(px(50.)).h(px(20.))
}

#[gpui::test]
fn a_sidebar_hides_its_content_when_collapsed(cx: &mut TestAppContext) {
    let count = Rc::new(Cell::new(0));
    let painted = count.clone();
    let cx = open(cx, move || Sidebar::new("side").child(paint_counter(painted.clone())).into_any_element());
    assert!(count.get() > 0);
    let toggle = cx.debug_bounds("sidebar-toggle").expect("drawn").center();
    cx.simulate_click(toggle, Modifiers::default());
    cx.run_until_parked();
    count.set(0);
    cx.update(|window, _| window.refresh());
    cx.run_until_parked();
    assert_eq!(count.get(), 0, "gone once collapsed");
}

#[gpui::test]
fn everything_static_draws_without_trouble(cx: &mut TestAppContext) {
    let cx = open(cx, || {
        div()
            .child(Steps::new([("One", Step::Done), ("Two", Step::Active), ("Three", Step::Waiting)]))
            .child(NotFound::new("404", "Gone."))
            .child(Marquee::new("marquee", ["A", "B"]))
            .child(Compare::new("compare", div().child("A"), div().child("B")))
            .child(ColorPicker::new("color"))
            .child(Dialog::new("sheet", true).sheet().title("Sheet"))
            .into_any_element()
    });
    cx.run_until_parked();
}

#[gpui::test]
fn the_color_picker_reports_keys_and_clicks(cx: &mut TestAppContext) {
    let seen = Rc::new(RefCell::new(Vec::new()));
    let record = seen.clone();
    let cx = open(cx, move || {
        let record = record.clone();
        ColorPicker::new("color").on_change(move |color, _, _| record.borrow_mut().push(color)).into_any_element()
    });
    cx.simulate_keystrokes("tab");
    cx.run_until_parked();
    cx.simulate_keystrokes("right");
    assert_eq!(seen.borrow().len(), 1);
    let field = cx.debug_bounds("color-field").expect("drawn");
    cx.simulate_mouse_down(field.origin + gpui::point(field.size.width - px(2.), px(2.)), gpui::MouseButton::Left, Modifiers::default());
    let saturation = (f32::from(field.size.width) - 2.0) / f32::from(field.size.width);
    let value = 1.0 - 2.0 / f32::from(field.size.height);
    let picked = *seen.borrow().last().unwrap();
    assert!(close(picked, hsv_to_rgb(210.0, saturation, value)), "the click sets it: {picked:?}");
}

struct ComposerHarness {
    composer: Entity<Composer>,
}

impl Render for ComposerHarness {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(self.composer.clone())
    }
}

#[gpui::test]
fn the_composer_sends_with_enter_and_clears(cx: &mut TestAppContext) {
    cx.update(fluidither::init);
    let (view, cx) = cx.add_window_view(|_, cx| ComposerHarness { composer: cx.new(Composer::new) });
    let composer = view.read_with(cx, |harness, _| harness.composer.clone());
    let sent = Rc::new(RefCell::new(Vec::new()));
    let record = sent.clone();
    cx.update(|window, cx| {
        window.activate_window();
        cx.subscribe(&composer, move |_, event: &ComposerEvent, _| record.borrow_mut().push(event.clone())).detach();
        let handle = composer.read(cx).focus_handle(cx);
        window.focus(&handle);
    });
    cx.run_until_parked();
    cx.simulate_input("hello");
    cx.simulate_keystrokes("enter");
    cx.simulate_keystrokes("enter");
    assert_eq!(*sent.borrow(), vec![ComposerEvent::Sent("hello".into())], "the second Enter had nothing to send");
}
