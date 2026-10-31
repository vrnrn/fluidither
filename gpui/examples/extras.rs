//! The map, globe, sidebar and other last parts in a window:
//! `cargo run --release --example extras`.

#[path = "support/capture.rs"]
mod capture;

use fluidither::{
    Badge, Bloom, Button, ColorPicker, Compare, Composer, ComposerEvent, Dialog, Dissolve,
    DitherGradient, FileCard, Gradient, Look, Marquee, Mode, NotFound, QuestionCard, Shape, Sidebar,
    Step, Steps, Tabs, Theme, Tone,
};
use gpui::{
    div, prelude::*, px, size, AnyWindowHandle, App, Application, Bounds, Context, Entity, Render,
    Window, WindowBounds, WindowOptions,
};

struct Extras {
    sheet: bool,
    composer: Entity<Composer>,
    sent: Vec<String>,
}

fn heading(title: &'static str) -> impl IntoElement {
    div().mt(px(20.)).mb(px(10.)).text_size(px(15.)).font_weight(gpui::FontWeight(600.)).child(title)
}

fn band(id: &'static str, from: f64, to: f64) -> impl IntoElement {
    let gradient = Gradient { shape: Shape::Linear { angle: 90.0 }, from, to, warp: 0.0 };
    DitherGradient::new(id, gradient).size_full()
}

impl Render for Extras {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Theme::get(cx).colors();
        let view = cx.entity();
        let opener = view.clone();
        div()
            .id("extras")
            .size_full()
            .overflow_y_scroll()
            .bg(colors.paper)
            .text_color(colors.ink)
            .text_size(px(13.))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .w(px(848.))
                    .mx_auto()
                    .pb(px(20.))
                    .child(heading("Map and globe"))
                    .child(
                        div()
                            .flex()
                            .gap(px(16.))
                            .child(DitherGradient::map("map").w(px(440.)).h(px(220.)))
                            .child(DitherGradient::globe("globe").w(px(220.)).h(px(220.))),
                    )
                    .child(heading("Tabs and marquee"))
                    .child(Tabs::new("underline", ["Overview", "Activity", "Billing", "Members", "Settings"]).underline())
                    .child(div().mt(px(12.)).child(Marquee::new("marquee", ["Rendered on the CPU", "Same cells on both targets", "No dependencies", "Honest about what is untested"])))
                    .child(heading("Layout and asking"))
                    .child(
                        div()
                            .flex()
                            .gap(px(16.))
                            .items_start()
                            .child(
                                Sidebar::new("sidebar").width(180.).child(
                                    div().flex().flex_col().gap(px(6.)).p(px(10.)).child("Inbox").child("Drafts").child("Archive"),
                                ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(14.))
                                    .w(px(220.))
                                    .child(Steps::new([("Read the file", Step::Done), ("Make the change", Step::Active), ("Run the checks", Step::Waiting)]))
                                    .child(FileCard::new("report.pdf", "2.4 MB")),
                            )
                            .child(div().flex_1().child(QuestionCard::new("question", "Which size?", ["Regular", "Compact"]))),
                    )
                    .child(heading("Comparing and holding"))
                    .child(
                        div()
                            .flex()
                            .gap(px(16.))
                            .child(div().flex_1().child(Compare::new("compare", band("before", 0.0, 0.3), band("after", 0.5, 1.0)).height(120.)))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .flex_1()
                                    .items_start()
                                    .gap(px(12.))
                                    .child(Dissolve::new("held", div().text_size(px(40.)).line_height(px(44.)).font_weight(gpui::FontWeight(700.)).child("Dithered")).hold(0.0, 0.8))
                                    .child(Bloom::new(Tone::Green, Badge::new("Live").tone(Tone::Green))),
                            ),
                    )
                    .child(heading("Writing and color"))
                    .child(
                        div()
                            .flex()
                            .gap(px(16.))
                            .items_start()
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .flex_1()
                                    .gap(px(8.))
                                    .child(self.composer.clone())
                                    .children(self.sent.iter().cloned().map(|text| div().text_color(colors.ink_soft).child(text)))
                                    .child(div().flex().mt(px(8.)).child(Button::new("open-sheet", "Open sheet").look(Look::Outline).on_click(move |_, _, cx| {
                                        opener.update(cx, |this, cx| {
                                            this.sheet = true;
                                            cx.notify();
                                        });
                                    })))
                                    .child(NotFound::new("404", "This page isn't here.")),
                            )
                            .child(ColorPicker::new("color")),
                    ),
            )
            .child(
                Dialog::new("sheet", self.sheet).sheet().title("Sheet").text("A panel against the bottom edge.").on_close(move |_, cx| {
                    view.update(cx, |this, cx| {
                        this.sheet = false;
                        cx.notify();
                    });
                }),
            )
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        fluidither::init(cx);
        match std::env::var("FD_THEME").as_deref() {
            Ok("light") => cx.global_mut::<Theme>().mode = Mode::Light,
            Ok("dark") => cx.global_mut::<Theme>().mode = Mode::Dark,
            _ => {}
        }
        let bounds = Bounds::centered(None, size(px(912.), px(1040.)), cx);
        let window = cx
            .open_window(WindowOptions { window_bounds: Some(WindowBounds::Windowed(bounds)), ..Default::default() }, |_, cx| {
                cx.new(|cx: &mut Context<Extras>| {
                    let composer = cx.new(Composer::new);
                    cx.subscribe(&composer, |this, _, event: &ComposerEvent, cx| {
                        let ComposerEvent::Sent(text) = event;
                        this.sent.push(text.clone());
                        cx.notify();
                    })
                    .detach();
                    Extras { sheet: false, composer, sent: Vec::new() }
                })
            })
            .unwrap();
        cx.activate(true);
        let handle: AnyWindowHandle = window.into();
        cx.spawn(async move |cx| capture::run_script(handle, cx).await).detach();
    });
}
