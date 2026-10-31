//! Static building blocks: Card, Badge and Banner.

use crate::button::{Button, Look};
use crate::generated::{RADIUS_CONTROL, WEIGHT_STRONG};
use crate::paint::checker;
use crate::theme::{cap_offset, fade, Theme};
use gpui::{
    div, prelude::*, px, AnyElement, App, Div, FontWeight, IntoElement, ParentElement, RenderOnce,
    Rgba, SharedString, Window,
};
use std::rc::Rc;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tone {
    #[default]
    Neutral,
    Red,
    Orange,
    Green,
    Blue,
    Violet,
    Pink,
}

impl Tone {
    fn color(self, cx: &App) -> Rgba {
        let colors = Theme::get(cx).colors();
        match self {
            Tone::Neutral => colors.ink_soft,
            Tone::Red => colors.red,
            Tone::Orange => colors.orange,
            Tone::Green => colors.green,
            Tone::Blue => colors.blue,
            Tone::Violet => colors.violet,
            Tone::Pink => colors.pink,
        }
    }
}

/// A surface to group things on. Add content with `.child`.
#[derive(IntoElement)]
pub struct Card {
    title: Option<SharedString>,
    text: Option<SharedString>,
    raised: bool,
    children: Vec<AnyElement>,
}

impl Card {
    pub fn new() -> Self {
        Self { title: None, text: None, raised: false, children: Vec::new() }
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn text(mut self, text: impl Into<SharedString>) -> Self {
        self.text = Some(text.into());
        self
    }

    pub fn raised(mut self) -> Self {
        self.raised = true;
        self
    }
}

impl Default for Card {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for Card {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Card {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let colors = theme.colors();
        let (level, shadow) = if self.raised { (3, 2) } else { (1, 0) };
        div()
            .flex()
            .flex_col()
            .p(px(16.))
            .rounded(px(RADIUS_CONTROL + 2.))
            .bg(colors.surfaces[level])
            .shadow(theme.shadow(shadow))
            .text_color(colors.ink)
            .text_size(px(13.))
            .line_height(px(18.))
            .when_some(self.title, |this, title| {
                this.child(div().mb(px(4.)).text_size(px(14.)).font_weight(FontWeight(WEIGHT_STRONG)).child(title))
            })
            .when_some(self.text, |this, text| this.child(div().text_color(colors.ink_soft).child(text)))
            .children(self.children)
    }
}

/// A short label. A colored cell carries the tone, so the text stays at full contrast.
#[derive(IntoElement)]
pub struct Badge {
    label: SharedString,
    tone: Tone,
}

impl Badge {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self { label: label.into(), tone: Tone::default() }
    }

    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }
}

impl RenderOnce for Badge {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = Theme::get(cx).colors();
        let nudge = cap_offset(window, px(11.));
        div()
            .flex()
            .flex_none()
            .items_center()
            .gap(px(5.))
            .h(px(20.))
            .px(px(7.))
            .rounded(px(4.))
            .bg(colors.soft)
            .text_color(colors.ink)
            .text_size(px(11.))
            .line_height(px(14.))
            .font_weight(FontWeight(WEIGHT_STRONG))
            .child(div().flex_none().size(px(6.)).bg(self.tone.color(cx)))
            .child(div().relative().top(nudge).child(self.label))
    }
}

/// A message across its container, with room for an action on the right.
#[derive(IntoElement)]
pub struct Banner {
    text: SharedString,
    tone: Tone,
    action: Option<AnyElement>,
    on_dismiss: Option<OnDismiss>,
}

type OnDismiss = Rc<dyn Fn(&mut Window, &mut App)>;

impl Banner {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self { text: text.into(), tone: Tone::default(), action: None, on_dismiss: None }
    }

    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }

    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.action = Some(action.into_any_element());
        self
    }

    /// Add a close button. Keep the banner's state in your view and clear it here.
    pub fn on_dismiss(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(handler));
        self
    }
}

fn strip(tone: Rgba) -> Div {
    div()
        .absolute()
        .top_0()
        .bottom_0()
        .left_0()
        .w(px(16.))
        .child(checker(fade(tone, 0.55), 2.0, 0.0))
        .child(div().absolute().top_0().bottom_0().left_0().w(px(3.)).bg(tone))
}

impl RenderOnce for Banner {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = Theme::get(cx).colors();
        let tone = self.tone.color(cx);
        let nudge = cap_offset(window, px(13.));
        div()
            .relative()
            .flex()
            .items_center()
            .justify_between()
            .gap(px(12.))
            .min_h(px(34.))
            .py(px(8.))
            .pl(px(28.))
            .pr(px(12.))
            .overflow_hidden()
            .rounded(px(RADIUS_CONTROL))
            .border_1()
            .border_color(colors.line)
            .bg(colors.surfaces[1])
            .text_color(colors.ink)
            .text_size(px(13.))
            .line_height(px(18.))
            .child(strip(tone))
            .child(div().relative().top(nudge).child(self.text))
            .children(self.action)
            .when_some(self.on_dismiss, |this, handler| {
                this.child(Button::new("fd-banner-dismiss", "Close").look(Look::Ghost).on_click(move |_, window, cx| handler(window, cx)))
            })
    }
}
