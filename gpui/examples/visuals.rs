//! Charts, fields, text effects, an image and the specialised pieces in a window:
//! `cargo run --release --example visuals`.

#[path = "support/capture.rs"]
mod capture;

use fluidither::{
    Avatar, Button, Calendar, Chart, ChartKind, CopyField, Count, Date, DatePicker, Dialog, Dissolve,
    DitherGradient, DitherImage, Dots, EmptyState, Field, Look, Message, Mode, Scramble, Status,
    Swatches, Theme, Thinking, Uptime,
};
use gpui::{
    div, prelude::*, px, size, AnyWindowHandle, App, Application, Bounds, Context, Render, Rgba,
    Window, WindowBounds, WindowOptions,
};

struct Visuals {
    drawer: bool,
    picture: std::path::PathBuf,
}

fn heading(title: &'static str) -> impl IntoElement {
    div().mt(px(20.)).mb(px(10.)).text_size(px(15.)).font_weight(gpui::FontWeight(600.)).child(title)
}

fn chart(id: &'static str, kind: ChartKind, series: Vec<Vec<f64>>, names: &[&'static str]) -> impl IntoElement {
    div().flex_1().min_w_0().child(Chart::new(id, kind, series).names(names.iter().copied()).height(120.))
}

fn field(name: &'static str, kind: Field, muted: Rgba) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .flex_1()
        .gap(px(4.))
        .child(DitherGradient::field(name, kind).corner_radius(6.).h(px(70.)).w_full())
        .child(div().text_size(px(12.)).text_color(muted).child(name))
}

