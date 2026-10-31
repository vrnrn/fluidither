//! Saves this process's own window to a PNG, for demo proofs.

use gpui::{AnyWindowHandle, AsyncApp, Keystroke, Timer};
use objc::runtime::Object;
use objc::{class, msg_send, sel, sel_impl};
use std::ffi::c_void;
use std::time::Duration;

/// For proofs of live states. FD_POINTER=x,y moves the pointer, FD_CLICKS="x,y x,y"
/// clicks, FD_KEYS="tab right" presses keys, and FD_CAPTURE=path.png saves the window
/// and quits. Positions are window content points from the top left.
#[allow(dead_code)]
pub async fn run_script(window: AnyWindowHandle, cx: &mut AsyncApp) {
    let Ok(path) = std::env::var("FD_CAPTURE") else { return };
    Timer::after(Duration::from_millis(600)).await;
    let spot = |text: &str| {
        let (x, y) = text.split_once(',').expect("a position is x,y");
        (x.parse::<f64>().unwrap(), y.parse::<f64>().unwrap())
    };
    for text in std::env::var("FD_CLICKS").unwrap_or_default().split_whitespace() {
        let (x, y) = spot(text);
        click(x, y);
        Timer::after(Duration::from_millis(250)).await;
    }
    if let Ok(text) = std::env::var("FD_POINTER") {
        let (x, y) = spot(&text);
        move_pointer(x, y);
    }
    for key in std::env::var("FD_KEYS").unwrap_or_default().split_whitespace() {
        let keystroke = Keystroke::parse(key).unwrap();
        // An untyped handle, because a typed one locks the root view and a change
        // handler may update it.
        window
            .update(cx, |_, window, cx| {
                window.dispatch_keystroke(keystroke, cx);
            })
            .ok();
        Timer::after(Duration::from_millis(60)).await;
    }
    Timer::after(Duration::from_millis(600)).await;
    match save_window(&path) {
        Ok(()) => eprintln!("saved {path}"),
        Err(error) => eprintln!("capture failed: {error}"),
    }
    cx.update(|cx| cx.quit()).ok();
}

#[repr(C)]
#[derive(Clone, Copy)]
struct CgRect {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGWindowListCreateImage(rect: CgRect, list: u32, window: u32, options: u32) -> *const c_void;
    fn CGImageGetWidth(image: *const c_void) -> usize;
    fn CGImageGetHeight(image: *const c_void) -> usize;
    fn CGImageGetBytesPerRow(image: *const c_void) -> usize;
    fn CGImageGetDataProvider(image: *const c_void) -> *const c_void;
    fn CGDataProviderCopyData(provider: *const c_void) -> *const c_void;
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFDataGetBytePtr(data: *const c_void) -> *const u8;
    fn CFDataGetLength(data: *const c_void) -> isize;
}

#[repr(C)]
#[derive(Clone, Copy)]
struct NsPoint {
    x: f64,
    y: f64,
}

#[allow(unexpected_cfgs)]
fn first_window() -> Option<*mut Object> {
    // SAFETY: documented AppKit getters, called on the main thread.
    unsafe {
        let app: *mut Object = msg_send![class!(NSApplication), sharedApplication];
        let windows: *mut Object = msg_send![app, windows];
        let count: usize = msg_send![windows, count];
        (count > 0).then(|| msg_send![windows, objectAtIndex: 0usize])
    }
}

#[allow(unexpected_cfgs)]
fn window_number() -> Option<u32> {
    let window = first_window()?;
    // SAFETY: a documented AppKit getter on a live window.
    let number: isize = unsafe { msg_send![window, windowNumber] };
    Some(number as u32)
}

/// Sends a pointer move to our own window, at a position in window content
/// points from the top left. Nothing outside this process sees it.
#[allow(dead_code)]
pub fn move_pointer(x: f64, y: f64) {
    send_mouse(5, x, y, 0, 0.0);
}

/// A left click at the same kind of position.
#[allow(dead_code)]
pub fn click(x: f64, y: f64) {
    send_mouse(5, x, y, 0, 0.0);
    send_mouse(1, x, y, 1, 1.0);
    send_mouse(2, x, y, 1, 0.0);
}

