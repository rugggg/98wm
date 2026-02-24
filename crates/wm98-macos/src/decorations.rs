//! macOS decoration overlay.
//!
//! Since we can't replace the system window server on macOS, the 98wm chrome
//! is drawn using a borderless, transparent `NSWindow` that sits just above
//! each managed window. We render the titlebar/border via tiny-skia then blit
//! the RGBA bytes into the overlay window.
//!
//! CGEventTap is used to intercept mouse clicks on the overlay so we can
//! forward move / close / minimise actions to the underlying window via AX.
//!
//! TODO: implement the NSWindow overlay using objc2 or raw Cocoa bindings.
//!
//! Reference implementations:
//!   • https://github.com/koekeishiya/yabai (C, CGSPrivate)
//!   • https://github.com/nicholasgasior/gsfmt  (overlay approach)

use wm98_core::theme::Theme;

/// Placeholder for a per-window decoration overlay.
pub struct DecorationOverlay {
    pub window_id: u64,
    pub width:     u32,
    pub height:    u32,
}

impl DecorationOverlay {
    pub fn new(window_id: u64, width: u32, height: u32) -> Self {
        Self { window_id, width, height }
    }

    /// Render the titlebar into an RGBA buffer and send it to the overlay NSWindow.
    pub fn update(&self, theme: &Theme, title: &str, focused: bool) {
        let _bytes = theme.render_titlebar(self.width, title, focused);

        // TODO: blit `_bytes` into the CGContext of the overlay NSWindow.
        // Steps:
        //   1. Create NSWindow with NSBorderlessWindowMask + non-opaque + level above content
        //   2. Create NSImageView backed by the RGBA bytes via CGBitmapContext
        //   3. Position the overlay window to match the managed window's frame
        //   4. On each sync tick, call update() to repaint
    }
}
