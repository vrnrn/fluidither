//! Buttons, tabs, forms and overlays in a window: `cargo run --release --example gallery`.
//! Set FD_THEME=light|dark to pick a theme. `support/capture.rs` lists the settings
//! that drive it for proofs.

#[path = "support/capture.rs"]
mod capture;

use fluidither::{
    Button, Checkbox, Checked, Combobox, CommandMenu, Density, Dialog, DitherGradient, Gradient,
    Look, Matrix, Menu, Mode, RadioGroup, Row, Select, Shape, Slider, Switch, Tab, Tabs, TextField,
    Theme,
};
use gpui::{
    div, prelude::*, px, size, AnyWindowHandle, App, Application, AssetSource, Bounds, Context,
    Entity, Render, Rgba, SharedString, Window, WindowBounds, WindowOptions,
};
use std::borrow::Cow;

const LIST_ICON: &str = "list.svg";

struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        let file = format!("{}/examples/assets/{path}", env!("CARGO_MANIFEST_DIR"));
        Ok(std::fs::read(file).ok().map(Cow::Owned))
    }

    fn list(&self, _: &str) -> gpui::Result<Vec<SharedString>> {
        Ok(Vec::new())
    }
}

struct Gallery {
    range: usize,
    clicks: u32,
    name: Entity<TextField>,
    email: Entity<TextField>,
    city: Entity<Combobox>,
    commands: Entity<CommandMenu>,
    confirming: bool,
}

fn field(label: &'static str, input: Entity<TextField>, hint: &'static str, muted: Rgba) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .flex_1()
        .gap(px(6.))
        .child(div().font_weight(gpui::FontWeight(560.)).child(label))
        .child(input)
        .child(div().text_color(muted).child(hint))
}

fn heading(title: &'static str, note: &'static str, muted: Rgba) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(px(2.))
        .mt(px(28.))
        .mb(px(12.))
        .child(div().text_size(px(15.)).font_weight(gpui::FontWeight(600.)).child(title))
        .child(div().text_size(px(13.)).text_color(muted).child(note))
}

fn tile(id: &'static str, caption: &'static str, gradient: Gradient, matrix: Matrix, muted: Rgba) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .flex_1()
        .gap(px(6.))
        .child(
            DitherGradient::new(id, gradient)
                .matrix(matrix)
                .corner_radius(8.)
                .h(px(150.))
                .w_full(),
        )
        .child(div().text_size(px(12.)).text_color(muted).child(caption))
}

impl Render for Gallery {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::get(cx);
        let colors = theme.colors();
        let muted = colors.ink_soft;
        let orange = colors.orange;
        let panels = ["Today's numbers.", "This week's numbers.", "This month's numbers.", "", "Everything so far."];
        let panel: SharedString = panels[self.range].into();
        let view = cx.entity();
        let closing = view.clone();
        let close_dialog = std::rc::Rc::new(move |_: &gpui::ClickEvent, _: &mut Window, cx: &mut App| {
            closing.update(cx, |this, cx| {
                this.confirming = false;
                cx.notify();
            });
        });