#[allow(dead_code, unexpected_cfgs)]
fn send_mouse(kind: usize, x: f64, y: f64, clicks: isize, pressure: f32) {
    let Some(window) = first_window() else { return };
    // SAFETY: the selector and argument types match NSEvent's documented factory.
    unsafe {
        let view: *mut Object = msg_send![window, contentView];
        let bounds: CgRect = msg_send![view, bounds];
        let number: isize = msg_send![window, windowNumber];
        let location = NsPoint { x, y: bounds.height - y };
        let nil: *mut Object = std::ptr::null_mut();
        let event: *mut Object = msg_send![class!(NSEvent),
            mouseEventWithType: kind
            location: location
            modifierFlags: 0usize
            timestamp: 0f64
            windowNumber: number
            context: nil
            eventNumber: 0isize
            clickCount: clicks
            pressure: pressure];
        let _: () = msg_send![window, sendEvent: event];
    }
}

/// The window as RGBA rows, and the height of its title bar in image pixels.
#[allow(dead_code, unexpected_cfgs)]
pub fn window_pixels() -> Result<(usize, usize, Vec<u8>, usize), String> {
    let number = window_number().ok_or("no window")?;
    let window = first_window().ok_or("no window")?;
    let null_rect = CgRect { x: f64::INFINITY, y: f64::INFINITY, width: 0.0, height: 0.0 };
    // SAFETY: as in `save_window`.
    unsafe {
        let view: *mut Object = msg_send![window, contentView];
        let bounds: CgRect = msg_send![view, bounds];
        let scale: f64 = msg_send![window, backingScaleFactor];
        let image = CGWindowListCreateImage(null_rect, 8, number, 1 | 8);
        if image.is_null() {
            return Err("CGWindowListCreateImage returned nothing".into());
        }
        let (width, height) = (CGImageGetWidth(image), CGImageGetHeight(image));
        let stride = CGImageGetBytesPerRow(image);
        let data = CGDataProviderCopyData(CGImageGetDataProvider(image));
        let bytes = std::slice::from_raw_parts(CFDataGetBytePtr(data), CFDataGetLength(data) as usize);
        let mut rgba = Vec::with_capacity(width * height * 4);
        for row in bytes.chunks_exact(stride).take(height) {
            for pixel in row[..width * 4].chunks_exact(4) {
                rgba.extend_from_slice(&[pixel[2], pixel[1], pixel[0], 255]);
            }
        }
        let title_bar = height.saturating_sub((bounds.height * scale).round() as usize);
        Ok((width, height, rgba, title_bar))
    }
}

/// Returns an error string when the system refuses the capture.
#[allow(dead_code)]
pub fn save_window(path: &str) -> Result<(), String> {
    let number = window_number().ok_or("no window")?;
    let null_rect = CgRect { x: f64::INFINITY, y: f64::INFINITY, width: 0.0, height: 0.0 };
    // SAFETY: the pointers come straight from CoreGraphics and are read within
    // the lengths it reports. The image and data leak, which is fine for a tool
    // that exits right after.
    unsafe {
        let image = CGWindowListCreateImage(null_rect, 8, number, 1 | 8);
        if image.is_null() {
            return Err("CGWindowListCreateImage returned nothing".into());
        }
        let (width, height) = (CGImageGetWidth(image), CGImageGetHeight(image));
        let stride = CGImageGetBytesPerRow(image);
        let data = CGDataProviderCopyData(CGImageGetDataProvider(image));
        let bytes = std::slice::from_raw_parts(CFDataGetBytePtr(data), CFDataGetLength(data) as usize);
        let mut rgba = Vec::with_capacity(width * height * 4);
        for row in bytes.chunks_exact(stride).take(height) {
            for pixel in row[..width * 4].chunks_exact(4) {
                rgba.extend_from_slice(&[pixel[2], pixel[1], pixel[0], 255]);
            }
        }
        image::save_buffer(path, &rgba, width as u32, height as u32, image::ColorType::Rgba8)
            .map_err(|error| error.to_string())
    }
}
