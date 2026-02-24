//! Implement smithay's handler traits for `Wm98State` and wire them up with
//! the corresponding `delegate_*!` macros.

use smithay::{
    delegate_compositor, delegate_data_device, delegate_output, delegate_seat,
    delegate_shm, delegate_xdg_shell,
    desktop::{Space, Window},
    input::{pointer::CursorImageStatus, Seat, SeatHandler, SeatState},
    reexports::wayland_server::{
        protocol::{wl_buffer::WlBuffer, wl_seat::WlSeat, wl_surface::WlSurface},
        Client, Resource,
    },
    utils::{Logical, Point, Serial},
    wayland::{
        buffer::BufferHandler,
        compositor::{
            get_parent, is_sync_subsurface, CompositorClientState, CompositorHandler,
            CompositorState,
        },
        output::OutputHandler,
        selection::{
            data_device::{
                ClientDndGrabHandler, DataDeviceHandler, DataDeviceState,
                ServerDndGrabHandler,
            },
            SelectionHandler,
        },
        shell::xdg::{
            PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState,
            XdgToplevelSurfaceData,
        },
        shm::{ShmHandler, ShmState},
    },
};

use crate::state::{ClientState, Wm98State};

// ─────────────────────────────────────────────────────────────────────────────
// CompositorHandler
// ─────────────────────────────────────────────────────────────────────────────

impl CompositorHandler for Wm98State {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        &client.get_data::<ClientState>().unwrap().compositor_state
    }

    fn commit(&mut self, surface: &WlSurface) {
        smithay::backend::renderer::utils::on_commit_buffer_handler::<Self>(surface);

        // Propagate commit to smithay's desktop helpers
        smithay::desktop::utils::surface_presentation_feedback_flags_from_states(surface, &Default::default());
        self.popups.commit(surface);

        // If this is a new top-level, map it into the space
        if let Some(window) = self
            .space
            .elements()
            .find(|w| w.toplevel().wl_surface() == surface)
            .cloned()
        {
            self.space.raise_element(&window, true);
        }
    }
}

delegate_compositor!(Wm98State);

// ─────────────────────────────────────────────────────────────────────────────
// BufferHandler (required by CompositorHandler blanket)
// ─────────────────────────────────────────────────────────────────────────────

impl BufferHandler for Wm98State {
    fn buffer_destroyed(&mut self, _buffer: &WlBuffer) {}
}

// ─────────────────────────────────────────────────────────────────────────────
// ShmHandler
// ─────────────────────────────────────────────────────────────────────────────

impl ShmHandler for Wm98State {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}

delegate_shm!(Wm98State);

// ─────────────────────────────────────────────────────────────────────────────
// XdgShellHandler — creates / destroys windows
// ─────────────────────────────────────────────────────────────────────────────

impl XdgShellHandler for Wm98State {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        let window = Window::new_wayland_window(surface);
        let (sw, sh) = self
            .space
            .outputs()
            .next()
            .and_then(|o| o.current_mode())
            .map(|m| (m.size.w as u32, m.size.h as u32))
            .unwrap_or((1920, 1080));

        let title = {
            let data = window.toplevel().with_pending_state(|s| s.title.clone());
            data.unwrap_or_default()
        };

        let _id = self.layout.add(title, sw, sh);
        let pos: Point<i32, Logical> = (
            self.layout.focused().map(|w| w.rect.x).unwrap_or(60),
            self.layout.focused().map(|w| w.rect.y).unwrap_or(60),
        )
            .into();

        self.space.map_element(window, pos, true);
    }

    fn toplevel_destroyed(&mut self, surface: ToplevelSurface) {
        let Some(window) = self
            .space
            .elements()
            .find(|w| w.toplevel().wl_surface() == surface.wl_surface())
            .cloned()
        else {
            return;
        };
        self.space.unmap_elem(&window);
    }

    fn new_popup(&mut self, surface: PopupSurface, _positioner: PositionerState) {
        self.popups.track_popup(surface.into()).ok();
    }

    fn grab(&mut self, _surface: PopupSurface, _seat: WlSeat, _serial: Serial) {}
}

delegate_xdg_shell!(Wm98State);

// ─────────────────────────────────────────────────────────────────────────────
// SeatHandler / input
// ─────────────────────────────────────────────────────────────────────────────

impl SeatHandler for Wm98State {
    type KeyboardFocus = WlSurface;
    type PointerFocus  = WlSurface;
    type TouchFocus    = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }

    fn cursor_image(&mut self, _seat: &Seat<Self>, _image: CursorImageStatus) {}

    fn focus_changed(&mut self, _seat: &Seat<Self>, _focused: Option<&WlSurface>) {}
}

delegate_seat!(Wm98State);

// ─────────────────────────────────────────────────────────────────────────────
// DataDevice (clipboard / drag-and-drop)
// ─────────────────────────────────────────────────────────────────────────────

impl SelectionHandler for Wm98State {
    type SelectionUserData = ();
}

impl DataDeviceHandler for Wm98State {
    fn data_device_state(&self) -> &DataDeviceState {
        &self.data_device_state
    }
}

impl ClientDndGrabHandler for Wm98State {}
impl ServerDndGrabHandler for Wm98State {}

delegate_data_device!(Wm98State);

// ─────────────────────────────────────────────────────────────────────────────
// Output
// ─────────────────────────────────────────────────────────────────────────────

impl OutputHandler for Wm98State {}

delegate_output!(Wm98State);
