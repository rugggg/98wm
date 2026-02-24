/// Platform-agnostic floating layout manager.
/// Both the Wayland compositor and the macOS AX-API manager use this to
/// track logical window positions before translating to their respective APIs.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

#[derive(Debug, Clone)]
pub struct WindowState {
    pub id: u64,
    pub title: String,
    pub rect: Rect,
    pub focused: bool,
    pub minimized: bool,
    pub maximized: bool,
}

#[derive(Debug, Default)]
pub struct FloatingLayout {
    pub windows: Vec<WindowState>,
    next_id: u64,
    cascade_step: i32,
}

impl FloatingLayout {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new window; returns its layout id.
    pub fn add(&mut self, title: impl Into<String>, screen_w: u32, screen_h: u32) -> u64 {
        let id = {
            self.next_id += 1;
            self.next_id
        };

        let off = self.cascade_step;
        self.cascade_step = (self.cascade_step + 28) % 180;

        let w = (screen_w * 2 / 3).max(400);
        let h = (screen_h * 2 / 3).max(300);

        // Unfocus existing windows
        for win in &mut self.windows {
            win.focused = false;
        }

        self.windows.push(WindowState {
            id,
            title: title.into(),
            rect: Rect { x: 60 + off, y: 60 + off, w, h },
            focused: true,
            minimized: false,
            maximized: false,
        });

        id
    }

    pub fn remove(&mut self, id: u64) {
        self.windows.retain(|w| w.id != id);
        if let Some(top) = self.windows.last_mut() {
            top.focused = true;
        }
    }

    pub fn focus(&mut self, id: u64) {
        for w in &mut self.windows {
            w.focused = w.id == id;
        }
    }

    pub fn move_window(&mut self, id: u64, x: i32, y: i32) {
        if let Some(w) = self.get_mut(id) {
            w.rect.x = x;
            w.rect.y = y;
        }
    }

    pub fn resize_window(&mut self, id: u64, width: u32, height: u32) {
        if let Some(w) = self.get_mut(id) {
            w.rect.w = width.max(120);
            w.rect.h = height.max(80);
        }
    }

    pub fn toggle_maximize(&mut self, id: u64, screen_w: u32, screen_h: u32) {
        if let Some(w) = self.get_mut(id) {
            if w.maximized {
                w.maximized = false;
                // TODO: restore saved pre-maximize rect
            } else {
                w.maximized = true;
                w.rect = Rect { x: 0, y: 0, w: screen_w, h: screen_h };
            }
        }
    }

    pub fn focused(&self) -> Option<&WindowState> {
        self.windows.iter().find(|w| w.focused)
    }

    pub fn get(&self, id: u64) -> Option<&WindowState> {
        self.windows.iter().find(|w| w.id == id)
    }

    fn get_mut(&mut self, id: u64) -> Option<&mut WindowState> {
        self.windows.iter_mut().find(|w| w.id == id)
    }
}
