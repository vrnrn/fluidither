//! A segmented tab row. A thumb marks the selection and a peek previews the
//! hovered tab, both gliding on springs over measured tab positions. Selection
//! follows the arrow keys.
//!
//! This is the tab row only: render the matching content yourself from
//! `on_change`. GPUI has no accessibility API, so screen readers see nothing here.

use crate::generated::{
    DITHER_HOVER_CELL, DITHER_HOVER_INK, GLIDE, QUICK, RADIUS_PILL, WEIGHT_REGULAR, WEIGHT_STRONG,
};
use crate::paint::checker;
use crate::spring::{Spring, LONGEST_STEP};
use crate::theme::{cap_offset, fade, mix, Density, Theme};
use gpui::{
    div, prelude::*, px, App, Bounds, ElementId, Entity, FocusHandle, FontWeight, IntoElement,
    KeyDownEvent, MouseButton, Pixels, RenderOnce, SharedString, Window,
};
use std::rc::Rc;
use std::time::Instant;

#[derive(Clone, Debug)]
pub struct Tab {
    label: SharedString,
    icon: Option<SharedString>,
    disabled: bool,
}

impl Tab {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self { label: label.into(), icon: None, disabled: false }
    }

    /// An SVG from the app's assets, drawn before the label.
    pub fn icon(mut self, path: impl Into<SharedString>) -> Self {
        self.icon = Some(path.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl<T: Into<SharedString>> From<T> for Tab {
    fn from(label: T) -> Self {
        Tab::new(label)
    }
}

type OnChange = Rc<dyn Fn(usize, &mut Window, &mut App)>;

#[derive(IntoElement)]
pub struct Tabs {
    id: ElementId,
    tabs: Vec<Tab>,
    selected: Option<usize>,
    initial: usize,
    density: Density,
    dithered_peek: bool,
    underline: bool,
    on_change: Option<OnChange>,
}

impl Tabs {
    pub fn new<T: Into<Tab>>(id: impl Into<ElementId>, tabs: impl IntoIterator<Item = T>) -> Self {
        Self {
            id: id.into(),
            tabs: tabs.into_iter().map(Into::into).collect(),
            selected: None,
            initial: 0,
            density: Density::default(),
            dithered_peek: true,
            underline: false,
            on_change: None,
        }
    }

    /// Own the selection yourself. Without it the component keeps its own.
    pub fn selected(mut self, index: usize) -> Self {
        self.selected = Some(index);
        self
    }

    pub fn initial(mut self, index: usize) -> Self {
        self.initial = index;
        self
    }

    pub fn density(mut self, density: Density) -> Self {
        self.density = density;
        self
    }

    /// A lighter form for many tabs: no track, and a line for the thumb.
    pub fn underline(mut self) -> Self {
        self.underline = true;
        self
    }

    /// A flat tint for the peek instead of a dither.
    pub fn flat_peek(mut self) -> Self {
        self.dithered_peek = false;
        self
    }

    pub fn on_change(mut self, handler: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

struct TabsState {
    focus: FocusHandle,
    selected: usize,
    /// Left edge and width of each tab, from the first tab's left edge.
    spans: Vec<(f64, f64)>,
    /// The first tab's left edge in the window, to read pointer positions.
    origin: f64,
    peeking: Option<usize>,
    thumb: [Spring; 2],
    peek: [Spring; 2],
    peek_opacity: Spring,
    brightness: Vec<Spring>,
    placed: bool,
    last_frame: Instant,
}

impl TabsState {
    fn new(selected: usize, focus: FocusHandle) -> Self {
        Self {
            focus,
            selected,
            spans: Vec::new(),
            origin: 0.0,
            peeking: None,
            thumb: [Spring::new(GLIDE, 0.0); 2],
            peek: [Spring::new(QUICK, 0.0); 2],
            peek_opacity: Spring::new(QUICK, 0.0),
            brightness: Vec::new(),
            placed: false,
            last_frame: Instant::now(),
        }
    }

    fn measure(&mut self, bounds: &[Bounds<Pixels>]) -> bool {
        let Some(first) = bounds.first() else { return false };
        let origin = f64::from(f32::from(first.origin.x));
        let spans: Vec<_> = bounds
            .iter()
            .map(|tab| (f64::from(f32::from(tab.origin.x)) - origin, f64::from(f32::from(tab.size.width))))
            .collect();
        let moved = spans != self.spans;
        self.spans = spans;
        self.origin = origin;
        moved
    }

    fn tab_at(&self, x: f64) -> Option<usize> {
        let x = x - self.origin;
        self.spans.iter().position(|(left, width)| x >= *left && x < left + width)
    }

    /// Point every spring at where it belongs, and step them. True once all landed.
    fn advance(&mut self, reduced: bool) -> bool {
        if let Some(&(left, width)) = self.spans.get(self.selected) {
            if self.placed {
                self.thumb[0].target = left;
                self.thumb[1].target = width;
            } else {
                self.thumb[0].jump(left);
                self.thumb[1].jump(width);
                self.placed = true;
            }
        }
        // The peek fades in where the pointer lands, glides between tabs after
        // that, and fades out where it is.
        let shown = self.peeking.filter(|index| *index != self.selected);
        if let Some(&(left, width)) = shown.and_then(|index| self.spans.get(index)) {
            if self.peek_opacity.target == 0.0 {
                self.peek[0].jump(left);
                self.peek[1].jump(width);
            }
            self.peek[0].target = left;
            self.peek[1].target = width;
            self.peek_opacity.target = 1.0;
        } else {
            self.peek_opacity.target = 0.0;
        }
        self.brightness.resize(self.spans.len(), Spring::new(QUICK, 0.0));
        for (index, spring) in self.brightness.iter_mut().enumerate() {
            spring.target = f64::from(u8::from(index == self.selected || Some(index) == self.peeking));
        }

        let springs = self
            .thumb
            .iter_mut()
            .chain(&mut self.peek)
            .chain(std::iter::once(&mut self.peek_opacity))
            .chain(&mut self.brightness);
        let dt = self.last_frame.elapsed().as_secs_f64().min(LONGEST_STEP);
        self.last_frame = Instant::now();
        let mut landed = true;
        for spring in springs {
            if reduced {
                spring.jump(spring.target);
            }
            landed &= spring.step(dt);
        }
        landed
    }
}

/// Where a key moves the selection, skipping disabled tabs and wrapping.
fn step_with_key(key: &str, from: usize, tabs: &[Tab]) -> Option<usize> {
    let usable: Vec<usize> = (0..tabs.len()).filter(|index| !tabs[*index].disabled).collect();
    let here = usable.iter().position(|index| *index == from)?;
    let count = usable.len();
    match key {
        "right" => Some(usable[(here + 1) % count]),
        "left" => Some(usable[(here + count - 1) % count]),
        "home" => usable.first().copied(),
        "end" => usable.last().copied(),
        _ => None,
    }
}

fn select(
    state: &Entity<TabsState>,
    index: usize,
    owned_outside: bool,
    on_change: Option<&OnChange>,
    window: &mut Window,
    cx: &mut App,
) {
    let changed = state.update(cx, |state, cx| {
        let changed = state.selected != index;
        if !owned_outside {
            state.selected = index;
        }
        cx.notify();
        changed
    });
    if let Some(handler) = on_change.filter(|_| changed) {
        handler(index, window, cx);
    }
}

impl RenderOnce for Tabs {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let colors = theme.colors();
        let (reduced, keyboard) = (theme.reduced_motion, theme.keyboard_input);
        let thumb_shadow = theme.shadow(1);
        let tokens = self.density.tokens();
        let start = self.selected.unwrap_or(self.initial);
        let owned_outside = self.selected.is_some();

        let state = window.use_keyed_state(self.id.clone(), cx, |_, cx| {
            TabsState::new(start, cx.focus_handle().tab_stop(true))
        });
        let view = state.update(cx, |state, _| {
            if let Some(index) = self.selected {
                state.selected = index;
            }
            let landed = state.advance(reduced);
            let brightness: Vec<f32> = state.brightness.iter().map(|spring| spring.position as f32).collect();
            (
                state.focus.clone(),
                state.selected,
                state.placed,
                state.thumb.map(|spring| spring.position as f32),
                state.peek.map(|spring| spring.position as f32),
                state.peek_opacity.position as f32,
                brightness,
                landed,
            )
        });
        let (focus, selected, placed, thumb, peek, peek_opacity, brightness, landed) = view;
        if !landed {
            window.request_animation_frame();
        }

        let inset = px(tokens.inset);
        let height = px(tokens.segment);
        let nudge = cap_offset(window, px(tokens.font_size));
        let show_ring = focus.is_focused(window) && keyboard;
        let tabs = Rc::new(self.tabs);
        let on_change = self.on_change;
        let dithered_peek = self.dithered_peek;
        let underline = self.underline;

        let row: Vec<_> = tabs
            .iter()
            .enumerate()
            .map(|(index, tab)| {
                let glow = brightness.get(index).copied().unwrap_or(0.0);
                let text = mix(colors.ink_soft, colors.ink, glow);
                let weight = if index == selected { WEIGHT_STRONG } else { WEIGHT_REGULAR };
                let clicked = state.clone();
                let handler = on_change.clone();
                let usable = !tab.disabled;
                // The label sits in a box as wide as its strong weight, so changing
                // weight moves nothing and it stays centered at either weight.
                let label = div()
                    .relative()
                    .top(nudge)
                    .child(div().invisible().font_weight(FontWeight(WEIGHT_STRONG)).child(tab.label.clone()))
                    .child(
                        div()
                            .absolute()
                            .inset_0()
                            .flex()
                            .justify_center()
                            .font_weight(FontWeight(weight))
                            .text_color(text)
                            .child(tab.label.clone()),
                    );
                div()
                    .id(index)
                    .debug_selector(|| format!("tab-{index}"))
                    .flex()
                    .flex_none()
                    .items_center()
                    .justify_center()
                    .gap(px(tokens.gap))
                    .h(height)
                    .px(px(tokens.tab_pad))
                    .when(tab.disabled, |this| this.opacity(0.5))
                    .when(usable, |this| {
                        this.cursor_pointer().on_click(move |_, window, cx| {
                            cx.global_mut::<Theme>().keyboard_input = false;
                            select(&clicked, index, owned_outside, handler.as_ref(), window, cx);
                        })
                    })
                    .when_some(tab.icon.clone(), |this, path| {
                        this.child(gpui::svg().path(path).flex_none().size(px(tokens.icon)).text_color(text))
                    })
                    .child(label)
            })
            .collect();

        let measured = state.clone();
        let moved = state.clone();
        let left = state.clone();
        let pressed = state;
        let moved_tabs = tabs.clone();

        div()
            .id(self.id)
            .track_focus(&focus)
            .relative()
            .flex()
            .flex_none()
            .when(!underline, |this| this.p(inset).rounded(px(RADIUS_PILL + tokens.inset)).bg(colors.wash))
            .when(underline, |this| this.border_b_1().border_color(colors.line))
            .text_size(px(tokens.font_size))
            .line_height(px(tokens.line_height))
            .on_mouse_down(MouseButton::Left, |_, _, cx| {
                cx.global_mut::<Theme>().keyboard_input = false;
            })
            .on_mouse_move(move |event, _, cx| {
                moved.update(cx, |state, cx| {
                    let over = state
                        .tab_at(f64::from(f32::from(event.position.x)))
                        .filter(|index| !moved_tabs[*index].disabled);
                    if over != state.peeking {
                        state.peeking = over;
                        cx.notify();
                    }
                });
            })
            .on_hover(move |inside, _, cx| {
                if !*inside {
                    left.update(cx, |state, cx| {
                        state.peeking = None;
                        cx.notify();
                    });
                }
            })
            .on_key_down(move |event: &KeyDownEvent, window, cx| {
                let keys = event.keystroke.modifiers;
                if keys.control || keys.alt || keys.platform || keys.shift {
                    return;
                }
                let from = pressed.read(cx).selected;
                if let Some(index) = step_with_key(event.keystroke.key.as_str(), from, &tabs) {
                    cx.global_mut::<Theme>().keyboard_input = true;
                    select(&pressed, index, owned_outside, on_change.as_ref(), window, cx);
                    cx.stop_propagation();
                }
            })
            .when(placed && underline, |this| {
                this.child(div().absolute().bottom(px(-1.)).left(px(thumb[0])).w(px(thumb[1])).h(px(2.)).bg(if show_ring { colors.focus } else { colors.ink }))
            })
            .when(placed && !underline, |this| {
                this.child(
                    div()
                        .absolute()
                        .top(inset)
                        .left(inset + px(thumb[0]))
                        .w(px(thumb[1]))
                        .h(height)
                        .rounded(px(RADIUS_PILL))
                        .bg(colors.surfaces[2])
                        .shadow(if show_ring { vec![ring(colors.focus)] } else { thumb_shadow }),
                )
            })
            .when(peek_opacity > 0.0, |this| {
                let area = div()
                    .absolute()
                    .top(if underline { px(0.) } else { inset })
                    .left(if underline { px(peek[0]) } else { inset + px(peek[0]) })
                    .w(px(peek[1]))
                    .h(height)
                    .rounded(px(RADIUS_PILL));
                this.child(if dithered_peek {
                    let ink = fade(colors.ink, DITHER_HOVER_INK * peek_opacity);
                    area.child(checker(ink, DITHER_HOVER_CELL, RADIUS_PILL))
                } else {
                    area.bg(fade(colors.soft, peek_opacity))
                })
            })
            .child(div().relative().flex().children(row).on_children_prepainted(move |bounds, _, cx| {
                measured.update(cx, |state, cx| {
                    if state.measure(&bounds) {
                        cx.notify();
                    }
                });
            }))
    }
}

fn ring(color: gpui::Rgba) -> gpui::BoxShadow {
    gpui::BoxShadow {
        color: gpui::Hsla::from(color),
        offset: gpui::point(px(0.), px(0.)),
        blur_radius: px(0.),
        spread_radius: px(1.),
    }
}
