//! The floating list behind menus, selects, comboboxes and the command menu. Rows
//! have a fixed height, so a dithered mark can glide between them without measuring.

use crate::generated::{DITHER_HOVER_CELL, DITHER_HOVER_INK, GLYPH_TICK, QUICK, RADIUS_PILL, WEIGHT_STRONG};
use crate::paint::{checker, glyph};
use crate::spring::{Spring, LONGEST_STEP};
use crate::theme::{cap_offset, fade, Density, Theme};
use gpui::{
    div, prelude::*, px, App, Div, FontWeight, Pixels, SharedString, Stateful, Window,
};
use std::rc::Rc;
use std::time::Instant;

pub(crate) const MOST_ROWS: usize = 8;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub label: SharedString,
    pub disabled: bool,
}

impl Row {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self { label: label.into(), disabled: false }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl<T: Into<SharedString>> From<T> for Row {
    fn from(label: T) -> Self {
        Row::new(label)
    }
}

/// The next usable row from `from`, wrapping. With no `from`, the first or last.
pub fn step(rows: &[Row], shown: &[usize], from: Option<usize>, delta: isize) -> Option<usize> {
    let usable: Vec<usize> = shown.iter().copied().filter(|index| !rows[*index].disabled).collect();
    if usable.is_empty() {
        return None;
    }
    let count = usable.len() as isize;
    let next = match from.and_then(|index| usable.iter().position(|each| *each == index)) {
        Some(here) => (here as isize + delta + count) % count,
        None if delta > 0 => 0,
        None => count - 1,
    };
    Some(usable[next as usize])
}

/// Rows whose label contains `query`, ignoring case.
pub fn matching(rows: &[Row], query: &str) -> Vec<usize> {
    let wanted = query.trim().to_lowercase();
    (0..rows.len()).filter(|index| rows[*index].label.to_lowercase().contains(&wanted)).collect()
}

/// Typing a few letters jumps to the row starting with them.
#[derive(Default)]
pub struct Typeahead {
    typed: String,
    last: Option<Instant>,
}

impl Typeahead {
    pub fn find(&mut self, rows: &[Row], from: Option<usize>, character: &str) -> Option<usize> {
        let paused = self.last.is_none_or(|last| last.elapsed().as_millis() > 600);
        if paused {
            self.typed.clear();
        }
        self.last = Some(Instant::now());
        self.typed.push_str(&character.to_lowercase());
        let start = from.map_or(0, |index| index + usize::from(self.typed.chars().count() == 1));
        (0..rows.len())
            .map(|offset| (start + offset) % rows.len())
            .find(|index| !rows[*index].disabled && rows[*index].label.to_lowercase().starts_with(&self.typed))
    }
}

/// The springs of the gliding mark.
pub(crate) struct Mark {
    y: Spring,
    opacity: Spring,
    last_frame: Instant,
}

impl Mark {
    pub(crate) fn new() -> Self {
        Self { y: Spring::new(QUICK, 0.0), opacity: Spring::new(QUICK, 0.0), last_frame: Instant::now() }
    }

    /// Point at a row, or at nothing. Returns where it is and whether it landed.
    pub(crate) fn advance(&mut self, row: Option<usize>, height: f32, reduced: bool) -> (f32, f32, bool) {
        match row {
            Some(row) => {
                let y = f64::from(row as f32 * height);
                if self.opacity.target == 0.0 {
                    self.y.jump(y);
                }
                self.y.target = y;
                self.opacity.target = 1.0;
            }
            None => self.opacity.target = 0.0,
        }
        if reduced {
            self.y.jump(self.y.target);
            self.opacity.jump(self.opacity.target);
        }
        let dt = self.last_frame.elapsed().as_secs_f64().min(LONGEST_STEP);
        self.last_frame = Instant::now();
        let landed = self.y.step(dt) & self.opacity.step(dt);
        (self.y.position as f32, self.opacity.position as f32, landed)
    }
}

pub(crate) struct Panel<'a> {
    pub rows: &'a [Row],
    pub shown: &'a [usize],
    pub selected: Option<usize>,
    pub mark: (f32, f32),
    pub density: Density,
    pub empty: Option<SharedString>,
    pub on_hover: Rc<dyn Fn(Option<usize>, &mut Window, &mut App)>,
    pub on_pick: Rc<dyn Fn(usize, &mut Window, &mut App)>,
}

/// The panel surface alone, for lists that sit inside something else.
pub(crate) fn rows_element(panel: Panel<'_>, window: &Window, cx: &App) -> Stateful<Div> {
    let theme = Theme::get(cx);
    let colors = theme.colors();
    let tokens = panel.density.tokens();
    let height = px(tokens.segment);
    let nudge = cap_offset(window, px(tokens.font_size));
    let (mark_y, mark_opacity) = panel.mark;
    let leave = panel.on_hover.clone();

    let rows: Vec<_> = panel
        .shown
        .iter()
        .map(|&index| {
            let row = &panel.rows[index];
            let hover = panel.on_hover.clone();
            let pick = panel.on_pick.clone();
            let chosen = panel.selected == Some(index);
            div()
                .id(("row", index))
                .debug_selector(|| format!("row-{index}"))
                .relative()
                .flex()
                .flex_none()
                .items_center()
                .justify_between()
                .h(height)
                .px(px(tokens.tab_pad))
                .when(chosen, |this| this.font_weight(FontWeight(WEIGHT_STRONG)))
                .when(row.disabled, |this| this.opacity(0.5))
                .when(!row.disabled, |this| {
                    this.cursor_pointer()
                        .on_mouse_move(move |_, window, cx| hover(Some(index), window, cx))
                        .on_click(move |_, window, cx| pick(index, window, cx))
                })
                .child(div().relative().top(nudge).child(row.label.clone()))
                .when(chosen, |this| this.child(div().relative().w(px(12.)).h(px(10.)).child(glyph(GLYPH_TICK, colors.ink))))
        })
        .collect();

    div()
        .id("fd-rows")
        .relative()
        .flex()
        .flex_col()
        .on_hover(move |inside, window, cx| {
            if !*inside {
                leave(None, window, cx);
            }
        })
        .when(mark_opacity > 0.0, |this| {
            this.child(
                div()
                    .absolute()
                    .top(px(mark_y))
                    .left_0()
                    .right_0()
                    .h(height)
                    .child(checker(fade(colors.ink, DITHER_HOVER_INK * mark_opacity), DITHER_HOVER_CELL, RADIUS_PILL)),
            )
        })
        .children(rows)
        .when_some(panel.empty, |this, empty| {
            this.child(div().h(height).px(px(tokens.tab_pad)).flex().items_center().text_color(colors.ink_soft).child(empty))
        })
}

/// The floating surface around rows.
pub(crate) fn surface(id: &'static str, content: Stateful<Div>, density: Density, min_width: Pixels, cx: &App) -> Stateful<Div> {
    let theme = Theme::get(cx);
    let colors = theme.colors();
    let tokens = density.tokens();
    div()
        .id(id)
        .occlude()
        .min_w(min_width)
        .max_h(px(tokens.segment * MOST_ROWS as f32 + tokens.inset * 2.0))
        .overflow_y_scroll()
        .p(px(tokens.inset))
        .rounded(px(RADIUS_PILL + tokens.inset))
        .bg(colors.surfaces[3])
        .shadow(theme.shadow(2))
        .text_color(colors.ink)
        .text_size(px(tokens.font_size))
        .line_height(px(tokens.line_height))
        .font_weight(FontWeight(400.))
        .child(content)
}
