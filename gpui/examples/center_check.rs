//! Checks that labels sit in the middle of their boxes, by measuring pixels in a
//! real window. GPUI's test platform draws nothing, so this cannot be a cargo test:
//! `npm run check:centering`. Exits non-zero if a label is off by more than half a
//! point. Letters are symmetric and have no descenders, so only the layout shows.

#[path = "support/capture.rs"]
mod capture;

use fluidither::generated::{SIZE_COMPACT, SIZE_REGULAR};
use fluidither::{Button, Density, Look, Mode, Tabs, Theme};
use gpui::{
    div, prelude::*, px, size, App, Application, Bounds, Context, Pixels, Render, Timer, Window,
    WindowBounds, WindowOptions,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

const LIMIT: f32 = 0.5;

type Boxes = Rc<RefCell<Vec<Bounds<Pixels>>>>;

struct Check {
    boxes: Boxes,
}

impl Render for Check {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Theme::get(cx).colors();
        let boxes = self.boxes.clone();
        div().size_full().bg(colors.paper).text_color(colors.ink).child(
            div()
                .flex()
                .items_start()
                .gap(px(24.))
                .p(px(24.))
                .on_children_prepainted(move |bounds, _, _| *boxes.borrow_mut() = bounds)
                .child(Tabs::new("selected", ["HEH"]))
                .child(Tabs::new("plain", ["HEH", "HEH"]))
                .child(Tabs::new("compact", ["HEH"]).density(Density::Compact))
                .child(Tabs::new("compact-plain", ["HEH", "HEH"]).density(Density::Compact))
                .child(Button::new("solid", "HEH"))
                .child(Button::new("soft", "HEH").look(Look::Soft))
                .child(Button::new("compact-button", "HEH").density(Density::Compact)),
        )
    }
}

struct Region {
    name: &'static str,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
}

/// How far ink sits from the region's center, in image pixels.
fn offset(image: &(usize, usize, Vec<u8>, usize), region: &Region) -> (f32, f32) {
    let (image_width, _, pixels, _) = image;
    let at = |x: usize, y: usize| (y * image_width + x) * 4;
    let reference = at(region.x, region.y + region.height / 2);
    let (mut left, mut right, mut top, mut bottom) = (usize::MAX, 0, usize::MAX, 0);
    for y in region.y..region.y + region.height {
        for x in region.x..region.x + region.width {
            let here = at(x, y);
            let distance: i32 = (0..3)
                .map(|channel| (i32::from(pixels[here + channel]) - i32::from(pixels[reference + channel])).abs())
                .sum();
            if distance > 60 {
                left = left.min(x);
                right = right.max(x);
                top = top.min(y);
                bottom = bottom.max(y);
            }
        }
    }
    let gap = |before: usize, after: usize| (before as f32 - after as f32) / 2.0;
    (
        gap(left - region.x, region.x + region.width - 1 - right),
        gap(top - region.y, region.y + region.height - 1 - bottom),
    )
}

fn regions(boxes: &[Bounds<Pixels>], scale: f32, title_bar: usize) -> Vec<Region> {
    let names = ["selected tab", "plain tab", "compact tab", "compact plain tab", "solid button", "soft button", "compact button"];
    // Stay clear of the control's rounded edges and the tab row's padding.
    let inset = 5.0;
    names
        .iter()
        .zip(boxes)
        .enumerate()
        .map(|(index, (name, bounds))| {
            let mut width = f32::from(bounds.size.width);
            let mut left = f32::from(bounds.origin.x);
            // Two equal tabs: the second is unselected. Measure just that tab,
            // leaving out the padding around the row.
            if matches!(index, 1 | 3) {
                let padding = if index == 1 { SIZE_REGULAR.inset } else { SIZE_COMPACT.inset };
                width = (width - padding * 2.0) / 2.0;
                left += padding + width;
            }
            Region {
                name,
                x: ((left + inset) * scale) as usize,
                y: ((f32::from(bounds.origin.y) + inset) * scale) as usize + title_bar,
                width: ((width - inset * 2.0) * scale) as usize,
                height: ((f32::from(bounds.size.height) - inset * 2.0) * scale) as usize,
            }
        })
        .collect()
}

fn main() {
    Application::new().run(|cx: &mut App| {
        fluidither::init(cx);
        cx.global_mut::<Theme>().mode = Mode::Light;
        let boxes: Boxes = Rc::default();
        let bounds = Bounds::centered(None, size(px(760.), px(120.)), cx);
        let view_boxes = boxes.clone();
        let window = cx
            .open_window(
                WindowOptions { window_bounds: Some(WindowBounds::Windowed(bounds)), ..Default::default() },
                move |_, cx| cx.new(|_| Check { boxes: view_boxes }),
            )
            .unwrap();
        cx.activate(true);

        cx.spawn(async move |cx| {
            let mut worst = 0.0f32;
            for mode in [Mode::Light, Mode::Dark] {
                window
                    .update(cx, |_, window, cx| {
                        cx.global_mut::<Theme>().mode = mode;
                        window.refresh();
                    })
                    .ok();
                Timer::after(Duration::from_millis(500)).await;
                let image = capture::window_pixels().expect("capture failed");
                let scale = window.update(cx, |_, window, _| window.scale_factor()).unwrap();
                for region in regions(&boxes.borrow(), scale, image.3) {
                    let (x, y) = offset(&image, &region);
                    let (x, y) = (x / scale, y / scale);
                    println!("{mode:?} {:<20} x {x:+.2}pt  y {y:+.2}pt", region.name);
                    worst = worst.max(x.abs()).max(y.abs());
                }
            }
            println!("worst {worst:.2}pt, limit {LIMIT}pt");
            std::process::exit(if worst <= LIMIT { 0 } else { 1 });
        })
        .detach();
    });
}