impl Render for Visuals {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Theme::get(cx).colors();
        let muted = colors.ink_soft;
        let view = cx.entity();
        let opener = view.clone();
        let picture = self.picture.clone();
        div()
            .id("visuals")
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
                    .child(heading("Charts"))
                    .child(
                        div()
                            .flex()
                            .gap(px(16.))
                            .child(chart("bar", ChartKind::Bar, vec![vec![3., 7., 2., 9., 5.], vec![5., 1., 6., 4., 7.]], &["Sales", "Costs"]))
                            .child(chart("area", ChartKind::Area, vec![vec![2., 6., 3., 8., 5., 9.], vec![1., 2., 2., 4., 3., 5.]], &["Visits", "Signups"]))
                            .child(chart("line", ChartKind::Line, vec![vec![2., 6., 3., 8., 5.], vec![7., 1., 6., 2., 4.]], &["This year", "Last year"]))
                            .child(chart("donut", ChartKind::Donut, vec![vec![5., 3., 2., 1.]], &["Search", "Direct", "Social", "Email"])),
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(16.))
                            .mt(px(14.))
                            .child(chart("radar", ChartKind::Radar, vec![vec![5., 8., 3., 7., 6.], vec![3., 4., 6., 2., 5.]], &["Plan", "Actual"]))
                            .child(chart("heatmap", ChartKind::Heatmap, vec![vec![1., 4., 2., 6.], vec![7., 3., 9., 2.], vec![2., 8., 5., 4.]], &[]))
                            .child(chart("gauge", ChartKind::Gauge, vec![vec![62., 100.]], &[])),
                    )
                    .child(heading("Dither fields"))
                    .child(
                        div()
                            .flex()
                            .gap(px(10.))
                            .child(field("Noise", Field::Noise, muted))
                            .child(field("Waves", Field::Waves, muted))
                            .child(field("Plasma", Field::Plasma, muted))
                            .child(field("Ripples", Field::Ripples, muted))
                            .child(field("Tunnel", Field::Tunnel, muted)),
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(10.))
                            .mt(px(8.))
                            .child(field("Metaballs", Field::Metaballs, muted))
                            .child(field("Orb", Field::Orb, muted))
                            .child(field("Stars", Field::Stars, muted))
                            .child(field("Aurora", Field::Aurora, muted))
                            .child(field("Contours", Field::Contours, muted)),
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(10.))
                            .mt(px(8.))
                            .child(field("Beams", Field::Beams, muted))
                            .child(field("Eclipse", Field::Eclipse, muted))
                            .child(field("Horizon", Field::Horizon, muted))
                            .child(field("Mesh", Field::Mesh, muted)),
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(10.))
                            .mt(px(8.))
                            .child(field("Grid", Field::Grid, muted))
                            .child(field("Ridges", Field::Ridges, muted))
                            .child(field("Silk", Field::Silk, muted))
                            .child(field("Smoke", Field::Smoke, muted)),
                    )
                    .child(heading("Text, image and specialised"))
                    .child(
                        div()
                            .flex()
                            .gap(px(20.))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(8.))
                                    .w(px(250.))
                                    .text_size(px(16.))
                                    .child(Scramble::new("scramble", "Scrambled into place"))
                                    .child(Dissolve::new("dissolve", div().child("Dissolved into view")))
                                    .child(div().flex().gap(px(6.)).child(Count::new("count", 1204.)).child("reports"))
                                    .child(DitherImage::new("portrait", picture).size(240., 120.)),
                            )
                            .child(Calendar::new("calendar", Date::new(2026, 10, 14)).value(Date::new(2026, 10, 14)))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .flex_1()
                                    .gap(px(10.))
                                    .child(DatePicker::new("due", Date::new(2026, 10, 30)).initial(Date::new(2026, 10, 30)))
                                    .child(Swatches::new("accent", [colors.red, colors.orange, colors.green, colors.blue, colors.violet]).initial(3))
                                    .child(CopyField::new("copy", "npm install fluidither"))
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap(px(12.))
                                            .child(Avatar::new("AL"))
                                            .child(Button::new("open-drawer", "Open drawer").look(Look::Outline).on_click(move |_, _, cx| {
                                                opener.update(cx, |this, cx| {
                                                    this.drawer = true;
                                                    cx.notify();
                                                });
                                            }))
                                            .child(Dots::new("slides", 3, 1)),
                                    )
                                    .child(Uptime::new([Status::Up, Status::Up, Status::Up, Status::Partial, Status::Up, Status::Up, Status::Down, Status::Up, Status::Up, Status::Up])),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(8.))
                            .mt(px(14.))
                            .child(Message::new("Can you draft the report?"))
                            .child(Message::new("Sure, starting now.").mine())
                            .child(Thinking::new("thinking", "Thinking")),
                    )
                    .child(EmptyState::new("No reports yet", "New reports will appear here.")),
            )
            .child(
                Dialog::new("drawer", self.drawer).drawer().title("Filters").text("A panel that slides in from the edge.").on_close(move |_, cx| {
                    view.update(cx, |this, cx| {
                        this.drawer = false;
                        cx.notify();
                    });
                }),
            )
    }
}

// A soft dark circle on a light gradient, so the example needs no image file.
fn write_picture() -> std::path::PathBuf {
    let path = std::env::temp_dir().join("fluidither-portrait.png");
    let picture = image::GrayImage::from_fn(240, 120, |x, y| {
        let distance = ((x as f32 - 120.).powi(2) + (y as f32 - 60.).powi(2)).sqrt();
        let backdrop = 255.0 - x as f32 * 0.5;
        image::Luma([if distance < 48. { (distance * 3.) as u8 } else { backdrop as u8 }])
    });
    picture.save(&path).expect("the temp folder is writable");
    path
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
        let picture = write_picture();
        let window = cx
            .open_window(
                WindowOptions { window_bounds: Some(WindowBounds::Windowed(bounds)), ..Default::default() },
                move |_, cx| cx.new(|_| Visuals { drawer: false, picture }),
            )
            .unwrap();
        cx.activate(true);
        let handle: AnyWindowHandle = window.into();
        cx.spawn(async move |cx| capture::run_script(handle, cx).await).detach();
    });
}
