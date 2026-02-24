/// 98wm theme system — early-2000s bubble aesthetic.
///
/// Visual language:
///   • Big, rounded titlebars with a two-stop blue gradient (XP Luna / Aqua)
///   • Traffic-light close/min/max bubbles (glossy highlight on each)
///   • Thick coloured border for the active window
///   • Soft drop shadow
///   • "Wet" button sheen: a small white oval in the upper-left quadrant

use tiny_skia::{Color, FillRule, Paint, PathBuilder, Pixmap, Rect, Transform};

// ---------------------------------------------------------------------------
// Colour palette
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ThemeColors {
    /// Titlebar gradient — bright end (top)
    pub titlebar_hi: Color,
    /// Titlebar gradient — dark end (bottom)
    pub titlebar_lo: Color,
    pub titlebar_text: Color,

    pub border_active: Color,
    pub border_inactive: Color,

    /// Close button (red)
    pub btn_close: Color,
    /// Minimise button (amber)
    pub btn_min: Color,
    /// Maximise button (green)
    pub btn_max: Color,

    pub shadow: Color,
    /// Classic win98-style silver/beige desktop background
    pub desktop_bg: Color,
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self {
            titlebar_hi:      Color::from_rgba8(74,  187, 238, 255), // aqua
            titlebar_lo:      Color::from_rgba8(15,  107, 189, 255), // deep blue
            titlebar_text:    Color::WHITE,
            border_active:    Color::from_rgba8(0,   120, 215, 255),
            border_inactive:  Color::from_rgba8(130, 170, 200, 255),
            btn_close:        Color::from_rgba8(255, 95,  86,  255),
            btn_min:          Color::from_rgba8(255, 189, 46,  255),
            btn_max:          Color::from_rgba8(39,  201, 63,  255),
            shadow:           Color::from_rgba8(0,   0,   0,   70),
            desktop_bg:       Color::from_rgba8(58,  110, 165, 255), // XP default blue
        }
    }
}

// ---------------------------------------------------------------------------
// Geometry
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ThemeGeometry {
    pub titlebar_h: u32,
    pub border_w:   u32,
    pub corner_r:   u32,
    pub button_sz:  u32,
    pub button_gap: u32,
    pub shadow_dx:  i32,
    pub shadow_dy:  i32,
    pub shadow_blur: u32,
}

impl Default for ThemeGeometry {
    fn default() -> Self {
        Self {
            titlebar_h:  36,
            border_w:    3,
            corner_r:    12,
            button_sz:   18,
            button_gap:  7,
            shadow_dx:   4,
            shadow_dy:   6,
            shadow_blur: 18,
        }
    }
}

// ---------------------------------------------------------------------------
// Theme
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct Theme {
    pub colors:   ThemeColors,
    pub geometry: ThemeGeometry,
}

impl Theme {
    /// Render the titlebar strip for a window into an RGBA byte buffer.
    ///
    /// The buffer is `width × titlebar_h` pixels, row-major, RGBA8.
    pub fn render_titlebar(&self, width: u32, _title: &str, focused: bool) -> Vec<u8> {
        let h = self.geometry.titlebar_h;
        let mut pm = Pixmap::new(width, h).expect("titlebar pixmap");

        // ── background (flat approximation of gradient; use shader for real gradient) ──
        let bg = if focused {
            self.colors.titlebar_hi
        } else {
            self.colors.border_inactive
        };
        fill_rect(&mut pm, 0.0, 0.0, width as f32, h as f32, bg);

        // ── lower half slightly darker to fake a gradient ──
        let half = h as f32 / 2.0;
        fill_rect(
            &mut pm,
            0.0, half, width as f32, half,
            darken(bg, 0.82),
        );

        // ── glossy highlight strip (top ~35%) ──
        fill_rect(
            &mut pm,
            0.0, 0.0, width as f32, h as f32 * 0.35,
            Color::from_rgba8(255, 255, 255, 55),
        );

        // ── traffic-light buttons ──
        let bsz  = self.geometry.button_sz;
        let gap  = self.geometry.button_gap;
        let by   = (h / 2) as i32 - (bsz / 2) as i32;
        let bx0  = gap;
        self.draw_bubble(&mut pm, bx0 as f32,             by as f32, bsz, self.colors.btn_close);
        self.draw_bubble(&mut pm, (bx0 + bsz + gap) as f32, by as f32, bsz, self.colors.btn_min);
        self.draw_bubble(&mut pm, (bx0 + (bsz + gap) * 2) as f32, by as f32, bsz, self.colors.btn_max);

        pm.data().to_vec()
    }

    /// Render just a border rectangle (used for window chrome outside the titlebar).
    pub fn render_border(&self, width: u32, height: u32, focused: bool) -> Vec<u8> {
        let mut pm = Pixmap::new(width, height).expect("border pixmap");
        let color = if focused {
            self.colors.border_active
        } else {
            self.colors.border_inactive
        };
        fill_rect(&mut pm, 0.0, 0.0, width as f32, height as f32, color);
        pm.data().to_vec()
    }

    // ── helpers ──────────────────────────────────────────────────────────────

    fn draw_bubble(&self, pm: &mut Pixmap, x: f32, y: f32, size: u32, color: Color) {
        let sz = size as f32;
        let r  = sz / 2.0;
        let cx = x + r;
        let cy = y + r;

        // Main circle
        let mut paint = Paint::default();
        paint.set_color(color);
        paint.anti_alias = true;

        let circle = {
            let mut pb = PathBuilder::new();
            pb.push_oval(Rect::from_xywh(x, y, sz, sz).unwrap());
            pb.finish().unwrap()
        };
        pm.fill_path(&circle, &paint, FillRule::Winding, Transform::identity(), None);

        // Gloss: small white oval in upper-left quadrant
        let mut gloss = Paint::default();
        gloss.set_color(Color::from_rgba8(255, 255, 255, 120));
        gloss.anti_alias = true;
        let gw = sz * 0.42;
        let gh = sz * 0.28;
        let gx = cx - r * 0.52 - gw / 2.0;
        let gy = cy - r * 0.55 - gh / 2.0;

        let shine = {
            let mut pb = PathBuilder::new();
            pb.push_oval(Rect::from_xywh(gx, gy, gw, gh).unwrap());
            pb.finish().unwrap()
        };
        pm.fill_path(&shine, &gloss, FillRule::Winding, Transform::identity(), None);
    }
}

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

fn fill_rect(pm: &mut Pixmap, x: f32, y: f32, w: f32, h: f32, color: Color) {
    let Some(rect) = Rect::from_xywh(x, y, w, h) else { return };
    let mut paint = Paint::default();
    paint.set_color(color);
    pm.fill_rect(rect, &paint, Transform::identity(), None);
}

/// Multiply each RGB channel by `factor` (< 1 darkens).
fn darken(c: Color, factor: f32) -> Color {
    Color::from_rgba(
        (c.red()   * factor).min(1.0),
        (c.green() * factor).min(1.0),
        (c.blue()  * factor).min(1.0),
        c.alpha(),
    )
    .unwrap_or(c)
}
