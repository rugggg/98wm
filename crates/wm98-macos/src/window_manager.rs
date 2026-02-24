//! macOS window manager — uses the Accessibility API to enumerate, move,
//! and resize windows, then overlays 98wm decorations via a transparent
//! borderless window drawn with tiny-skia.
//!
//! Requires: System Settings → Privacy & Security → Accessibility → allow this app.
//!
//! Architecture
//! ────────────
//!   1. Poll running applications (NSWorkspace / CGWindowList).
//!   2. For each window, read its AXPosition / AXSize attributes.
//!   3. Sync logical positions into `wm98_core::layout::FloatingLayout`.
//!   4. On hotkey / click, call `set_window_position` / `set_window_size`.
//!   5. `decorations::OverlayWindow` draws the bubble titlebar on top.

use accessibility::{AXAttribute, AXUIElement};
use core_graphics::window::{
    kCGNullWindowID, kCGWindowListOptionOnScreenOnly, CGWindowListCopyWindowInfo,
};
use std::collections::HashMap;
use std::time::Duration;

use wm98_core::{config::Config, layout::FloatingLayout, theme::Theme};

/// A window tracked by the macOS WM.
#[derive(Debug, Clone)]
pub struct ManagedWindow {
    pub pid:   i32,
    pub title: String,
    pub frame: wm98_core::layout::Rect,
    pub ax:    AXUIElement,
}

pub struct WindowManager {
    pub config:  Config,
    pub theme:   Theme,
    pub layout:  FloatingLayout,
    pub windows: HashMap<u64, ManagedWindow>,
}

impl WindowManager {
    pub fn new(config: Config, theme: Theme) -> anyhow::Result<Self> {
        check_accessibility_permission()?;
        Ok(Self {
            config,
            theme,
            layout:  FloatingLayout::new(),
            windows: HashMap::new(),
        })
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        log::info!("98wm macOS running — press Ctrl-C to exit");

        loop {
            self.sync_windows()?;
            // TODO: process hotkeys via CGEventTap (see decorations.rs)
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    /// Enumerate on-screen windows and update our internal map.
    fn sync_windows(&mut self) -> anyhow::Result<()> {
        let raw_list = unsafe {
            CGWindowListCopyWindowInfo(
                kCGWindowListOptionOnScreenOnly,
                kCGNullWindowID,
            )
        };

        if raw_list.is_null() {
            return Ok(());
        }

        // core-graphics returns a CFArray of CFDictionary.
        // We use the `core-foundation` crate to iterate safely.
        use core_foundation::{
            array::CFArray,
            base::{CFType, TCFType},
            dictionary::CFDictionary,
            string::CFString,
            number::CFNumber,
        };

        let array: CFArray<CFDictionary<CFString, CFType>> =
            unsafe { CFArray::wrap_under_create_rule(raw_list) };

        for entry in array.iter() {
            let pid_key  = CFString::from_static_str("kCGWindowOwnerPID");
            let name_key = CFString::from_static_str("kCGWindowName");

            let pid: i32 = entry
                .find(&pid_key)
                .and_then(|(_, v)| v.downcast::<CFNumber>())
                .and_then(|n| n.to_i32())
                .unwrap_or(0);

            if pid == 0 { continue; }

            let title: String = entry
                .find(&name_key)
                .and_then(|(_, v)| v.downcast::<CFString>())
                .map(|s| s.to_string())
                .unwrap_or_default();

            // Get AX element for this app
            let app_ax = AXUIElement::application(pid);

            // Read position and size via AX attributes
            if let (Ok(pos), Ok(size)) = (
                app_ax.attribute(&AXAttribute::new(&CFString::from_static_str("AXPosition"))),
                app_ax.attribute(&AXAttribute::new(&CFString::from_static_str("AXSize"))),
            ) {
                // TODO: decode pos/size from AXValue (CGPoint / CGSize)
                let _ = (pos, size, title, pid);
            }
        }

        Ok(())
    }

    /// Move a window to (x, y) using the Accessibility API.
    pub fn move_window(&self, pid: i32, x: f64, y: f64) -> anyhow::Result<()> {
        use core_foundation::string::CFString;
        use accessibility::AXAttribute;

        let app = AXUIElement::application(pid);

        // AXPosition takes a CGPoint wrapped in AXValue — use raw AX API:
        // TODO: encode CGPoint as AXValue and call AXUIElementSetAttributeValue.
        // See: https://developer.apple.com/documentation/appkit/nsaccessibility/position
        let _ = (app, x, y);
        Ok(())
    }
}

/// Check that Accessibility permissions have been granted.
fn check_accessibility_permission() -> anyhow::Result<()> {
    // AXIsProcessTrustedWithOptions — prompt if not already granted
    let trusted = unsafe {
        use core_foundation::dictionary::CFDictionary;
        use core_foundation::string::CFString;
        use core_foundation::boolean::CFBoolean;

        extern "C" {
            fn AXIsProcessTrustedWithOptions(options: core_foundation::base::CFTypeRef) -> bool;
        }

        let prompt_key = CFString::from_static_str("AXTrustedCheckOptionPrompt");
        let dict = CFDictionary::from_CFType_pairs(&[(
            prompt_key.as_CFType(),
            CFBoolean::true_value().as_CFType(),
        )]);

        AXIsProcessTrustedWithOptions(dict.as_CFTypeRef())
    };

    if !trusted {
        anyhow::bail!(
            "Accessibility permission not granted. \
             Open System Settings → Privacy & Security → Accessibility and enable 98wm."
        );
    }

    Ok(())
}
