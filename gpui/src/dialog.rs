//! A modal dialog. You own whether it is open and close it from `on_close`. While
//! open it takes focus, keeps Tab inside, and hands focus back when it closes. The
//! page behind is screened by a dither.

use crate::dither::{Gradient, Matrix, Shape};
use crate::dither_gradient::DitherGradient;
use crate::generated::{RADIUS_CONTROL, WEIGHT_STRONG};
use crate::theme::Theme;
use gpui::{
    anchored, deferred, div, point, prelude::*, px, AnyElement, App, ElementId, FocusHandle,
    FontWeight, IntoElement, KeyDownEvent, MouseButton, Point, RenderOnce, SharedString,
    Window,
};
use std::rc::Rc;

type OnClose = Rc<dyn Fn(&mut Window, &mut App)>;

const SCREEN_TONE: f64 = 0.5;
const SCREEN_CELL: f32 = 2.0;

#[derive(IntoElement)]
pub struct Dialog {
    id: ElementId,
    open: bool,
    title: SharedString,
    text: Option<SharedString>,
    body: Option<AnyElement>,
    actions: Vec<AnyElement>,
    start_focus: Option<FocusHandle>,
    on_close: Option<OnClose>,
    place: Place,
}

/// Where the panel sits.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Place {
    #[default]
    Center,
    Right,
    Bottom,
}

impl Dialog {
    pub fn new(id: impl Into<ElementId>, open: bool) -> Self {
        Self {
            id: id.into(),
            open,
            title: SharedString::default(),
            text: None,
            body: None,
            actions: Vec::new(),
            start_focus: None,
            on_close: None,
            place: Place::default(),
        }
    }

    /// A full-height panel against the right edge, in place of a centered box.
    pub fn drawer(mut self) -> Self {
        self.place = Place::Right;
        self
    }

    /// A full-width panel against the bottom edge.
    pub fn sheet(mut self) -> Self {
        self.place = Place::Bottom;
        self
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = title.into();
        self
    }

    pub fn text(mut self, text: impl Into<SharedString>) -> Self {
        self.text = Some(text.into());
        self
    }

    pub fn body(mut self, body: impl IntoElement) -> Self {
        self.body = Some(body.into_any_element());
        self
    }

    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.actions.push(action.into_any_element());
        self
    }

    /// Where focus lands on opening. Defaults to the dialog itself.
    pub fn start_focus(mut self, handle: FocusHandle) -> Self {
        self.start_focus = Some(handle);
        self
    }

    pub fn on_close(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(Rc::new(handler));
        self
    }
}

struct DialogState {
    focus: FocusHandle,
    restore: Option<FocusHandle>,
    was_open: bool,
}

impl RenderOnce for Dialog {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = Theme::get(cx).colors();
        let panel_shadow = Theme::get(cx).shadow(3);
        let state = window.use_keyed_state(self.id.clone(), cx, |_, cx| DialogState {
            focus: cx.focus_handle(),
            restore: None,
            was_open: false,
        });
        let (focus, changed) = state.update(cx, |state, _| {
            let changed = state.was_open != self.open;
            state.was_open = self.open;
            (state.focus.clone(), changed)
        });
        if changed {
            let opening = self.open;
            let start = self.start_focus.clone().unwrap_or_else(|| focus.clone());
            let modal = focus.clone();
            let state = state.clone();
            window.defer(cx, move |window, cx| {
                if opening {
                    let previous = window.focused(cx);
                    state.update(cx, |state, _| state.restore = previous);
                    cx.global_mut::<Theme>().modal = Some(modal);
                    window.focus(&start);
                } else {
                    cx.global_mut::<Theme>().modal = None;
                    if let Some(previous) = state.update(cx, |state, _| state.restore.take()) {
                        window.focus(&previous);
                    }
                }
            });
        }
        if !self.open {
            return div().into_any_element();
        }

        let viewport = window.viewport_size();
        let place = self.place;
        let on_escape = self.on_close.clone();
        let on_backdrop = self.on_close;
        let flat = Gradient { shape: Shape::Linear { angle: 180.0 }, from: SCREEN_TONE, to: SCREEN_TONE, warp: 0.0 };

        let panel = div()
            .id("fd-dialog-panel")
            .track_focus(&focus)
            .occlude()
            .relative()
            .flex()
            .flex_col()
            .map(|this| match place {
                Place::Center => this.w(px(440.)).rounded(px(RADIUS_CONTROL + 4.)),
                Place::Right => this.w(px(360.)).h_full(),
                Place::Bottom => this.w_full().rounded_t(px(RADIUS_CONTROL + 4.)),
            })
            .p(px(20.))
            .bg(colors.surfaces[3])
            .shadow(panel_shadow)
            .text_color(colors.ink)
            .text_size(px(13.))
            .line_height(px(18.))
            .on_key_down(move |event: &KeyDownEvent, window, cx| {
                if event.keystroke.key == "escape" {
                    if let Some(close) = &on_escape {
                        close(window, cx);
                    }
                    cx.stop_propagation();
                }
            })
            .child(div().mb(px(6.)).text_size(px(15.)).line_height(px(20.)).font_weight(FontWeight(WEIGHT_STRONG)).child(self.title))
            .when_some(self.text, |this, text| this.child(div().text_color(colors.ink_soft).child(text)))
            .when_some(self.body, |this, body| this.child(body))
            .when(!self.actions.is_empty(), |this| {
                this.child(div().flex().justify_end().gap(px(6.)).mt(px(16.)).children(self.actions))
            });

        deferred(
            anchored().position(Point::default()).offset(point(px(0.), px(0.))).child(
                div()
                    .id(self.id)
                    .relative()
                    .flex()
                    .map(|this| match place {
                        Place::Center => this.items_center().justify_center(),
                        Place::Right => this.items_center().justify_end(),
                        Place::Bottom => this.items_end().justify_center(),
                    })
                    .w(viewport.width)
                    .h(viewport.height)
                    .occlude()
                    .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                        if let Some(close) = &on_backdrop {
                            close(window, cx);
                        }
                    })
                    .child(
                        div().absolute().size_full().child(
                            DitherGradient::new("fd-dialog-screen", flat)
                                .matrix(Matrix::Bayer2)
                                .cell(SCREEN_CELL)
                                .ink(colors.screen)
                                .size_full(),
                        ),
                    )
                    .child(panel),
            ),
        )
        .with_priority(2)
        .into_any_element()
    }
}
