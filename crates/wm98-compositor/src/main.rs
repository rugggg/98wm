//! 98wm — Wayland compositor (Linux only).
//!
//! Built on [smithay](https://github.com/Smithay/smithay).
//! For development you can run against an existing display with the winit backend:
//!
//!   DISPLAY=:0 cargo run --bin wm98-compositor
//!
//! For production, run from a TTY — the udev/DRM backend takes over automatically.

fn main() -> anyhow::Result<()> {
    #[cfg(not(target_os = "linux"))]
    {
        eprintln!("wm98-compositor is Linux-only (requires Wayland/DRM).");
        std::process::exit(1);
    }

    #[cfg(target_os = "linux")]
    linux::run()
}

// ─────────────────────────────────────────────────────────────────────────────
// Linux implementation
// ─────────────────────────────────────────────────────────────────────────────
#[cfg(target_os = "linux")]
mod linux {
    pub fn run() -> anyhow::Result<()> {
        env_logger::init();
        log::info!("98wm compositor starting");

        let config = wm98_core::config::Config::load()?;
        let theme  = wm98_core::theme::Theme::default();

        crate::compositor::start(config, theme)
    }
}

#[cfg(target_os = "linux")]
mod compositor;
#[cfg(target_os = "linux")]
mod state;
#[cfg(target_os = "linux")]
mod handlers;
#[cfg(target_os = "linux")]
mod decorations;
#[cfg(target_os = "linux")]
mod input;
