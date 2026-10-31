//! Sections that open and close on a spring. Tab moves between headers, and Enter
//! or Space toggles. One section stays open at a time unless you call `multiple`.

use crate::generated::{GLYPH_ARROW, GLYPH_ARROW_UP, SETTLE, WEIGHT_STRONG};
use crate::paint::{focus_ring, glyph};
use crate::spring::{Spring, LONGEST_STEP};
use crate::theme::{cap_offset, Density, Theme};
use gpui::{
    div, prelude::*, px, AnyElement, App, ElementId, FocusHandle, FontWeight, IntoElement,
    KeyDownEvent, MouseButton, RenderOnce, SharedString, Window,
};
use std::time::Instant;

struct Section {
    title: SharedString,
    body: AnyElement,
}

#[derive(IntoElement)]
pub struct Accordion {
    id: ElementId,
    sections: Vec<Section>,
    initial: Option<usize>,
    multiple: bool,
    density: Density,
}

impl Accordion {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self { id: id.into(), sections: Vec::new(), initial: None, multiple: false, density: Density::default() }
    }

    pub fn section(mut self, title: impl Into<SharedString>, body: impl IntoElement) -> Self {
        self.sections.push(Section { title: title.into(), body: body.into_any_element() });
        self
    }

    pub fn initial(mut self, index: usize) -> Self {
        self.initial = Some(index);
        self
    }

    pub fn multiple(mut self) -> Self {
        self.multiple = true;
        self
    }

    pub fn density(mut self, density: Density) -> Self {
        self.density = density;
        self
    }
}

struct AccordionState {
    focus: Vec<FocusHandle>,
    open: Vec<bool>,
    progress: Vec<Spring>,
    heights: Vec<f32>,
    last_frame: Instant,
}

impl AccordionState {
    fn toggle(&mut self, index: usize, multiple: bool) {
        let opening = !self.open[index];
        if !multiple {
            self.open.iter_mut().for_each(|open| *open = false);
        }
        self.open[index] = opening;
    }

    fn advance(&mut self, reduced: bool) -> bool {
        let dt = self.last_frame.elapsed().as_secs_f64().min(LONGEST_STEP);
        self.last_frame = Instant::now();
        let mut landed = true;
        for (spring, open) in self.progress.iter_mut().zip(&self.open) {
            spring.target = f64::from(u8::from(*open));
            if reduced {
                spring.jump(spring.target);
            }
            landed &= spring.step(dt);
        }
        landed
    }
}

impl RenderOnce for Accordion {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let colors = theme.colors();
        let (reduced, keyboard) = (theme.reduced_motion, theme.keyboard_input);
        let tokens = self.density.tokens();
        let count = self.sections.len();
        let initial = self.initial;
        let multiple = self.multiple;
        let state = window.use_keyed_state(self.id.clone(), cx, |_, cx| AccordionState {
            focus: (0..count).map(|_| cx.focus_handle().tab_stop(true)).collect(),
            open: (0..count).map(|index| Some(index) == initial).collect(),
            progress: (0..count).map(|index| Spring::new(SETTLE, f64::from(u8::from(Some(index) == initial)))).collect(),
            heights: vec![0.0; count],
            last_frame: Instant::now(),
        });
        let landed = state.update(cx, |state, _| state.advance(reduced));
        if !landed {
            window.request_animation_frame();
        }
        let nudge = cap_offset(window, px(tokens.font_size));

        let sections: Vec<_> = self
            .sections
            .into_iter()
            .enumerate()
            .map(|(index, section)| {
                let (focus, open, shown, height) = {
                    let state = state.read(cx);
                    (state.focus[index].clone(), state.open[index], state.progress[index].position as f32, state.heights[index])
                };
                let ring = focus.is_focused(window) && keyboard;
                let clicked = state.clone();
                let keyed = state.clone();
                let measured = state.clone();
                div()
                    .border_b_1()
                    .border_color(colors.line)
                    .child(
                        div()
                            .id(("section", index))
                            .debug_selector(|| format!("section-{index}"))
                            .track_focus(&focus)
                            .relative()
                            .flex()
                            .items_center()
                            .justify_between()
                            .min_h(px(tokens.control))
                            .font_weight(FontWeight(WEIGHT_STRONG))
                            .cursor_pointer()
                            .on_mouse_down(MouseButton::Left, |_, _, cx| {
                                cx.global_mut::<Theme>().keyboard_input = false;
                            })
                            .on_click(move |_, _, cx| {
                                clicked.update(cx, |state, cx| {
                                    state.toggle(index, multiple);
                                    cx.notify();
                                });
                            })
                            .on_key_down(move |event: &KeyDownEvent, _, cx| {
                                if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                                    keyed.update(cx, |state, cx| {
                                        state.toggle(index, multiple);
                                        cx.notify();
                                    });
                                    cx.stop_propagation();
                                }
                            })
                            .child(div().relative().top(nudge).child(section.title))
                            .child(div().relative().flex_none().w(px(10.)).h(px(6.)).child(glyph(if open { GLYPH_ARROW_UP } else { GLYPH_ARROW }, colors.ink_soft)))
                            .when(ring, |this| this.child(focus_ring(colors.focus, 5., 1.))),
                    )
                    .when(open || shown > 0.0, |this| {
                        this.child(
                            div()
                                .overflow_hidden()
                                .h(px(height * shown.max(0.0)))
                                .child(
                                    div()
                                        .debug_selector(|| format!("body-{index}"))
                                        .text_color(colors.ink_soft)
                                        .pb(px(12.))
                                        .child(section.body)
                                        .on_children_prepainted(move |bounds, _, cx| {
                                            let Some(first) = bounds.first() else { return };
                                            let tall = bounds.iter().map(|each| f32::from(each.bottom() - first.top())).fold(0.0, f32::max) + 12.0;
                                            measured.update(cx, |state, cx| {
                                                if (state.heights[index] - tall).abs() > 0.5 {
                                                    state.heights[index] = tall;
                                                    cx.notify();
                                                }
                                            });
                                        }),
                                ),
                        )
                    })
            })
            .collect();

        div()
            .id(self.id)
            .flex()
            .flex_col()
            .w_full()
            .text_color(colors.ink)
            .text_size(px(tokens.font_size))
            .line_height(px(tokens.line_height))
            .children(sections)
    }
}
