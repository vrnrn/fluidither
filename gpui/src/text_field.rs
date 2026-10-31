//! A one-line text field. It is a view rather than an element, because the system's
//! text input talks to a long-lived object: `cx.new(|cx| TextField::new(cx))`, then
//! add the entity as a child. Subscribe to `TextFieldEvent` to hear about changes.
//!
//! Typing arrives through GPUI's input handler, so dead keys and input methods take
//! the normal path.

use crate::generated::RADIUS_CONTROL;
use crate::paint::focus_ring;
use crate::theme::{cap_offset, fade, Density, Theme};
use gpui::{
    canvas, div, fill, point, prelude::*, px, size, App, Bounds, ClipboardItem, Context,
    CursorStyle, ElementInputHandler, Entity, EntityInputHandler, EventEmitter, FocusHandle,
    Focusable, Hsla, IntoElement, KeyDownEvent, MouseButton, MouseDownEvent, MouseMoveEvent, Pixels,
    Point, Render, ShapedLine, SharedString, TextRun, UTF16Selection, UnderlineStyle, Window,
};
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

const CARET_WIDTH: f32 = 1.5;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TextFieldEvent {
    Changed(String),
    Submitted(String),
}

pub struct TextField {
    focus: FocusHandle,
    text: String,
    placeholder: SharedString,
    density: Density,
    invalid: bool,
    disabled: bool,
    selection: Range<usize>,
    /// The end of the selection that moves when it is extended.
    active_at_start: bool,
    marked: Option<Range<usize>>,
    scroll: f32,
    shaped: Option<ShapedLine>,
    bounds: Bounds<Pixels>,
    dragging: bool,
}

impl EventEmitter<TextFieldEvent> for TextField {}

impl Focusable for TextField {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl TextField {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus: cx.focus_handle().tab_stop(true),
            text: String::new(),
            placeholder: SharedString::default(),
            density: Density::default(),
            invalid: false,
            disabled: false,
            selection: 0..0,
            active_at_start: false,
            marked: None,
            scroll: 0.0,
            shaped: None,
            bounds: Bounds::default(),
            dragging: false,
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn density(mut self, density: Density) -> Self {
        self.density = density;
        self
    }

    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self.selection = self.text.len()..self.text.len();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_placeholder(&mut self, placeholder: impl Into<SharedString>) {
        self.placeholder = placeholder.into();
    }

    /// Replace everything, with no `Changed` event.
    pub fn set_text(&mut self, text: impl Into<String>, cx: &mut Context<Self>) {
        self.text = text.into();
        self.selection = self.text.len()..self.text.len();
        self.active_at_start = false;
        self.marked = None;
        cx.notify();
    }

    pub fn set_invalid(&mut self, invalid: bool, cx: &mut Context<Self>) {
        self.invalid = invalid;
        cx.notify();
    }

    fn caret(&self) -> usize {
        if self.active_at_start { self.selection.start } else { self.selection.end }
    }

    fn collapse_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        self.selection = offset..offset;
        self.active_at_start = false;
        cx.notify();
    }

