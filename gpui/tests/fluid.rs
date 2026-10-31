//! The motion-led parts on GPUI's test platform: real layout, focus and input dispatch, but no
//! GPU, so nothing here proves how they look.

use fluidither::fluid::joins;
use fluidither::{Button, CheckGroup, Composer, ComposerEvent, HoverList, InputGroup, Questions, Reveal, Shimmer, Step, TextField, ThinkingSteps};
use gpui::{div, prelude::*, AnyElement, Context, Render, TestAppContext, VisualTestContext, Window};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

struct Harness {
    build: Rc<dyn Fn(&mut Context<Harness>) -> AnyElement>,
}

impl Render for Harness {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().flex().flex_col().items_start().child((self.build.clone())(cx))
    }
}

fn open<'a>(cx: &'a mut TestAppContext, build: impl Fn(&mut Context<Harness>) -> AnyElement + 'static) -> &'a mut VisualTestContext {
    cx.update(fluidither::init);
    let build: Rc<dyn Fn(&mut Context<Harness>) -> AnyElement> = Rc::new(build);
    let (_, cx) = cx.add_window_view(move |_, _| Harness { build });
    cx.update(|window, _| window.activate_window());
    cx.run_until_parked();
    cx
}

#[test]
fn checked_neighbours_join() {
    assert_eq!(joins(&[true, true, false, true]), vec![(false, true), (true, false), (false, false), (false, false)]);
    assert_eq!(joins(&[true, true, true]), vec![(false, true), (true, true), (true, false)]);
}

#[gpui::test]
fn the_new_parts_lay_out(cx: &mut TestAppContext) {
    let cx = open(cx, |cx| {
        let field = cx.new(|cx| TextField::new(cx).placeholder("Name"));
        div()
            .child(HoverList::new("rows", 28.).row("One").row("Two"))
            .child(CheckGroup::new("fruit", ["Apple", "Pear"]).initial([0, 1]))
            .child(ThinkingSteps::new("run", "Research").step("Read", Step::Done).step("Write", Step::Active).source("docs"))
            .child(InputGroup::new().field("Name", field))
            .child(Reveal::new("rise", "Hello").delay(Duration::from_millis(10)))
            .child(Shimmer::new("thinking", ["Thinking", "Reading"]))
            .child(div().debug_selector(|| "after".into()).child(Button::new("after", "After")))
            .into_any_element()
    });
    assert!(cx.debug_bounds("after").is_some());
}

#[gpui::test]
fn number_keys_answer_and_the_flow_finishes(cx: &mut TestAppContext) {
    let answers: Rc<RefCell<Vec<Vec<usize>>>> = Rc::default();
    let seen = answers.clone();
    let cx = open(cx, move |_| {
        let seen = seen.clone();
        Questions::new("survey")
            .question("One?", ["A", "B"])
            .several("Two?", ["C", "D"])
            .on_done(move |all, _, _| *seen.borrow_mut() = all.to_vec())
            .into_any_element()
    });
    cx.simulate_keystrokes("tab");
    cx.run_until_parked();
    cx.simulate_keystrokes("2");
    cx.run_until_parked();
    cx.simulate_keystrokes("2 1");
    cx.run_until_parked();
    assert!(answers.borrow().is_empty(), "several waits for Next");
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();
    assert_eq!(*answers.borrow(), vec![vec![1], vec![0, 1]]);
}

#[gpui::test]
fn tab_takes_the_suggestion_into_an_empty_box(cx: &mut TestAppContext) {
    cx.update(fluidither::init);
    let (composer, cx) = cx.add_window_view(|_, cx| Composer::new(cx).suggestion("Why so stiff?", cx));
    let sent: Rc<RefCell<Vec<String>>> = Rc::default();
    let log = sent.clone();
    cx.update(|_, cx| {
        cx.subscribe(&composer, move |_, event: &ComposerEvent, _| {
            let ComposerEvent::Sent(text) = event;
            log.borrow_mut().push(text.clone());
        })
        .detach();
    });
    cx.update(|window, cx| {
        window.activate_window();
        window.focus(&gpui::Focusable::focus_handle(composer.read(cx), cx));
    });
    cx.run_until_parked();
    cx.simulate_keystrokes("tab enter");
    cx.run_until_parked();
    assert_eq!(*sent.borrow(), vec!["Why so stiff?".to_string()]);
}
