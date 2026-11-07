//! The few places where crates.io gpui (feature `zed`) and gpui-pre (feature
//! `pre`) differ. Everything else is written once against both.

use gpui::{Bounds, Corners, Pixels, RenderImage, Window};
use std::sync::Arc;

#[cfg(feature = "pre")]
pub use gpui::Anchor as Corner;
#[cfg(feature = "zed")]
pub use gpui::Corner;

/// Paints an image filling `bounds`.
pub fn paint_image(
    window: &mut Window,
    bounds: Bounds<Pixels>,
    corners: Corners<Pixels>,
    image: Arc<RenderImage>,
) -> gpui::Result<()> {
    #[cfg(feature = "zed")]
    return window.paint_image(bounds, corners, image, 0, false);
    #[cfg(feature = "pre")]
    return window.paint_image(bounds, bounds, corners, image, 0, false);
}

/// The result of `AsyncApp::update`, or `fallback` when the app has gone.
#[cfg(feature = "zed")]
pub fn settled<R>(result: gpui::Result<R>, fallback: R) -> R {
    result.unwrap_or(fallback)
}
#[cfg(feature = "pre")]
pub fn settled<R>(result: R, _fallback: R) -> R {
    result
}