        div()
            .id("gallery")
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
                    .py(px(32.))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(div().text_size(px(20.)).font_weight(gpui::FontWeight(600.)).child("fluidither"))
                            .child(
                                Button::new("theme", "Switch theme")
                                    .look(Look::Outline)
                                    .density(Density::Compact)
                                    .on_click(|_, window, cx| {
                                        let theme = cx.global_mut::<Theme>();
                                        theme.mode = match theme.mode {
                                            Mode::Light => Mode::Dark,
                                            Mode::Dark => Mode::Light,
                                        };
                                        window.refresh();
                                    }),
                            ),
                    )
                    .child(
                        div().mt(px(24.)).child(
                            DitherGradient::new(
                                "band",
                                Gradient {
                                    shape: Shape::Linear { angle: 90.0 },
                                    from: 0.0,
                                    to: 0.75,
                                    warp: 0.12,
                                },
                            )
                            .speed(0.4)
                            .corner_radius(12.)
                            .h(px(120.))
                            .w_full(),
                        ),
                    )
                    .child(heading("Button", "Four looks and two densities.", muted))
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .gap(px(8.))
                            .child(Button::new("primary", "Solid").on_click(cx.listener(|this, _, _, cx| {
                                this.clicks += 1;
                                cx.notify();
                            })))
                            .child(Button::new("secondary", "Soft").look(Look::Soft))
                            .child(Button::new("tertiary", "Outline").look(Look::Outline))
                            .child(Button::new("ghost", "Ghost").look(Look::Ghost))
                            .child(Button::new("disabled", "Disabled").disabled(true))
                            .child(Button::new("loading", "Loading").busy(true))
                            .child(Button::new("icon", "With icon").look(Look::Outline).icon(LIST_ICON)),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .mt(px(12.))
                            .gap(px(8.))
                            .child(Button::new("solid-c", "Solid").density(Density::Compact))
                            .child(Button::new("soft-c", "Soft").density(Density::Compact).look(Look::Soft))
                            .child(Button::new("outline-c", "Outline").density(Density::Compact).look(Look::Outline))
                            .child(Button::new("ghost-c", "Ghost").density(Density::Compact).look(Look::Ghost)),
                    )
                    .child(heading(
                        "Tabs",
                        "Hover to see a dithered peek at the next tab. Arrow keys, Home and End move the selection.",
                        muted,
                    ))
                    .child(
                        div().flex().child(
                            Tabs::new(
                                "range",
                                [
                                    Tab::new("Day"),
                                    Tab::new("Week"),
                                    Tab::new("Month"),
                                    Tab::new("Year").disabled(true),
                                    Tab::new("All time"),
                                ],
                            )
                            .selected(self.range)
                            .on_change(move |index, _, cx| {
                                view.update(cx, |this, cx| {
                                    this.range = index;
                                    cx.notify();
                                });
                            }),
                        ),
                    )
                    .child(div().mt(px(12.)).child(panel))
                    .child(
                        div().flex().mt(px(16.)).child(
                            Tabs::new("view", [Tab::new("List").icon(LIST_ICON), Tab::new("Grid")])
                                .density(Density::Compact)
                                .flat_peek(),
                        ),
                    )
                    .child(heading("Form controls", "Checks, radios, a switch, a slider and text fields.", muted))
                    .child(
                        div()
                            .flex()
                            .gap(px(16.))
                            .child(field("Name", self.name.clone(), "As it should appear on the invoice.", muted))
                            .child(field("Email", self.email.clone(), "That doesn't look like an address.", muted)),
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(14.))
                            .mt(px(14.))
                            .child(Checkbox::new("checked").label("Checked").initial(true))
                            .child(Checkbox::new("unchecked").label("Unchecked"))
                            .child(Checkbox::new("mixed").label("Mixed").checked(Checked::Mixed))
                            .child(Checkbox::new("off").label("Disabled").disabled(true)),
                    )
                    .child(
                        div()
                            .flex()
                            .mt(px(12.))
                            .child(RadioGroup::new("plan", ["Monthly", "Yearly", "Lifetime"]).disable(2)),
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(14.))
                            .mt(px(12.))
                            .child(Switch::new("sync").label("Sync").initial(true))
                            .child(Switch::new("alerts").label("Alerts")),
                    )
                    .child(div().w(px(320.)).mt(px(8.)).child(Slider::new("volume").initial(40.)))
                    .child(heading("Overlays", "Tooltip, menu, select, combobox, dialog and a command menu.", muted))
                    .child(
                        div()
                            .flex()
                            .gap(px(8.))
                            .child(Button::new("tip", "Hover me").look(Look::Soft).tip("Saves a copy"))
                            .child(Menu::new(
                                "actions",
                                "Actions",
                                [Row::new("Rename"), Row::new("Duplicate"), Row::new("Archive").disabled(true), Row::new("Delete")],
                            ))
                            .child(Select::new("fruit", ["Apple", "Banana", "Pear", "Plum"]).initial(2))
                            .child(div().w(px(220.)).child(self.city.clone())),
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(8.))
                            .mt(px(12.))
                            .child(Button::new("open-dialog", "Open dialog").on_click(cx.listener(|this, _, _, cx| {
                                this.confirming = true;
                                cx.notify();
                            })))
                            .child(Button::new("open-command", "Command menu").look(Look::Soft).on_click(cx.listener(
                                |this, _, _, cx| this.commands.update(cx, |menu, cx| menu.show(cx)),
                            ))),
                    )
                    .child(heading(
                        "Dither gradient",
                        "Cells computed on the CPU and uploaded as an image. Linear, radial and conic.",
                        muted,
                    ))
                    .child(
                        div()
                            .flex()
                            .gap(px(16.))
                            .child(tile(
                                "linear",
                                "Linear, Bayer 4",
                                Gradient { warp: 0.0, ..Default::default() },
                                Matrix::Bayer4,
                                muted,
                            ))
                            .child(tile(
                                "radial",
                                "Radial, Bayer 8",
                                Gradient {
                                    shape: Shape::Radial { center: (0.5, 0.5), radius: 1.0 },
                                    warp: 0.1,
                                    ..Default::default()
                                },
                                Matrix::Bayer8,
                                muted,
                            ))
                            .child(tile(
                                "conic",
                                "Conic, blue noise",
                                Gradient {
                                    shape: Shape::Conic { center: (0.5, 0.5), angle: 0.0 },
                                    warp: 0.1,
                                    ..Default::default()
                                },
                                Matrix::BlueNoise,
                                muted,
                            ))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .flex_1()
                                    .gap(px(6.))
                                    .child(
                                        DitherGradient::new(
                                            "accent",
                                            Gradient {
                                                shape: Shape::Linear { angle: 135.0 },
                                                from: 0.1,
                                                to: 0.9,
                                                warp: 0.12,
                                            },
                                        )
                                        .ink(orange)
                                        .speed(1.0)
                                        .corner_radius(8.)
                                        .h(px(150.))
                                        .w_full(),
                                    )
                                    .child(div().text_size(px(12.)).text_color(muted).child("Animated, your own color")),
                            ),
                    ),
            )
            .child(
                Dialog::new("confirm", self.confirming)
                    .title("Delete this report?")
                    .text("This can't be undone.")
                    .action(Button::new("cancel", "Cancel").look(Look::Soft).on_click({
                        let close = close_dialog.clone();
                        move |event, window, cx| close(event, window, cx)
                    }))
                    .action(Button::new("delete", "Delete").on_click({
                        let close = close_dialog.clone();
                        move |event, window, cx| close(event, window, cx)
                    }))
                    .on_close(move |window, cx| close_dialog(&gpui::ClickEvent::default(), window, cx)),
            )
            .child(self.commands.clone())
    }
}

