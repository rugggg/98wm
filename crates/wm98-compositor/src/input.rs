//! Keyboard and pointer input handling.

use smithay::{
    backend::input::{
        AbsolutePositionEvent, ButtonState, Event, InputEvent, KeyState,
        KeyboardKeyEvent, PointerButtonEvent, PointerMotionEvent,
        WinitInput,
    },
    input::{
        keyboard::{keysyms, FilterResult},
        pointer::{ButtonEvent, MotionEvent},
    },
    utils::{Logical, Point, SERIAL_COUNTER},
    reexports::wayland_server::protocol::wl_pointer,
};

use crate::state::Wm98State;

/// Dispatch a raw winit input event into the compositor.
pub fn handle_input(state: &mut Wm98State, event: InputEvent<WinitInput>) {
    match event {
        InputEvent::Keyboard { event } => handle_key(state, event),
        InputEvent::PointerMotion { event } => handle_pointer_motion(state, event),
        InputEvent::PointerButton { event } => handle_pointer_button(state, event),
        _ => {}
    }
}

fn handle_key(state: &mut Wm98State, event: impl KeyboardKeyEvent<WinitInput>) {
    let Some(kb) = state.seat.get_keyboard() else { return };
    let serial = SERIAL_COUNTER.next_serial();
    let time   = event.time_msec();
    let key    = event.key_code();

    kb.input::<(), _>(
        state,
        key,
        event.state(),
        serial,
        time,
        |state, modifiers, keysym| {
            if event.state() != KeyState::Pressed {
                return FilterResult::Forward;
            }

            // Super + Q — close focused window
            if modifiers.logo && keysym.modified_sym() == keysyms::KEY_q {
                if let Some(window) = state
                    .space
                    .elements()
                    .find(|w| {
                        state
                            .seat
                            .get_keyboard()
                            .and_then(|kb| kb.current_focus())
                            .map(|f| w.toplevel().wl_surface() == &f)
                            .unwrap_or(false)
                    })
                    .cloned()
                {
                    window.toplevel().send_close();
                }
                return FilterResult::Intercept(());
            }

            // Super + Shift + Q — quit compositor
            if modifiers.logo && modifiers.shift && keysym.modified_sym() == keysyms::KEY_q {
                log::info!("quit requested");
                std::process::exit(0);
            }

            // Super + Return — launch terminal
            if modifiers.logo && keysym.modified_sym() == keysyms::KEY_Return {
                let term = state
                    .config
                    .keybinds
                    .iter()
                    .find(|kb| kb.key == "Return")
                    .and_then(|kb| kb.action.strip_prefix("spawn ").map(str::to_owned))
                    .unwrap_or_else(|| "alacritty".into());

                std::process::Command::new(&term).spawn().ok();
                return FilterResult::Intercept(());
            }

            FilterResult::Forward
        },
    );
}

fn handle_pointer_motion(state: &mut Wm98State, event: impl PointerMotionEvent<WinitInput>) {
    let Some(ptr) = state.seat.get_pointer() else { return };
    let serial = SERIAL_COUNTER.next_serial();

    // TODO: translate delta to absolute position, hit-test against windows,
    //       and handle window drag (move) when clicking a titlebar.
    let _ = (ptr, serial, event);
}

fn handle_pointer_button(state: &mut Wm98State, event: impl PointerButtonEvent<WinitInput>) {
    let Some(ptr) = state.seat.get_pointer() else { return };
    let serial = SERIAL_COUNTER.next_serial();

    // Focus the window under the cursor on left-click
    if event.button_code() == 0x110 /* BTN_LEFT */ && event.state() == ButtonState::Pressed {
        // TODO: resolve cursor position → window, update keyboard focus
    }

    let _ = (ptr, serial);
}