    fn extend_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        let anchor = if self.active_at_start { self.selection.end } else { self.selection.start };
        self.active_at_start = offset < anchor;
        self.selection = anchor.min(offset)..anchor.max(offset);
        cx.notify();
    }

    fn previous_boundary(&self, offset: usize) -> usize {
        self.text[..offset].grapheme_indices(true).next_back().map_or(0, |(index, _)| index)
    }

    fn next_boundary(&self, offset: usize) -> usize {
        self.text[offset..].graphemes(true).next().map_or(self.text.len(), |grapheme| offset + grapheme.len())
    }

    fn previous_word(&self, offset: usize) -> usize {
        self.text[..offset].unicode_word_indices().next_back().map_or(0, |(index, _)| index)
    }

    fn next_word(&self, offset: usize) -> usize {
        self.text[offset..]
            .unicode_word_indices()
            .next()
            .map_or(self.text.len(), |(index, word)| offset + index + word.len())
    }

    fn replace(&mut self, range: Range<usize>, new_text: &str, cx: &mut Context<Self>) {
        self.text.replace_range(range.clone(), new_text);
        let caret = range.start + new_text.len();
        self.selection = caret..caret;
        self.active_at_start = false;
        self.marked = None;
        cx.emit(TextFieldEvent::Changed(self.text.clone()));
        cx.notify();
    }

    fn selected_text(&self) -> String {
        self.text[self.selection.clone()].to_string()
    }

    fn on_key(&mut self, event: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        let keys = event.keystroke.modifiers;
        let command = keys.platform;
        let word = keys.alt;
        let caret = self.caret();
        let target = match event.keystroke.key.as_str() {
            "left" if command => Some(0),
            "right" if command => Some(self.text.len()),
            "left" if word => Some(self.previous_word(caret)),
            "right" if word => Some(self.next_word(caret)),
            "left" if !keys.shift && !self.selection.is_empty() => Some(self.selection.start),
            "right" if !keys.shift && !self.selection.is_empty() => Some(self.selection.end),
            "left" => Some(self.previous_boundary(caret)),
            "right" => Some(self.next_boundary(caret)),
            "home" | "up" => Some(0),
            "end" | "down" => Some(self.text.len()),
            _ => None,
        };
        if let Some(target) = target {
            if keys.shift {
                self.extend_to(target, cx);
            } else {
                self.collapse_to(target, cx);
            }
            cx.stop_propagation();
            return;
        }
        let selection = self.selection.clone();
        match event.keystroke.key.as_str() {
            "backspace" | "delete" if !selection.is_empty() => self.replace(selection, "", cx),
            "backspace" if selection.start > 0 => {
                let from = if command {
                    0
                } else if word {
                    self.previous_word(selection.start)
                } else {
                    self.previous_boundary(selection.start)
                };
                self.replace(from..selection.start, "", cx);
            }
            "delete" if selection.end < self.text.len() => {
                let to = if word { self.next_word(selection.end) } else { self.next_boundary(selection.end) };
                self.replace(selection.end..to, "", cx);
            }
            "a" if command => {
                self.selection = 0..self.text.len();
                self.active_at_start = false;
                cx.notify();
            }
            "c" if command && !selection.is_empty() => {
                cx.write_to_clipboard(ClipboardItem::new_string(self.selected_text()));
            }
            "x" if command && !selection.is_empty() => {
                cx.write_to_clipboard(ClipboardItem::new_string(self.selected_text()));
                self.replace(selection, "", cx);
            }
            "v" if command => {
                if let Some(pasted) = cx.read_from_clipboard().and_then(|item| item.text()) {
                    self.replace(selection, &pasted.replace(['\n', '\r'], " "), cx);
                }
            }
            "enter" => cx.emit(TextFieldEvent::Submitted(self.text.clone())),
            _ => return,
        }
        cx.stop_propagation();
    }

    fn offset_at(&self, position: Point<Pixels>) -> usize {
        let Some(line) = &self.shaped else { return 0 };
        if self.text.is_empty() {
            return 0;
        }
        line.closest_index_for_x(position.x - self.bounds.left() + px(self.scroll))
    }

    fn on_mouse_down(&mut self, event: &MouseDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        cx.global_mut::<Theme>().keyboard_input = false;
        self.dragging = true;
        let offset = self.offset_at(event.position);
        if event.modifiers.shift {
            self.extend_to(offset, cx);
        } else {
            self.collapse_to(offset, cx);
        }
    }

    fn on_mouse_move(&mut self, event: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        if self.dragging {
            let offset = self.offset_at(event.position);
            self.extend_to(offset, cx);
        }
    }

    fn to_utf16(&self, offset: usize) -> usize {
        self.text[..offset].encode_utf16().count()
    }

    fn from_utf16(&self, target: usize) -> usize {
        let mut units = 0;
        for (offset, character) in self.text.char_indices() {
            if units >= target {
                return offset;
            }
            units += character.len_utf16();
        }
        self.text.len()
    }

    fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.to_utf16(range.start)..self.to_utf16(range.end)
    }

    fn range_from_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.from_utf16(range.start)..self.from_utf16(range.end)
    }

    /// Lay out the line for this frame and keep the caret in view.
    fn shape(&mut self, width: f32, window: &mut Window, cx: &App) -> ShapedLine {
        let colors = Theme::get(cx).colors();
        let tokens = self.density.tokens();
        let font = window.text_style().font();
        let showing_placeholder = self.text.is_empty();
        let shown: SharedString = if showing_placeholder { self.placeholder.clone() } else { self.text.clone().into() };
        let color: Hsla = if showing_placeholder { colors.ink_soft.into() } else { colors.ink.into() };
        let run = |len: usize, underline: Option<UnderlineStyle>| TextRun {
            len,
            font: font.clone(),
            color,
            background_color: None,
            underline,
            strikethrough: None,
        };
        let runs = match &self.marked {
            Some(marked) if !showing_placeholder => {
                let line = UnderlineStyle { color: Some(color), thickness: px(1.), wavy: false };
                [(0..marked.start, None), (marked.clone(), Some(line)), (marked.end..shown.len(), None)]
                    .into_iter()
                    .filter(|(range, _)| !range.is_empty())
                    .map(|(range, underline)| run(range.len(), underline))
                    .collect()
            }
            _ => vec![run(shown.len(), None)],
        };
        let line = window.text_system().shape_line(shown, px(tokens.font_size), &runs, None);
        if !self.text.is_empty() {
            let caret_x = f32::from(line.x_for_index(self.caret()));
            if caret_x - self.scroll > width - CARET_WIDTH {
                self.scroll = caret_x - width + CARET_WIDTH;
            } else if caret_x < self.scroll {
                self.scroll = caret_x;
            }
        } else {
            self.scroll = 0.0;
        }
        line
    }
}

