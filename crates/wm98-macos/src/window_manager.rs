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

use accessibility::AXUIElement;
use core_foundation::{
    base::TCFType,
    boolean::CFBoolean,
    dictionary::CFDictionary,
    string::CFString,
};
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
            // TODO: process hotkeys via CGEventTap
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    /// Enumerate on-screen windows and update our internal map.
    fn sync_windows(&mut self) -> anyhow::Result<()> {
        // CGWindowListCopyWindowInfo returns a CFArrayRef of CFDictionaryRef.
        // We wrap it in a CFArray<CFDictionary<CFString, CFType>> for safe iteration.
        use core_foundation::{
            array::CFArray,
            base::CFType,
            number::CFNumber,
        };

        let raw = unsafe {
            CGWindowListCopyWindowInfo(kCGWindowListOptionOnScreenOnly, kCGNullWindowID)
        };
        if raw.is_null() {
            return Ok(());
        }

        let array: CFArray<CFDictionary<CFString, CFType>> =
            unsafe { CFArray::wrap_under_create_rule(raw as _) };

        let pid_key  = CFString::from_static_string("kCGWindowOwnerPID");
        let name_key = CFString::from_static_string("kCGWindowName");

        for entry in array.iter() {
            let pid: i32 = entry
                .find(pid_key.as_concrete_TypeRef())
                .and_then(|v| v.clone().downcast_into::<CFNumber>())
                .and_then(|n| n.to_i32())
                .unwrap_or(0);

            if pid == 0 {
                continue;
            }

            let _title: String = entry
                .find(name_key.as_concrete_TypeRef())
                .and_then(|v| v.clone().downcast_into::<CFString>())
                .map(|s| s.to_string())
                .unwrap_or_default();

            // TODO: read AXPosition / AXSize for this pid via the AX API,
            // then upsert into self.layout and self.windows.
        }

        Ok(())
    }

    /// Move a window to (x, y) using the Accessibility API.
    ///
    /// CGPoint must be packed into an AXValue before setting — see:
    /// https://developer.apple.com/documentation/appkit/nsaccessibility/position
    pub fn move_window(&self, pid: i32, x: f64, y: f64) -> anyhow::Result<()> {
        let _app = AXUIElement::application(pid);
        // TODO: encode (x, y) as AXValueType::kAXValueCGPointType via
        //       AXValueCreate and call AXUIElementSetAttributeValue.
        let _ = (x, y);
        Ok(())
    }
}

/// Check that Accessibility permissions have been granted; prompt if not.
fn check_accessibility_permission() -> anyhow::Result<()> {
    extern "C" {
        fn AXIsProcessTrustedWithOptions(
            options: core_foundation::base::CFTypeRef,
        ) -> bool;
    }

    let prompt_key = CFString::from_static_string("AXTrustedCheckOptionPrompt");
    let dict = CFDictionary::from_CFType_pairs(&[(
        prompt_key.as_CFType(),
        CFBoolean::true_value().as_CFType(),
    )]);

    let trusted = unsafe { AXIsProcessTrustedWithOptions(dict.as_CFTypeRef()) };

    if !trusted {
        anyhow::bail!(
            "Accessibility permission not granted. \
             Open System Settings → Privacy & Security → Accessibility and enable 98wm."
        );
    }

    Ok(())
}