fn main() {
    let started = std::time::Instant::now();
    Application::new().with_assets(Assets).run(move |cx: &mut App| {
        fluidither::init(cx);
        match std::env::var("FD_THEME").as_deref() {
            Ok("light") => cx.global_mut::<Theme>().mode = Mode::Light,
            Ok("dark") => cx.global_mut::<Theme>().mode = Mode::Dark,
            _ => {}
        }
        let bounds = Bounds::centered(None, size(px(912.), px(1040.)), cx);
        let window = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |window, cx| {
                    cx.new(|cx| Gallery {
                        range: 1,
                        clicks: 0,
                        name: cx.new(|cx| TextField::new(cx).placeholder("Your name")),
                        email: cx.new(|cx| {
                            let mut email = TextField::new(cx).with_text("not an email");
                            email.set_invalid(true, cx);
                            email
                        }),
                        city: cx.new(|cx| {
                            Combobox::new(["Lisbon", "London", "Oslo", "Tokyo"], window, cx).placeholder("Search cities", cx)
                        }),
                        commands: cx.new(|cx| CommandMenu::new(["New report", "Open recent", "Switch theme", "Help"], cx)),
                        confirming: false,
                    })
                },
            )
            .unwrap();
        cx.activate(true);
        eprintln!("window open after {:?}", started.elapsed());

        let handle: AnyWindowHandle = window.into();
        cx.spawn(async move |cx| capture::run_script(handle, cx).await).detach();
    });
}
