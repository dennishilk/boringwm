use x11rb::protocol::xproto::Window;

pub struct WmState {
    pub windows: Vec<Window>,
    pub focused: usize,
    pub master_ratio: f32,
}

impl WmState {
    pub fn new() -> Self {
        Self {
            windows: Vec::new(),
            focused: 0,
            master_ratio: 0.6,
        }
    }

    pub fn focus_next(&mut self) {
        if !self.windows.is_empty() {
            self.focused = (self.focused + 1) % self.windows.len();
        }
    }

    pub fn focus_prev(&mut self) {
        if !self.windows.is_empty() {
            if self.focused == 0 {
                self.focused = self.windows.len() - 1;
            } else {
                self.focused -= 1;
            }
        }
    }
}
