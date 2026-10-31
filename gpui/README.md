# fluidither

![The word fluidither fading out in dots beside a dithered globe](https://raw.githubusercontent.com/vrnrn/fluidither/main/docs/images/hero-light.png)

Dithered, spring-driven UI components for [GPUI](https://crates.io/crates/gpui). Dithers are
drawn on the CPU, and the same components exist for the plain web, with tests holding both
targets to the same cells.

![A GPUI window with a dotted world map, a globe, tabs, a sidebar and a color picker](https://raw.githubusercontent.com/vrnrn/fluidither/main/docs/images/gpui-light.png)

## Install

```toml
[dependencies]
fluidither = "0.1"
gpui = { version = "=0.2.2", features = ["runtime_shaders"] }
```

Building needs no Metal toolchain, because shaders compile at startup. The first build compiles
GPUI itself, which took about 10 minutes on an M1 Pro.

## Use it

```rust
use fluidither::{Button, DitherGradient, Gradient, Look, Tabs, Theme};
use gpui::{div, prelude::*, px, App, Application, Context, Render, Window, WindowOptions};

struct Home;

impl Render for Home {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Theme::get(cx).colors();
        div()
            .size_full()
            .flex()
            .flex_col()
            .items_start()
            .gap(px(12.))
            .p(px(24.))
            .bg(colors.paper)
            .text_color(colors.ink)
            .child(Tabs::new("range", ["Day", "Week", "Month"]))
            .child(DitherGradient::new("band", Gradient::default()).speed(0.4).h(px(120.)).w_full())
            .child(Button::new("save", "Save").look(Look::Soft).on_click(|_, _, _| println!("saved")))
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        fluidither::init(cx);
        cx.open_window(WindowOptions::default(), |_, cx| cx.new(|_| Home)).unwrap();
        cx.activate(true);
    });
}
```

## Support

Tested on macOS on Apple Silicon with Rust 1.98.1. Linux and Windows have not been built.

`gpui` 0.2.2 has no accessibility API, so VoiceOver can't see these components. Don't use this
crate alone for anything that must be accessible.

## Learn more

- [Full README](https://github.com/vrnrn/fluidither#readme)
- [Every component and method](https://github.com/vrnrn/fluidither/blob/main/skills/fluidither/references/gpui.md)
- [What has and hasn't been tested](https://github.com/vrnrn/fluidither/blob/main/docs/support.md)

## License

MIT
