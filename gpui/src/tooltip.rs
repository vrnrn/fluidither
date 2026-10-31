//! A short text label for GPUI's built-in tooltips, which show after a hover. There
//! is no way to show one on keyboard focus yet.

use crate::theme::Theme;
use gpui::{div, prelude::*, px, AnyView, App, Context, Render, SharedString, Window};

struct Tip {
    text: SharedString,
}

impl Render for Tip {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Theme::get(cx).colors();
        div()
            .px(px(8.))
            .py(px(4.))
            .rounded(px(crate::generated::RADIUS_PILL))
            .bg(colors.ink)
            .text_color(colors.on_solid)
            .text_size(px(12.))
            .line_height(px(16.))
            .child(self.text.clone())
    }
}

/// Pass this to `.tooltip()` on any element with an id.
pub fn tooltip(text: impl Into<SharedString>) -> impl Fn(&mut Window, &mut App) -> AnyView + 'static {
    let text = text.into();
    move |_, cx| cx.new(|_| Tip { text: text.clone() }).into()
}
