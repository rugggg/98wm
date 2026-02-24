//! Server-side decoration rendering.
//!
//! For each mapped window we draw a 98wm-style chrome (titlebar + border)
//! directly into the GL framebuffer using smithay's Gles2Renderer.
//!
//! The titlebar pixel data is produced by `wm98_core::theme::Theme::render_titlebar`
//! (tiny-skia RGBA buffer) and uploaded as a GL texture each frame.
//!
//! TODO: cache textures keyed on (window_id, width, focused) so we don't
//!       re-upload on every frame.

use smithay::{
    backend::{
        renderer::{
            damage::OutputDamageTracker,
            element::{
                surface::WaylandSurfaceRenderElement,
                utils::select_dmabuf_feedback,
                AsRenderElements,
            },
            gles::GlesRenderer,
            utils::draw_render_elements,
        },
        winit::WinitGraphicsBackend,
    },
    desktop::{space::SpaceRenderElements, Space, Window},
    output::Output,
    utils::{Physical, Rectangle, Scale},
};

use crate::state::Wm98State;

pub fn render_frame(
    state: &mut Wm98State,
    backend: &WinitGraphicsBackend<GlesRenderer>,
    output: &Output,
) {
    let renderer = backend.renderer();
    let output_geometry = state
        .space
        .output_geometry(output)
        .unwrap_or_else(|| Rectangle::from_loc_and_size((0, 0), (1920, 1080)));

    // TODO: replace with OutputDamageTracker for proper damage tracking
    let _ = renderer; // suppress unused warning until full impl

    // Draw each window's decoration above its surface
    for window in state.space.elements().cloned().collect::<Vec<_>>() {
        let Some(loc) = state.space.element_location(&window) else { continue };
        let geo = window.geometry();
        let focused = state
            .seat
            .get_keyboard()
            .and_then(|kb| kb.current_focus())
            .map(|f| {
                window
                    .toplevel()
                    .wl_surface()
                    == &f
            })
            .unwrap_or(false);

        let title = window
            .toplevel()
            .with_pending_state(|s| s.title.clone())
            .unwrap_or_default();

        let tb_bytes = state
            .theme
            .render_titlebar(geo.size.w as u32, &title, focused);

        // TODO: upload `tb_bytes` as a GlesTexture and draw it at
        //       (loc.x, loc.y - titlebar_h) using renderer.render_texture_at(...)
        let _ = tb_bytes;

        let _ = loc;
    }
}
