//! A table of text. Rows answer the pointer with a fine dither. Columns share the
//! width equally unless you weight them. GPUI has no accessibility API, so screen
//! readers see nothing here.

use crate::generated::{DITHER_HOVER_CELL, DITHER_HOVER_INK, WEIGHT_STRONG};
use crate::paint::checker;
use crate::theme::{cap_offset, fade, Density, Theme};
use gpui::{div, prelude::*, px, App, ElementId, FontWeight, IntoElement, RenderOnce, SharedString, Window};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Align {
    #[default]
    Start,
    End,
}

#[derive(IntoElement)]
pub struct Table {
    id: ElementId,
    columns: Vec<(SharedString, Align, f32)>,
    rows: Vec<Vec<SharedString>>,
    density: Density,
}

impl Table {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self { id: id.into(), columns: Vec::new(), rows: Vec::new(), density: Density::default() }
    }

    pub fn column(mut self, title: impl Into<SharedString>) -> Self {
        self.columns.push((title.into(), Align::Start, 1.0));
        self
    }

    /// A right-aligned column, for numbers.
    pub fn number_column(mut self, title: impl Into<SharedString>) -> Self {
        self.columns.push((title.into(), Align::End, 1.0));
        self
    }

    pub fn row<T: Into<SharedString>>(mut self, cells: impl IntoIterator<Item = T>) -> Self {
        self.rows.push(cells.into_iter().map(Into::into).collect());
        self
    }

    pub fn density(mut self, density: Density) -> Self {
        self.density = density;
        self
    }
}

#[derive(Default)]
struct TableState {
    hovered: Option<usize>,
}

impl RenderOnce for Table {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = Theme::get(cx).colors();
        let tokens = self.density.tokens();
        let height = px(tokens.control);
        let nudge = cap_offset(window, px(tokens.font_size));
        let state = window.use_keyed_state(self.id.clone(), cx, |_, _| TableState::default());
        let hovered = state.read(cx).hovered;
        let columns = self.columns;
        let last = self.rows.len().saturating_sub(1);

        let cell = |text: SharedString, align: Align| {
            div()
                .relative()
                .top(nudge)
                .flex()
                .flex_1()
                .min_w_0()
                .px(px(tokens.tab_pad))
                .when(align == Align::End, |this| this.justify_end())
                .child(text)
        };
        let head = div()
            .flex()
            .items_center()
            .h(height)
            .border_b_1()
            .border_color(colors.line)
            .text_color(colors.ink_soft)
            .font_weight(FontWeight(WEIGHT_STRONG))
            .children(columns.iter().map(|(title, align, _)| cell(title.clone(), *align)));

        let rows = self.rows.into_iter().enumerate().map(|(index, row)| {
            let entered = state.clone();
            div()
                .id(("table-row", index))
                .relative()
                .flex()
                .items_center()
                .h(height)
                .when(index != last, |this| this.border_b_1().border_color(colors.line))
                .on_hover(move |over, _, cx: &mut App| {
                    entered.update(cx, |state, cx| {
                        if *over {
                            state.hovered = Some(index);
                        } else if state.hovered == Some(index) {
                            state.hovered = None;
                        }
                        cx.notify();
                    });
                })
                .when(hovered == Some(index), |this| {
                    this.child(checker(fade(colors.ink, DITHER_HOVER_INK), DITHER_HOVER_CELL, 0.0))
                })
                .children(row.into_iter().zip(&columns).map(|(text, (_, align, _))| cell(text, *align)))
        });

        div()
            .id(self.id)
            .flex()
            .flex_col()
            .w_full()
            .text_color(colors.ink)
            .text_size(px(tokens.font_size))
            .line_height(px(tokens.line_height))
            .child(head)
            .children(rows)
    }
}
