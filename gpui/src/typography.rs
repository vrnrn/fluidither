//! GPUI 0.2.2 selects named faces without instantiating variable weight axes.
//! Use the upstream static OTFs so regular, strong and italic text stay distinct.

use gpui::App;
use std::borrow::Cow;

const FONTS: &[&[u8]] = &[
    include_bytes!("fonts/obel-mono/ObelMono-Thin.otf"),
    include_bytes!("fonts/obel-mono/ObelMono-ThinItalic.otf"),
    include_bytes!("fonts/obel-mono/ObelMono-ExtraLight.otf"),
    include_bytes!("fonts/obel-mono/ObelMono-ExtraLightItalic.otf"),
    include_bytes!("fonts/obel-mono/ObelMono-Light.otf"),
    include_bytes!("fonts/obel-mono/ObelMono-LightItalic.otf"),
    include_bytes!("fonts/obel-mono/ObelMono-Regular.otf"),
    include_bytes!("fonts/obel-mono/ObelMono-Italic.otf"),
    include_bytes!("fonts/obel-mono/ObelMono-Medium.otf"),
    include_bytes!("fonts/obel-mono/ObelMono-MediumItalic.otf"),
    include_bytes!("fonts/obel-mono/ObelMono-SemiBold.otf"),
    include_bytes!("fonts/obel-mono/ObelMono-SemiBoldItalic.otf"),
    include_bytes!("fonts/obel-mono/ObelMono-Bold.otf"),
    include_bytes!("fonts/obel-mono/ObelMono-BoldItalic.otf"),
    include_bytes!("fonts/obel-mono/ObelMono-ExtraBold.otf"),
    include_bytes!("fonts/obel-mono/ObelMono-ExtraBoldItalic.otf"),
    include_bytes!("fonts/obel-mono/ObelMono-Black.otf"),
    include_bytes!("fonts/obel-mono/ObelMono-BlackItalic.otf"),
];

pub(crate) fn init(cx: &mut App) {
    if let Err(error) = cx
        .text_system()
        .add_fonts(FONTS.iter().map(|bytes| Cow::Borrowed(*bytes)).collect())
    {
        eprintln!("fluidither: could not load bundled Obel Mono: {error}");
    }
}
