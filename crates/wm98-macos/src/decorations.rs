//! Bubble-aesthetic titlebar overlays for macOS.
//!
//! One borderless NSWindow is created per tracked app window and positioned to
//! cover its titlebar strip.  The background is set to the theme's aqua blue.
//! Traffic-light circle buttons will be added once we wire up CALayer support.
//!
//! The overlay sits at kCGFloatingWindowLevel (3) and ignores mouse events so
//! clicks fall through to the real window underneath.

use std::collections::HashMap;

use objc2::rc::Retained;
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSBackingStoreType, NSColor,
    NSScreen, NSWindow, NSWindowLevel, NSWindowStyleMask,
};
use objc2_foundation::{MainThreadMarker, NSDate, NSPoint, NSRect, NSRunLoop, NSSize};

use wm98_core::theme::Theme;

use crate::window_manager::WindowInfo;

/// Floating window level — sits above normal app windows.
const FLOATING_LEVEL: NSWindowLevel = NSWindowLevel(3);

// ── Public API ───────────────────────────────────────────────────────────────

/// Initialise NSApplication in accessory mode (no Dock icon / menu bar).
/// Call once on the main thread before creating any NSWindows.
pub fn init_app(mtm: MainThreadMarker) {
    let app = NSApplication::sharedApplication(mtm);
    unsafe {
        app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
    }
}

/// Manages all per-window titlebar overlay windows.
pub struct OverlayManager {
    overlays: HashMap<i32, Retained<NSWindow>>, // keyed by pid
}

impl OverlayManager {
    pub fn new() -> Self {
        Self { overlays: HashMap::new() }
    }

    /// Create / reposition / remove overlay windows to match `windows`.
    pub fn sync(&mut self, windows: &[WindowInfo], theme: &Theme, mtm: MainThreadMarker) {
        let screen_h = screen_height(mtm);
        let tb_h = theme.geometry.titlebar_h as f64;

        // Drop overlays whose window has gone away
        let live: std::collections::HashSet<i32> = windows.iter().map(|w| w.pid).collect();
        self.overlays.retain(|pid, _| live.contains(pid));

        for win in windows {
            let Some([x, y, w, _h]) = win.bounds else { continue };
            if w < 1.0 { continue; }

            // CGWindowList uses top-left origin / Y-down; NSWindow uses bottom-left / Y-up
            let ns_y  = screen_h - y - tb_h;
            let frame = NSRect::new(NSPoint::new(x, ns_y), NSSize::new(w, tb_h));

            let overlay = self.overlays.entry(win.pid).or_insert_with(|| {
                make_overlay_window(frame, theme, mtm)
            });

            unsafe {
                overlay.setFrame_display(frame, false);
                overlay.orderFront(None);
            }
        }
    }

    /// Drain pending AppKit events without blocking.
    pub fn pump_events(&self) {
        unsafe {
            NSRunLoop::mainRunLoop().runUntilDate(&NSDate::now());
        }
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn make_overlay_window(frame: NSRect, theme: &Theme, mtm: MainThreadMarker) -> Retained<NSWindow> {
    unsafe {
        let win = NSWindow::initWithContentRect_styleMask_backing_defer(
            mtm.alloc::<NSWindow>(),
            frame,
            NSWindowStyleMask::empty(), // borderless
            NSBackingStoreType::NSBackingStoreBuffered,
            false,
        );

        let c  = &theme.colors.titlebar_hi;
        let bg = NSColor::colorWithRed_green_blue_alpha(
            c.red()   as f64,
            c.green() as f64,
            c.blue()  as f64,
            c.alpha() as f64,
        );

        win.setBackgroundColor(Some(&bg));
        win.setOpaque(false);
        win.setIgnoresMouseEvents(true);
        win.setLevel(FLOATING_LEVEL);
        win.setHasShadow(false);
        win.makeKeyAndOrderFront(None);
        win
    }
}

fn screen_height(mtm: MainThreadMarker) -> f64 {
    unsafe {
        NSScreen::mainScreen(mtm)
            .map(|s| s.frame().size.height)
            .unwrap_or(1080.0)
    }
}
