//! The theme global, density tokens and the color helpers components share.

use crate::generated::{
    SizeTokens, ThemeColors, DARK, LIGHT, SIZE_COMPACT, SIZE_REGULAR, SURFACE_SHADOW,
};
use gpui::{point, px, App, BoxShadow, FocusHandle, Global, Hsla, Pixels, Rgba, Window, WindowAppearance};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Light,
    Dark,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Density {
    #[default]
    Regular,
    Compact,
}

impl Density {
    pub fn tokens(self) -> SizeTokens {
        match self {
            Density::Regular => SIZE_REGULAR,
            Density::Compact => SIZE_COMPACT,
        }
    }
}

pub struct Theme {
    pub mode: Mode,
    /// Skip springs and animation. Read from the system at `init`.
    pub reduced_motion: bool,
    /// Whether the last input was the keyboard. Focus rings only show then.
    pub keyboard_input: bool,
    /// The open dialog, which Tab may not leave.
    pub modal: Option<FocusHandle>,
}

impl Global for Theme {}

impl Theme {
    pub fn get(cx: &App) -> &Theme {
        cx.global::<Theme>()
    }

    pub fn colors(&self) -> &'static ThemeColors {
        match self.mode {
            Mode::Light => &LIGHT,
            Mode::Dark => &DARK,
        }
    }

    /// A one-pixel ring plus the drop shadow for a surface level, 0 to 3.
    pub fn shadow(&self, level: usize) -> Vec<BoxShadow> {
        let colors = self.colors();
        let ring = BoxShadow {
            color: Hsla::from(colors.line),
            offset: point(px(0.), px(0.)),
            blur_radius: px(0.),
            spread_radius: px(1.),
        };
        let (y, blur) = SURFACE_SHADOW[level];
        if y == 0.0 {
            return vec![ring];
        }
        let drop = BoxShadow {
            color: Hsla::from(colors.shadow),
            offset: point(px(0.), px(y)),
            blur_radius: px(blur),
            spread_radius: px(0.),
        };
        vec![ring, drop]
    }
}

pub(crate) fn init(cx: &mut App) {
    let mode = match cx.window_appearance() {
        WindowAppearance::Dark | WindowAppearance::VibrantDark => Mode::Dark,
        WindowAppearance::Light | WindowAppearance::VibrantLight => Mode::Light,
    };
    cx.set_global(Theme { mode, reduced_motion: system_reduced_motion(), keyboard_input: false, modal: None });
}

#[cfg(target_os = "macos")]
#[allow(unexpected_cfgs)] // The objc macros test a cfg of their own.
fn system_reduced_motion() -> bool {
    use objc::runtime::{Object, BOOL, NO};
    use objc::{class, msg_send, sel, sel_impl};
    // SAFETY: both are documented, side-effect-free AppKit getters.
    unsafe {
        let workspace: *mut Object = msg_send![class!(NSWorkspace), sharedWorkspace];
        let reduced: BOOL = msg_send![workspace, accessibilityDisplayShouldReduceMotion];
        reduced != NO
    }
}

#[cfg(not(target_os = "macos"))]
fn system_reduced_motion() -> bool {
    false
}

/// Blend in sRGB, `amount` of the way from `a` to `b`.
pub fn mix(a: Rgba, b: Rgba, amount: f32) -> Rgba {
    let blend = |x: f32, y: f32| x + (y - x) * amount;
    Rgba { r: blend(a.r, b.r), g: blend(a.g, b.g), b: blend(a.b, b.b), a: blend(a.a, b.a) }
}

pub fn fade(color: Rgba, alpha: f32) -> Rgba {
    Rgba { a: color.a * alpha, ..color }
}

/// How far to move text down so its capitals, not its line box, sit in the middle.
/// Negative moves it up. Pure so it can be tested without a font.
pub fn cap_nudge(ascent: f32, descent: f32, cap_height: f32) -> f32 {
    // A line box centers ascent + descent, which puts the middle of the capitals
    // this far below the middle of the line.
    let caps_below_middle = (ascent - descent - cap_height) / 2.0;
    -caps_below_middle
}

/// `cap_nudge` for the font and size in effect here, snapped to device pixels so
/// text stays crisp.
pub fn cap_offset(window: &Window, font_size: Pixels) -> Pixels {
    let font = window.text_style().font();
    let text = window.text_system();
    let id = text.resolve_font(&font);
    let nudge = cap_nudge(
        f32::from(text.ascent(id, font_size)),
        f32::from(text.descent(id, font_size)).abs(),
        f32::from(text.cap_height(id, font_size)),
    );
    let scale = window.scale_factor();
    px((nudge * scale).round() / scale)
}

