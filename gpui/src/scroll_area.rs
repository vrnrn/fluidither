//! A vertical scroll box with a slim bar, which GPUI does not draw by itself. The bar
//! shows position. It follows the wheel and cannot be dragged yet.

use crate::theme::Theme;
use gpui::{
    canvas, div, fill, point, prelude::*, px, size, AnyElement, App, Bounds, ElementId, IntoElement,
    ParentElement, RenderOnce, ScrollHandle, StyleRefinement, Styled, Window,
};

const BAR_WIDTH: f32 = 4.0;
const SHORTEST_BAR: f32 = 24.0;

#[derive(IntoElement)]
pub struct ScrollArea {
    id: ElementId,
    children: Vec<AnyElement>,
    style: StyleRefinement,
}

impl ScrollArea {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self { id: id.into(), children: Vec::new(), style: StyleRefinement::default() }
    }
}

impl ParentElement for ScrollArea {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl Styled for ScrollArea {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

/// Where the bar sits: its top and length along a track of `visible` points.
pub fn bar_span(visible: f32, content: f32, scrolled: f32) -> Option<(f32, f32)> {
    if content <= visible || visible <= 0.0 {
        return None;
    }
    let length = (visible * visible / content).max(SHORTEST_BAR).min(visible);
    let travel = visible - length;
    let share = (scrolled / (content - visible)).clamp(0.0, 1.0);
    Some((travel * share, length))
}

impl RenderOnce for ScrollArea {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let edge = Theme::get(cx).colors().edge;
        let state = window.use_keyed_state(self.id.clone(), cx, |_, _| ScrollHandle::new());
        let handle = state.read(cx).clone();
        let painted = handle.clone();
        let mut outer = div();
        *outer.style() = std::mem::take(&mut self.style);
        let notify = state;

        // Relative after the copy, or the copy would wipe it.
        outer
            .relative()
            .child(
                div()
                    .id(self.id)
                    .size_full()
                    .overflow_y_scroll()
                    .track_scroll(&handle)
                    .on_scroll_wheel(move |_, _, cx| notify.update(cx, |_, cx| cx.notify()))
                    .children(self.children),
            )
            .child(
                canvas(
                    |_, _, _| (),
                    move |bounds: Bounds<gpui::Pixels>, _, window, _| {
                        let visible = f32::from(bounds.size.height);
                        let content = visible + f32::from(painted.max_offset().height);
                        let scrolled = -f32::from(painted.offset().y);
                        if let Some((top, length)) = bar_span(visible, content, scrolled) {
                            let origin = point(bounds.right() - px(BAR_WIDTH), bounds.top() + px(top));
                            window.paint_quad(fill(Bounds::new(origin, size(px(BAR_WIDTH), px(length))), edge));
                        }
                    },
                )
                // Offsets too: absolute alone starts below the scrolling box.
                .absolute()
                .top_0()
                .left_0()
                .size_full(),
            )
    }
}
