//! Structure and feedback components in a window:
//! `cargo run --release --example blocks`. The first gallery is `gallery`.

#[path = "support/capture.rs"]
mod capture;

use fluidither::{
    Accordion, Badge, Banner, Button, Card, Divider, Loader, Look, Mode, Nav, Progress, Row,
    ScrollArea, Skeleton, Sparkline, Stat, Table, Theme, Tone,
};
use gpui::{
    div, prelude::*, px, size, AnyWindowHandle, App, Application, Bounds, Context, Render, Window,
    WindowBounds, WindowOptions,
};

struct Blocks;

fn heading(title: &'static str) -> impl IntoElement {
    div().mt(px(24.)).mb(px(10.)).text_size(px(15.)).font_weight(gpui::FontWeight(600.)).child(title)
}

impl Render for Blocks {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Theme::get(cx).colors();
        div()
            .id("blocks")
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
                    .py(px(16.))
                    .child(heading("Structure"))
                    .child(
                        div()
                            .flex()
                            .gap(px(8.))
                            .child(Badge::new("Draft"))
                            .child(Badge::new("Live").tone(Tone::Green))
                            .child(Badge::new("Review").tone(Tone::Orange))
                            .child(Badge::new("Failed").tone(Tone::Red))
                            .child(Badge::new("Info").tone(Tone::Blue))
                            .child(Badge::new("Beta").tone(Tone::Violet))
                            .child(Badge::new("New").tone(Tone::Pink)),
                    )
                    .child(
                        div().mt(px(12.)).child(
                            Banner::new("Your trial ends in three days.")
                                .tone(Tone::Orange)
                                .action(Button::new("upgrade", "Upgrade").look(Look::Soft).density(fluidither::Density::Compact)),
                        ),
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(16.))
                            .mt(px(16.))
                            .child(div().flex_1().child(Card::new().title("Quiet card").text("Sits on the page with a thin ring.")))
                            .child(div().flex_1().child(Card::new().title("Raised card").text("Lifts off the page for what matters.").raised())),
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(16.))
                            .mt(px(16.))
                            .child(div().flex_1().child(Card::new().child(Nav::new(
                                "side",
                                [Row::new("Overview"), Row::new("Activity"), Row::new("Billing"), Row::new("Settings")],
                            ))))
                            .child(
                                div().flex_1().child(
                                    Accordion::new("faq")
                                        .section("What is dithering?", div().child("Mixing two colors in a fine pattern so they read as a tone."))
                                        .section("Why a spring?", div().child("It keeps its speed when the target moves, so motion never jerks."))
                                        .section("Does it need a framework?", div().child("Only GPUI, on this target."))
                                        .initial(0),
                                ),
                            ),
                    )
                    .child(
                        div().mt(px(16.)).child(
                            Table::new("invoices")
                                .column("Invoice")
                                .column("Client")
                                .number_column("Amount")
                                .row(["INV-1021", "Harbour Coffee", "$1,204.00"])
                                .row(["INV-1022", "Atlas Print", "$98.50"])
                                .row(["INV-1023", "North Mill", "$3,410.75"]),
                        ),
                    )
                    .child(
                        ScrollArea::new("notes").mt(px(16.)).h(px(70.)).children(
                            ["Version 0.4 added overlays.", "Version 0.3 added form controls.", "Version 0.2 added tabs.", "Version 0.1 added the button.", "Before that there was only a dither."]
                                .map(|line| div().pb(px(8.)).text_color(colors.ink_soft).child(line)),
                        ),
                    )
                    .child(heading("Feedback and numbers"))
                    .child(
                        div()
                            .flex()
                            .gap(px(16.))
                            .child(
                                div().flex_1().child(
                                    Card::new()
                                        .child(Stat::new("Revenue", "$48,210").change("Up 12% this month"))
                                        .child(div().mt(px(12.)).child(Sparkline::new("revenue", [4., 6., 5., 8., 7., 11., 9., 13., 12., 16.]))),
                                ),
                            )
                            .child(
                                div().flex_1().child(
                                    Card::new()
                                        .child(div().flex().items_center().gap(px(10.)).child(Loader::new("loader")).child("Loading reports"))
                                        .child(div().my(px(12.)).child(Divider))
                                        .child(Skeleton::new("line-1"))
                                        .child(div().w(px(220.)).mt(px(8.)).child(Skeleton::new("line-2")))
                                        .child(div().my(px(12.)).child(Divider))
                                        .child(Progress::new(0.62)),
                                ),
                            ),
                    ),
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
        let bounds = Bounds::centered(None, size(px(912.), px(960.)), cx);
        let window = cx
            .open_window(
                WindowOptions { window_bounds: Some(WindowBounds::Windowed(bounds)), ..Default::default() },
                |_, cx| cx.new(|_| Blocks),
            )
            .unwrap();
        cx.activate(true);
        let handle: AnyWindowHandle = window.into();
        cx.spawn(async move |cx| capture::run_script(handle, cx).await).detach();
    });
}
