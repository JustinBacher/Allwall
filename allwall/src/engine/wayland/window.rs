pub struct WindowInfo {
    pub x: f32,
    pub y: f32,
    pub width: u32,
    pub height: u32,
}

pub struct WindowTracker;

impl WindowTracker {
    pub fn new() -> Self {
        Self
    }

    pub fn window_at_point(&self, _x: f32, _y: f32) -> Option<WindowInfo> {
        None
    }
}

impl Default for WindowTracker {
    fn default() -> Self {
        Self::new()
    }
}
