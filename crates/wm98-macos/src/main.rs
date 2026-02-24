//! 98wm — macOS window manager.
//!
//! Uses the macOS Accessibility API to move, resize, and decorate windows
//! (similar approach to yabai / amethyst).
//!
//! Requires the app to be granted Accessibility access:
//!   System Settings → Privacy & Security → Accessibility
//!
//! Run with:
//!   cargo run --bin wm98-macos

fn main() -> anyhow::Result<()> {
    #[cfg(not(target_os = "macos"))]
    {
        eprintln!("wm98-macos is macOS-only.");
        std::process::exit(1);
    }

    #[cfg(target_os = "macos")]
    macos::run()
}

#[cfg(target_os = "macos")]
mod macos {
    pub fn run() -> anyhow::Result<()> {
        env_logger::init();
        log::info!("98wm macOS starting");

        let config = wm98_core::config::Config::load()?;
        let theme  = wm98_core::theme::Theme::default();

        crate::window_manager::WindowManager::new(config, theme)?.run()
    }
}

#[cfg(target_os = "macos")]
mod window_manager;
#[cfg(target_os = "macos")]
mod decorations;
