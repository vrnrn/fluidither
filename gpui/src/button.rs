//! A button in four looks and two densities. Pressing nudges it down a pixel, and
//! the quiet looks answer the pointer with a fine dither.

use crate::generated::{
    ThemeColors, DITHER_HOVER_CELL, DITHER_HOVER_INK, GLIDE, QUICK, RADIUS_CONTROL, WEIGHT_STRONG,
};
use crate::paint::checker;
use crate::spring::{Spring, LONGEST_STEP};
use crate::theme::{cap_offset, fade, mix, Density, Theme};
use gpui::{
    div, prelude::*, px, App, ClickEvent, Context, ElementId, FocusHandle, FontWeight, IntoElement,
    KeyDownEvent, MouseButton, RenderOnce, Rgba, SharedString, Task, Timer, Window,
};
use std::rc::Rc;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Look {
    #[default]
    Solid,
    Soft,
    Outline,
    Ghost,
}

type OnClick = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    label: SharedString,
    icon: Option<SharedString>,
    look: Look,
    density: Density,
    disabled: bool,
    busy: bool,
    tip: Option<SharedString>,
    on_click: Option<OnClick>,
}

impl Button {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            look: Look::default(),
            density: Density::default(),
            disabled: false,
            busy: false,
            tip: None,
            on_click: None,
        }
    }

    pub fn look(mut self, look: Look) -> Self {
        self.look = look;
        self
    }

    pub fn density(mut self, density: Density) -> Self {
        self.density = density;
        self
    }

    /// An SVG from the app's assets, drawn before the label in the label's color.
    pub fn icon(mut self, path: impl Into<SharedString>) -> Self {
        self.icon = Some(path.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Keeps the label's width, counts three cells and blocks clicks.
    pub fn busy(mut self, busy: bool) -> Self {
        self.busy = busy;
        self
    }

    /// A hover tooltip.
    pub fn tip(mut self, text: impl Into<SharedString>) -> Self {
        self.tip = Some(text.into());
        self
    }

    pub fn on_click(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }
}

struct ButtonState {
    focus: FocusHandle,
    pointer_over: bool,
    held: bool,
    hover: Spring,
    press: Spring,
    last_frame: Instant,
    /// Cells showing while busy, 1 to 3. Stepped by a slow timer rather than an
    /// animation, which would redraw the whole window on every display refresh.
    count: usize,
    counter: Option<Task<()>>,
}

const COUNT_STEP: Duration = Duration::from_millis(300);

impl ButtonState {
    fn keep_counting(&mut self, counting: bool, cx: &mut Context<Self>) {
        if !counting {
            self.counter = None;
        } else if self.counter.is_none() {
            self.counter = Some(cx.spawn(async move |this, cx| {
                loop {
                    Timer::after(COUNT_STEP).await;
                    let stepped = this.update(cx, |this, cx| {
                        this.count = this.count % 3 + 1;
                        cx.notify();
                    });
                    if stepped.is_err() {
                        break;
                    }
                }
            }));
        }
    }

    fn advance(&mut self, active: bool, reduced: bool) -> bool {
        self.hover.target = f64::from(u8::from(self.pointer_over && active));
        self.press.target = f64::from(u8::from(self.held && active));
        if reduced {
            self.hover.jump(self.hover.target);
            self.press.jump(self.press.target);
        }
        let dt = self.last_frame.elapsed().as_secs_f64().min(LONGEST_STEP);
        self.last_frame = Instant::now();
        self.hover.step(dt) & self.press.step(dt)
    }
}

struct Palette {
    rest: Rgba,
    hover: Rgba,
    press: Rgba,
    text: Rgba,
    text_hover: Rgba,
    edge: Option<Rgba>,
}

fn palette(look: Look, colors: &ThemeColors) -> Palette {
    let clear = fade(colors.ink, 0.0);
    match look {
        Look::Solid => Palette {
            rest: colors.ink,
            hover: colors.solid_hover,
            press: colors.solid_press,
            text: colors.on_solid,
            text_hover: colors.on_solid,
            edge: None,
        },
        Look::Soft => Palette {
            rest: colors.soft,
            hover: colors.soft_hover,
            press: colors.press,
            text: colors.ink,
            text_hover: colors.ink,
            edge: None,
        },
        Look::Outline => Palette {
            rest: clear,
            hover: clear,
            press: colors.press,
            text: colors.ink,
            text_hover: colors.ink,
            edge: Some(colors.line),
        },
        Look::Ghost => Palette {
            rest: clear,
            hover: clear,
            press: colors.press,
            text: colors.ink_soft,
            text_hover: colors.ink,
            edge: None,
        },
    }
}

impl RenderOnce for Button {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let colors = theme.colors();
        let (reduced, keyboard) = (theme.reduced_motion, theme.keyboard_input);
        let tokens = self.density.tokens();
        let shades = palette(self.look, colors);
        let active = !self.disabled && !self.busy;

        let state = window.use_keyed_state(self.id.clone(), cx, |_, cx| ButtonState {
            focus: cx.focus_handle().tab_stop(true),
            pointer_over: false,
            held: false,
            hover: Spring::new(QUICK, 0.0),
            press: Spring::new(GLIDE, 0.0),
            last_frame: Instant::now(),
            count: 3,
            counter: None,
        });
        let busy = self.busy;
        let (hover, press, focus, count, settled) = state.update(cx, |state, cx| {
            state.keep_counting(busy && !reduced, cx);
            let settled = state.advance(active, reduced);
            let (hover, press) = (state.hover.position as f32, state.press.position as f32);
            (hover, press, state.focus.clone(), state.count, settled)
        });
        if !settled {
            window.request_animation_frame();
        }

        let text = mix(shades.text, shades.text_hover, hover);
        let ground = mix(mix(shades.rest, shades.hover, hover), shades.press, press);
        let dithers = matches!(self.look, Look::Outline | Look::Ghost);
        let show_ring = focus.is_focused(window) && keyboard && active;
        let nudge = cap_offset(window, px(tokens.font_size));

        let hovered = state.clone();
        let pressed = state.clone();
        let released = state.clone();
        let abandoned = state;
        let on_click = self.on_click.filter(|_| active);
        let on_key = on_click.clone();

        div()
            .id(self.id)
            .track_focus(&focus.tab_stop(active))
            .relative()
            .top(px(press))
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .gap(px(tokens.gap))
            .h(px(tokens.control))
            .px(px(tokens.pad))
            .rounded(px(RADIUS_CONTROL))
            .bg(ground)
            .when_some(shades.edge, |this, edge| this.border_1().border_color(edge))
            .font_weight(FontWeight(WEIGHT_STRONG))
            .text_size(px(tokens.font_size))
            .line_height(px(tokens.line_height))
            .text_color(text)
            .when(self.disabled && !busy, |this| this.opacity(0.5))
            .when(active, |this| this.cursor_pointer())
            .when(show_ring, |this| {
                this.shadow(vec![gpui::BoxShadow {
                    color: gpui::Hsla::from(colors.focus),
                    offset: gpui::point(px(0.), px(0.)),
                    blur_radius: px(0.),
                    spread_radius: px(1.),
                }])
            })
            .on_hover(move |over, _, cx| {
                hovered.update(cx, |state, cx| {
                    state.pointer_over = *over;
                    cx.notify();
                });
            })
            .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                cx.global_mut::<Theme>().keyboard_input = false;
                pressed.update(cx, |state, cx| {
                    state.held = true;
                    cx.notify();
                });
            })
            .on_mouse_up(MouseButton::Left, move |_, _, cx| {
                released.update(cx, |state, cx| {
                    state.held = false;
                    cx.notify();
                });
            })
            .on_mouse_up_out(MouseButton::Left, move |_, _, cx| {
                abandoned.update(cx, |state, cx| {
                    state.held = false;
                    cx.notify();
                });
            })
            .when_some(on_click, |this, handler| {
                this.on_click(move |event, window, cx| handler(event, window, cx))
            })
            .when_some(self.tip, |this, text| this.tooltip(crate::tooltip::tooltip(text)))
            .when_some(on_key, |this, handler| {
                this.on_key_down(move |event: &KeyDownEvent, window, cx| {
                    if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                        handler(&ClickEvent::default(), window, cx);
                        cx.stop_propagation();
                    }
                })
            })
            .when(dithers && hover > 0.0, |this| {
                this.child(checker(fade(text, DITHER_HOVER_INK * hover), DITHER_HOVER_CELL, RADIUS_CONTROL))
            })
            .when_some(self.icon, |this, path| {
                this.child(
                    gpui::svg()
                        .path(path)
                        .flex_none()
                        .size(px(tokens.icon))
                        .text_color(text)
                        .when(busy, |this| this.invisible()),
                )
            })
            .child(
                div()
                    .debug_selector(|| "button-label".into())
                    .relative()
                    .top(nudge)
                    .when(busy, |this| this.invisible())
                    .child(self.label),
            )
            .when(busy, |this| this.child(counting_cells(text, count)))
    }
}

fn counting_cells(ink: Rgba, count: usize) -> impl IntoElement {
    let cell = |index: usize| div().size(px(3.)).when(index < count, |this| this.bg(ink));
    div().absolute().flex().items_center().gap(px(3.)).child(cell(0)).child(cell(1)).child(cell(2))
}
