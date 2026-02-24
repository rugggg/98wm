//! Entry point: set up the event loop, pick a backend, and run.

use anyhow::Context;
use smithay::{
    backend::winit::{self, WinitEvent},
    output::{Mode, Output, PhysicalProperties, Subpixel},
    reexports::{
        calloop::{EventLoop, generic::Generic, Interest, Mode as PollMode, PostAction},
        wayland_server::{Display, socket::ListeningSocketSource},
    },
    utils::{Rectangle, Transform},
};

use wm98_core::{config::Config, theme::Theme};
use crate::state::{ClientState, Wm98State};

pub fn start(config: Config, theme: Theme) -> anyhow::Result<()> {
    let mut event_loop: EventLoop<Wm98State> =
        EventLoop::try_new().context("create event loop")?;

    let mut display: Display<Wm98State> = Display::new().context("create wayland display")?;

    // ── Wayland socket ──────────────────────────────────────────────────────
    let listening_socket = ListeningSocketSource::new_auto()
        .context("bind wayland socket")?;
    let socket_name = listening_socket.socket_name().to_os_string();

    event_loop
        .handle()
        .insert_source(listening_socket, move |client_stream, _, state| {
            state
                .display_handle
                .insert_client(client_stream, std::sync::Arc::new(ClientState {
                    compositor_state: Default::default(),
                }))
                .expect("insert client");
        })
        .context("insert socket source")?;

    // ── Flush display source ────────────────────────────────────────────────
    event_loop
        .handle()
        .insert_source(
            Generic::new(
                display.backend().poll_fd().try_clone_to_owned().unwrap(),
                Interest::READ,
                PollMode::Level,
            ),
            |_, _, state| {
                state.display_handle.backend_handle().flush_clients().ok();
                Ok(PostAction::Continue)
            },
        )
        .context("insert display flush source")?;

    let loop_handle = event_loop.handle();
    let mut state = Wm98State::new(&display, loop_handle, config, theme);

    // ── Winit backend (runs inside an existing desktop for dev/testing) ─────
    // TODO: detect whether we're on a TTY and use the udev/DRM backend instead.
    let (mut winit_backend, mut winit_evt_loop) =
        winit::init().context("init winit backend")?;

    // Create a virtual output matching the winit window
    let output = Output::new(
        "winit".into(),
        PhysicalProperties {
            size: (0, 0).into(),
            subpixel: Subpixel::Unknown,
            make: "98wm".into(),
            model: "virtual".into(),
        },
    );
    let mode = Mode {
        size: winit_backend.window_size(),
        refresh: 60_000,
    };
    output.change_current_state(Some(mode), Some(Transform::Normal), None, Some((0, 0).into()));
    output.set_preferred(mode);
    state.space.map_output(&output, (0, 0));

    std::env::set_var("WAYLAND_DISPLAY", &socket_name);
    log::info!("Wayland socket: {:?}", socket_name);

    // ── Main loop ───────────────────────────────────────────────────────────
    loop {
        winit_evt_loop
            .dispatch_new_events(|event| match event {
                WinitEvent::Resized { size, .. } => {
                    let mode = Mode { size, refresh: 60_000 };
                    output.change_current_state(Some(mode), None, None, None);
                }
                WinitEvent::Input(input) => {
                    crate::input::handle_input(&mut state, input);
                }
                WinitEvent::CloseRequested => {
                    log::info!("Window closed, exiting.");
                    std::process::exit(0);
                }
                _ => {}
            })
            .ok();

        // Render frame
        winit_backend
            .bind()
            .ok()
            .and_then(|_| {
                crate::decorations::render_frame(&mut state, &winit_backend, &output);
                winit_backend.submit(None).ok()
            });

        // Flush Wayland clients
        display.flush_clients().ok();

        event_loop
            .dispatch(Some(std::time::Duration::from_millis(1)), &mut state)
            .context("dispatch event loop")?;
    }
}
