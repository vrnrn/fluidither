//! Obel Mono alongside prose and readouts. Run with `--example typography`.
//! FD_THEME=light|dark and FD_CAPTURE=path.png work on macOS.

#[cfg(target_os = "macos")]
#[path = "support/capture.rs"]
mod capture;

use fluidither::{CopyField, Count, Kbd, Mode, Stat, Table, Theme, FONT_MONO};
use gpui::{
    div, prelude::*, px, size, App, Application, Bounds, Context, FontWeight, Render, Window,
    WindowBounds, WindowOptions,
};

struct Specimen;

impl Render for Specimen {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Theme::get(cx).colors();
        div()
            .size_full()
            .flex()
            .flex_col()
            .gap(px(20.))
            .p(px(32.))
            .bg(colors.paper)
            .text_color(colors.ink)
            .text_size(px(14.))
            .child(
                div()
                    .font_family(FONT_MONO)
                    .text_size(px(40.))
                    .font_weight(FontWeight(560.))
                    .child("fluidither"),
            )
            .child(
                "Type in cells. Code and numbers keep an even rhythm; prose has room to breathe.",
            )
            .child(
                div()
                    .font_family(FONT_MONO)
                    .text_size(px(24.))
                    .line_height(px(32.))
                    .child(
                        div()
                            .font_weight(FontWeight(300.))
                            .child("Il1 O0 rn m 0123456789"),
                    )
                    .child(
                        div()
                            .font_weight(FontWeight(560.))
                            .child("Ink, paper, rhythm."),
                    )
                    .child(div().italic().child("Roman and italic, 100–900.")),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(24.))
                    .child(Stat::new("Readout", "48,210").change("Up 12%"))
                    .child(Kbd::new("⌘ K"))
                    .child(Count::new("count", 1204.)),
            )
            .child(
                Table::new("numbers")
                    .column("Sample")
                    .number_column("Value")
                    .row(["One", "111.00"])
                    .row(["Eight", "888.00"]),
            )
            .child(CopyField::new("command", "npx fluidither add typography"))
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        fluidither::init(cx);
        assert!(
            cx.text_system()
                .all_font_names()
                .iter()
                .any(|name| name == FONT_MONO),
            "bundled font must be registered"
        );
        // Registration alone does not rule out a silent system-font fallback.
        let text = cx.text_system();
        let mut faces = Vec::new();
        for weight in [400., 560., 700.] {
            let mut requested = gpui::font(FONT_MONO);
            requested.weight = FontWeight(weight);
            let id = text.resolve_font(&requested);
            assert!(!faces.contains(&id), "regular, strong and bold must resolve to distinct faces");
            faces.push(id);
            let advance = f32::from(text.advance(id, px(16.), '0').unwrap().width);
            let cap_height = f32::from(text.cap_height(id, px(16.)));
            eprintln!("Obel Mono weight {weight}: advance {advance}, cap height {cap_height}, font {id:?}");
            assert!((advance - 9.6).abs() < 0.01, "Obel Mono has a 0.6em cell");
            assert!((cap_height - 11.336).abs() < 0.01, "Obel Mono has a 1417/2000em cap height");
            requested.style = gpui::FontStyle::Italic;
            let italic = text.resolve_font(&requested);
            assert!(!faces.contains(&italic), "italic must resolve to its own face");
            faces.push(italic);
        }
        let mut semibold = gpui::font(FONT_MONO);
        semibold.weight = FontWeight(600.);
        assert_eq!(text.resolve_font(&semibold), faces[2], "560 selects SemiBold");
        cx.global_mut::<Theme>().reduced_motion = true;
        match std::env::var("FD_THEME").as_deref() {
            Ok("light") => cx.global_mut::<Theme>().mode = Mode::Light,
            Ok("dark") => cx.global_mut::<Theme>().mode = Mode::Dark,
            _ => {}
        }
        let bounds = Bounds::centered(None, size(px(680.), px(570.)), cx);
        let window = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |_, cx| cx.new(|_| Specimen),
            )
            .unwrap();
        cx.activate(true);
        #[cfg(target_os = "macos")]
        cx.spawn(async move |cx| capture::run_script(window.into(), cx).await)
            .detach();
        #[cfg(not(target_os = "macos"))]
        let _ = window;
    });
}
