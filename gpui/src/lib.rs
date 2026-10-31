//! Dithered, spring-driven components for GPUI. See the repository README for
//! what is covered so far and what is not.

pub mod accordion;
pub mod button;
pub mod calendar;
pub mod chart;
pub mod charts;
pub mod choice;
pub mod combobox;
pub mod dates;
pub mod dialog;
pub mod dither;
pub mod dither_gradient;
pub mod dither_image;
pub mod export;
pub mod extras;
pub mod feedback;
pub mod fields;
pub mod fluid;
pub mod generated;
pub mod list;
pub mod misc;
pub mod nav;
pub mod paint;
pub mod picker;
pub mod scroll_area;
pub mod slider;
pub mod spring;
pub mod structure;
pub mod table;
pub mod tabs;
pub mod text_effects;
pub mod text_field;
pub mod theme;
pub mod tooltip;
pub mod world;

pub use accordion::Accordion;
pub use calendar::{Calendar, DatePicker};
pub use chart::Chart;
pub use dates::Date;
pub use misc::{Avatar, CopyField, Dots, EmptyState, Message, Status, Swatches, Thinking, Uptime};
pub use charts::ChartKind;
pub use dither_image::DitherImage;
pub use extras::{
    Bloom, ColorPicker, Compare, Composer, ComposerEvent, FileCard, Marquee, NotFound, QuestionCard,
    Sidebar, Step, Steps,
};
pub use fields::Field;
pub use fluid::{CheckGroup, HoverList, InputGroup, Kbd, Questions, Reveal, ThinkingSteps};
pub use text_effects::{Count, Dissolve, Scramble, Shimmer};
pub use button::{Button, Look};
pub use choice::{Checkbox, Checked, RadioGroup, Switch};
pub use combobox::{Combobox, ComboboxEvent, CommandEvent, CommandMenu};
pub use dialog::Dialog;
pub use dither::{Gradient, Matrix, Shape};
pub use dither_gradient::{DitherGradient, FrameStats, PaintMode};
pub use feedback::{Divider, Loader, Progress, Skeleton, Sparkline, Stat};
pub use list::Row;
pub use nav::Nav;
pub use picker::{Menu, Select};
pub use scroll_area::ScrollArea;
pub use slider::Slider;
pub use spring::Spring;
pub use structure::{Badge, Banner, Card, Tone};
pub use table::{Align, Table};
pub use tabs::{Tab, Tabs};
pub use text_field::{TextField, TextFieldEvent};
pub use theme::{Density, Mode, Theme};
pub use tooltip::tooltip;

use gpui::{actions, App, KeyBinding};

actions!(fluidither, [FocusNext, FocusPrevious]);

const MOST_TAB_STOPS: usize = 256;

/// Call once at startup: sets the theme from the system and makes Tab move focus.
pub fn init(cx: &mut App) {
    theme::init(cx);
    cx.bind_keys([
        KeyBinding::new("tab", FocusNext, None),
        KeyBinding::new("shift-tab", FocusPrevious, None),
    ]);
    // App-level, because with nothing focused yet an action reaches only the
    // root of the window, not a handler further down the tree.
    cx.on_action(|_: &FocusNext, cx| move_focus(cx, true));
    cx.on_action(|_: &FocusPrevious, cx| move_focus(cx, false));
}

fn move_focus(cx: &mut App, forwards: bool) {
    cx.global_mut::<Theme>().keyboard_input = true;
    let Some(handle) = cx.active_window() else { return };
    // The window is busy dispatching this action, so wait until it is done.
    cx.defer(move |cx| {
        handle
            .update(cx, |_, window, cx| {
                let modal = Theme::get(cx).modal.clone();
                // Tab order covers the whole window, so step past whatever is
                // behind an open dialog.
                for _ in 0..MOST_TAB_STOPS {
                    if forwards {
                        window.focus_next();
                    } else {
                        window.focus_prev();
                    }
                    if modal.as_ref().is_none_or(|modal| modal.contains_focused(window, cx)) {
                        break;
                    }
                }
                window.refresh();
            })
            .ok();
    });
}
