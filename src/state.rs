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

#[cfg(test)]
mod tests {
    use super::WmState;

    #[test]
    fn focus_next_empty_windows_is_stable() {
        let mut state = WmState::new();
        state.focused = 0;

        state.focus_next();

        assert_eq!(state.focused, 0);
        assert!(state.windows.is_empty());
    }

    #[test]
    fn focus_next_wraps_from_last_to_first() {
        let mut state = WmState::new();
        state.windows = vec![1, 2, 3];
        state.focused = 2;

        state.focus_next();

        assert_eq!(state.focused, 0);
    }

    #[test]
    fn focus_prev_wraps_from_first_to_last() {
        let mut state = WmState::new();
        state.windows = vec![1, 2, 3];
        state.focused = 0;

        state.focus_prev();

        assert_eq!(state.focused, 2);
    }

    #[test]
    fn focus_prev_empty_windows_is_stable() {
        let mut state = WmState::new();
        state.focused = 0;

        state.focus_prev();

        assert_eq!(state.focused, 0);
        assert!(state.windows.is_empty());
    }
}
