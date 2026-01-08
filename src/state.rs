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

    pub fn remove_window(&mut self, window: Window) {
        let removed_index = self.windows.iter().position(|&w| w == window);
        if let Some(index) = removed_index {
            self.windows.retain(|&w| w != window);

            if self.windows.is_empty() {
                self.focused = 0;
                return;
            }

            if self.focused == index {
                self.focused = if index == 0 { 0 } else { index - 1 };
            } else if index < self.focused {
                self.focused -= 1;
            }

            if self.focused >= self.windows.len() {
                self.focused = self.windows.len() - 1;
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
    fn remove_focused_window_moves_focus_to_previous() {
        let mut state = WmState::new();
        state.windows = vec![10, 20, 30];
        state.focused = 1;

        state.remove_window(20);

        assert_eq!(state.windows, vec![10, 30]);
        assert_eq!(state.focused, 0);
    }

    #[test]
    fn remove_window_before_focus_shifts_focus_left() {
        let mut state = WmState::new();
        state.windows = vec![10, 20, 30];
        state.focused = 2;

        state.remove_window(10);

        assert_eq!(state.windows, vec![20, 30]);
        assert_eq!(state.focused, 1);
    }

    #[test]
    fn remove_last_window_resets_focus() {
        let mut state = WmState::new();
        state.windows = vec![10];
        state.focused = 0;

        state.remove_window(10);

        assert!(state.windows.is_empty());
        assert_eq!(state.focused, 0);
    }
}
