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

/// Lightweight snapshot returned by each sync pass.
#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub pid:   i32,
    pub title: String,
}

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
        println!("98wm macOS — press Ctrl-C to exit\n");

        let mut tick: u64 = 0;
        loop {
            let windows = self.sync_windows()?;

            // Print the window list every second (every 10 ticks × 100 ms)
            if tick % 10 == 0 {
                println!("--- tick {} — {} window(s) ---", tick, windows.len());
                for w in &windows {
                    println!("  [{:>6}]  {}", w.pid, w.title);
                }
            }

            tick += 1;
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    /// Enumerate on-screen windows and return the current list.
    fn sync_windows(&mut self) -> anyhow::Result<Vec<WindowInfo>> {
        use core_foundation::{
            array::CFArray,
            base::CFType,
            number::CFNumber,
        };

        let raw = unsafe {
            CGWindowListCopyWindowInfo(kCGWindowListOptionOnScreenOnly, kCGNullWindowID)
        };
        if raw.is_null() {
            return Ok(vec![]);
        }

        let array: CFArray<CFDictionary<CFString, CFType>> =
            unsafe { CFArray::wrap_under_create_rule(raw as _) };

        let pid_key   = CFString::from_static_string("kCGWindowOwnerPID");
        let name_key  = CFString::from_static_string("kCGWindowOwnerName");
        let title_key = CFString::from_static_string("kCGWindowName");
        let layer_key = CFString::from_static_string("kCGWindowLayer");

        let mut seen = Vec::new();

        for entry in array.iter() {
            let layer: i32 = entry
                .find(layer_key.as_concrete_TypeRef())
                .and_then(|v| v.clone().downcast_into::<CFNumber>())
                .and_then(|n| n.to_i32())
                .unwrap_or(0);

            // Skip background/desktop layers — only normal app windows (layer 0)
            if layer != 0 {
                continue;
            }

            let pid: i32 = entry
                .find(pid_key.as_concrete_TypeRef())
                .and_then(|v| v.clone().downcast_into::<CFNumber>())
                .and_then(|n| n.to_i32())
                .unwrap_or(0);

            if pid == 0 {
                continue;
            }

            let app: String = entry
                .find(name_key.as_concrete_TypeRef())
                .and_then(|v| v.clone().downcast_into::<CFString>())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "<unknown>".into());

            let title: String = entry
                .find(title_key.as_concrete_TypeRef())
                .and_then(|v| v.clone().downcast_into::<CFString>())
                .map(|s| s.to_string())
                .unwrap_or_default();

            let label = if title.is_empty() { app } else { format!("{title}") };

            seen.push(WindowInfo { pid, title: label });
        }

        Ok(seen)
    }

    /// Move a window to (x, y) using the Accessibility API.
    ///
    /// CGPoint must be packed into an AXValue before setting — see:
    /// https://developer.apple.com/documentation/appkit/nsaccessibility/position
    pub fn move_window(&self, _pid: i32, _x: f64, _y: f64) -> anyhow::Result<()> {
        // TODO: encode (x, y) as AXValueType::kAXValueCGPointType via
        //       AXValueCreate and call AXUIElementSetAttributeValue on the app's AXUIElement.
        Ok(())
    }
}

/// Check that Accessibility permissions have been granted; prompt if not.
fn check_accessibility_permission() -> anyhow::Result<()> {
    #[link(name = "ApplicationServices", kind = "framework")]
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
