//! Global compositor state — owns every smithay subsystem.
//!
//! All smithay handler traits (`CompositorHandler`, `XdgShellHandler`, …)
//! are implemented on this struct in `handlers.rs`, then wired up with the
//! corresponding `delegate_*!` macros at the bottom of that file.

use smithay::{
    desktop::{PopupManager, Space, Window},
    input::{Seat, SeatState},
    reexports::{
        calloop::LoopHandle,
        wayland_server::{
            backend::{ClientData, ClientId, DisconnectReason},
            Display, DisplayHandle,
        },
    },
    wayland::{
        compositor::{CompositorClientState, CompositorState},
        output::OutputManagerState,
        selection::data_device::DataDeviceState,
        shell::xdg::XdgShellState,
        shm::ShmState,
    },
};

use wm98_core::{config::Config, layout::FloatingLayout, theme::Theme};

// ---------------------------------------------------------------------------
// Per-client data stored by smithay
// ---------------------------------------------------------------------------

pub struct ClientState {
    pub compositor_state: CompositorClientState,
}

impl ClientData for ClientState {
    fn initialized(&self, _id: ClientId) {}
    fn disconnected(&self, _id: ClientId, _reason: DisconnectReason) {}
}

// ---------------------------------------------------------------------------
// Main compositor state
// ---------------------------------------------------------------------------

pub struct Wm98State {
    // ── smithay core ───────────────────────────────────────────────────────
    pub display_handle:       DisplayHandle,
    pub loop_handle:          LoopHandle<'static, Self>,
    pub compositor_state:     CompositorState,
    pub xdg_shell_state:      XdgShellState,
    pub shm_state:            ShmState,
    pub output_manager_state: OutputManagerState,
    pub seat_state:           SeatState<Self>,
    pub data_device_state:    DataDeviceState,

    // ── input ──────────────────────────────────────────────────────────────
    pub seat: Seat<Self>,

    // ── desktop ────────────────────────────────────────────────────────────
    /// The "canvas" that tracks all mapped windows and their positions.
    pub space:  Space<Window>,
    pub popups: PopupManager,

    // ── 98wm ───────────────────────────────────────────────────────────────
    pub layout: FloatingLayout,
    pub config: Config,
    pub theme:  Theme,
}

impl Wm98State {
    pub fn new(
        display: &Display<Self>,
        loop_handle: LoopHandle<'static, Self>,
        config: Config,
        theme: Theme,
    ) -> Self {
        let display_handle = display.handle();

        let compositor_state     = CompositorState::new::<Self>(&display_handle);
        let xdg_shell_state      = XdgShellState::new::<Self>(&display_handle);
        let shm_state            = ShmState::new::<Self>(&display_handle, vec![]);
        let output_manager_state = OutputManagerState::new_with_xdg_output::<Self>(&display_handle);
        let data_device_state    = DataDeviceState::new::<Self>(&display_handle);
        let mut seat_state       = SeatState::new();
        let seat                 = seat_state.new_wl_seat(&display_handle, "seat0");

        Self {
            display_handle,
            loop_handle,
            compositor_state,
            xdg_shell_state,
            shm_state,
            output_manager_state,
            seat_state,
            data_device_state,
            seat,
            space:  Space::default(),
            popups: PopupManager::default(),
            layout: FloatingLayout::new(),
            config,
            theme,
        }
    }
}