impl EntityInputHandler for TextField {
    fn text_for_range(
        &mut self,
        range: Range<usize>,
        adjusted: &mut Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<String> {
        let bytes = self.range_from_utf16(&range);
        adjusted.replace(self.range_to_utf16(&bytes));
        Some(self.text[bytes].to_string())
    }

    fn selected_text_range(
        &mut self,
        _: bool,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.range_to_utf16(&self.selection),
            reversed: self.active_at_start,
        })
    }

    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        self.marked.as_ref().map(|range| self.range_to_utf16(range))
    }

    fn unmark_text(&mut self, _: &mut Window, _: &mut Context<Self>) {
        self.marked = None;
    }

    fn replace_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.disabled {
            return;
        }
        let bytes = range
            .map(|range| self.range_from_utf16(&range))
            .or(self.marked.clone())
            .unwrap_or(self.selection.clone());
        self.replace(bytes, text, cx);
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        selected: Option<Range<usize>>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.disabled {
            return;
        }
        let bytes = range
            .map(|range| self.range_from_utf16(&range))
            .or(self.marked.clone())
            .unwrap_or(self.selection.clone());
        self.text.replace_range(bytes.clone(), text);
        self.marked = (!text.is_empty()).then(|| bytes.start..bytes.start + text.len());
        let caret = selected
            .map(|range| self.range_from_utf16(&range))
            .map_or(bytes.start + text.len(), |range| bytes.start + range.start);
        self.selection = caret..caret;
        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        range: Range<usize>,
        bounds: Bounds<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let line = self.shaped.as_ref()?;
        let bytes = self.range_from_utf16(&range);
        let left = line.x_for_index(bytes.start) - px(self.scroll);
        let right = line.x_for_index(bytes.end) - px(self.scroll);
        Some(Bounds::from_corners(
            point(bounds.left() + left, bounds.top()),
            point(bounds.left() + right, bounds.bottom()),
        ))
    }

    fn character_index_for_point(
        &mut self,
        position: Point<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<usize> {
        Some(self.to_utf16(self.offset_at(position)))
    }
}

impl Render for TextField {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::get(cx);
        let colors = theme.colors();
        let tokens = self.density.tokens();
        let focused = self.focus.is_focused(window);
        let ring = focused && theme.keyboard_input;
        let edge = if self.invalid {
            colors.red
        } else if focused {
            colors.focus
        } else {
            colors.edge
        };
        let nudge = cap_offset(window, px(tokens.font_size));
        let this: Entity<Self> = cx.entity();
        let painter = this.clone();
        let focus = self.focus.clone();
        let selection_ink = fade(colors.focus, 0.3);
        let caret_ink = colors.ink;
        let line_height = px(tokens.line_height);

        div()
            .id("fd-text-field")
            .track_focus(&self.focus.clone().tab_stop(!self.disabled))
            .relative()
            .flex()
            .items_center()
            .w_full()
            .h(px(tokens.control))
            .px(px(tokens.tab_pad))
            .rounded(px(RADIUS_CONTROL))
            .border_1()
            .border_color(edge)
            .bg(colors.surfaces[1])
            .text_size(px(tokens.font_size))
            .line_height(line_height)
            .when(self.disabled, |this| this.opacity(0.5))
            .when(!self.disabled, |this| {
                this.cursor(CursorStyle::IBeam)
                    .on_key_down(cx.listener(Self::on_key))
                    .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
                    .on_mouse_move(cx.listener(Self::on_mouse_move))
                    .on_mouse_up(MouseButton::Left, cx.listener(|this, _, _, _| this.dragging = false))
                    .on_mouse_up_out(MouseButton::Left, cx.listener(|this, _, _, _| this.dragging = false))
            })
            .child(
                canvas(
                    move |bounds, window, cx| {
                        this.update(cx, |field, cx| {
                            field.bounds = bounds;
                            field.shaped = Some(field.shape(f32::from(bounds.size.width), window, cx));
                        });
                    },
                    move |bounds, _, window, cx| {
                        let (line, selection, caret, scroll, typed) = painter.update(cx, |field, _| {
                            (field.shaped.clone(), field.selection.clone(), field.caret(), field.scroll, !field.text.is_empty())
                        });
                        let Some(line) = line else { return };
                        window.handle_input(&focus, ElementInputHandler::new(bounds, painter.clone()), cx);
                        let top = bounds.top() + (bounds.size.height - line_height) / 2.0;
                        let left = bounds.left() - px(scroll);
                        window.with_content_mask(Some(gpui::ContentMask { bounds }), |window| {
                            if typed && !selection.is_empty() {
                                let from = line.x_for_index(selection.start);
                                let to = line.x_for_index(selection.end);
                                let area = Bounds::new(point(left + from, top), size(to - from, line_height));
                                window.paint_quad(fill(area, selection_ink));
                            }
                            line.paint(point(left, top + nudge), line_height, window, cx).ok();
                            if focused && selection.is_empty() {
                                let x = if typed { line.x_for_index(caret) } else { px(0.) };
                                let bar = Bounds::new(point(left + x, top), size(px(CARET_WIDTH), line_height));
                                window.paint_quad(fill(bar, caret_ink));
                            }
                        });
                    },
                )
                .size_full(),
            )
            .when(ring, |this| this.child(focus_ring(colors.focus, RADIUS_CONTROL, 1.)))
    }
}
