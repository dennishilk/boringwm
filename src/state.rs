use crate::layout::Rect;
use std::collections::HashMap;
use x11rb::protocol::xproto::Window;

#[derive(Clone, Debug)]
pub struct Client {
    pub window: Window,
    pub workspace: usize,
    pub monitor: usize,
    pub floating: bool,
    pub fullscreen: bool,
    pub geometry: Rect,
    pub saved_geometry: Option<Rect>,
    pub saved_floating: bool,
}

#[derive(Debug)]
pub struct WmState {
    clients: HashMap<Window, Client>,
    order: Vec<Vec<Window>>,
    focus: Vec<Option<Window>>,
    pub current_workspace: usize,
    pub focused: Option<Window>,
    pub monitors: Vec<Rect>,
    pub master_ratio: f32,
}

impl WmState {
    pub fn new(workspaces: usize, monitors: Vec<Rect>, master_ratio: f32) -> Self {
        let count = workspaces.max(1);
        Self {
            clients: HashMap::new(),
            order: vec![Vec::new(); count],
            focus: vec![None; count],
            current_workspace: 0,
            focused: None,
            monitors,
            master_ratio: master_ratio.clamp(0.2, 0.8),
        }
    }
    pub fn workspace_count(&self) -> usize {
        self.order.len()
    }
    pub fn client(&self, w: Window) -> Option<&Client> {
        self.clients.get(&w)
    }
    pub fn client_mut(&mut self, w: Window) -> Option<&mut Client> {
        self.clients.get_mut(&w)
    }
    pub fn clients(&self) -> impl Iterator<Item = &Client> {
        self.clients.values()
    }
    pub fn contains(&self, w: Window) -> bool {
        self.clients.contains_key(&w)
    }
    pub fn add(&mut self, client: Client) -> bool {
        if self.contains(client.window) {
            return false;
        }
        let w = client.window;
        let workspace = client.workspace.min(self.order.len() - 1);
        self.clients.insert(
            w,
            Client {
                workspace,
                ..client
            },
        );
        self.order[workspace].push(w);
        if workspace == self.current_workspace {
            self.set_focus(Some(w));
        }
        true
    }
    pub fn remove(&mut self, w: Window) -> Option<Client> {
        let client = self.clients.remove(&w)?;
        let order = &mut self.order[client.workspace];
        let index = order.iter().position(|id| *id == w).unwrap_or(0);
        order.retain(|id| *id != w);
        if self.focus[client.workspace] == Some(w) {
            self.focus[client.workspace] = order
                .get(index.saturating_sub(1))
                .copied()
                .or_else(|| order.get(index).copied());
        }
        if self.focused == Some(w) {
            self.focused = self.focus[self.current_workspace];
        }
        Some(client)
    }
    pub fn visible(&self) -> Vec<Window> {
        self.order[self.current_workspace].clone()
    }
    pub fn tiled_on(&self, monitor: usize) -> Vec<Window> {
        self.order[self.current_workspace]
            .iter()
            .copied()
            .filter(|w| {
                self.clients
                    .get(w)
                    .is_some_and(|c| c.monitor == monitor && !c.floating && !c.fullscreen)
            })
            .collect()
    }
    pub fn set_focus(&mut self, w: Option<Window>) {
        self.focused = w.filter(|id| {
            self.clients
                .get(id)
                .is_some_and(|c| c.workspace == self.current_workspace)
        });
        self.focus[self.current_workspace] = self.focused;
    }
    pub fn focus_cycle(&mut self, delta: isize) {
        let order = &self.order[self.current_workspace];
        if order.is_empty() {
            self.set_focus(None);
            return;
        }
        let current = self
            .focused
            .and_then(|w| order.iter().position(|id| *id == w))
            .unwrap_or(0) as isize;
        let next = (current + delta).rem_euclid(order.len() as isize) as usize;
        self.set_focus(Some(order[next]));
    }
    pub fn switch_workspace(&mut self, workspace: usize) -> bool {
        if workspace >= self.order.len() || workspace == self.current_workspace {
            return false;
        }
        self.current_workspace = workspace;
        self.focused = self.focus[workspace]
            .filter(|w| self.clients.contains_key(w))
            .or_else(|| self.order[workspace].last().copied());
        self.focus[workspace] = self.focused;
        true
    }
    pub fn move_focused_to_workspace(&mut self, workspace: usize) -> Option<Window> {
        if workspace >= self.order.len() {
            return None;
        }
        let w = self.focused?;
        let old = self.current_workspace;
        self.order[old].retain(|id| *id != w);
        self.order[workspace].push(w);
        self.clients.get_mut(&w)?.workspace = workspace;
        self.focus[workspace] = Some(w);
        self.focused = self.order[old].last().copied();
        self.focus[old] = self.focused;
        Some(w)
    }
    pub fn reorder(&mut self, delta: isize) {
        let order = &mut self.order[self.current_workspace];
        let Some(w) = self.focused else { return };
        let Some(index) = order.iter().position(|id| *id == w) else {
            return;
        };
        let other = (index as isize + delta).rem_euclid(order.len() as isize) as usize;
        order.swap(index, other);
    }
    pub fn promote(&mut self) {
        if let Some(w) = self.focused {
            let order = &mut self.order[self.current_workspace];
            if let Some(i) = order.iter().position(|id| *id == w) {
                order.swap(0, i);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn client(w: Window, ws: usize) -> Client {
        Client {
            window: w,
            workspace: ws,
            monitor: 0,
            floating: false,
            fullscreen: false,
            geometry: Rect::default(),
            saved_geometry: None,
            saved_floating: false,
        }
    }
    fn state() -> WmState {
        WmState::new(
            3,
            vec![Rect {
                x: 0,
                y: 0,
                width: 800,
                height: 600,
            }],
            0.6,
        )
    }
    #[test]
    fn duplicate_add_is_rejected() {
        let mut s = state();
        assert!(s.add(client(1, 0)));
        assert!(!s.add(client(1, 0)));
        assert_eq!(s.visible(), vec![1]);
    }
    #[test]
    fn focus_cycles_both_directions() {
        let mut s = state();
        for w in 1..=3 {
            s.add(client(w, 0));
        }
        s.focus_cycle(1);
        assert_eq!(s.focused, Some(1));
        s.focus_cycle(-1);
        assert_eq!(s.focused, Some(3));
    }
    #[test]
    fn removal_selects_deterministic_replacement() {
        let mut s = state();
        for w in 1..=3 {
            s.add(client(w, 0));
        }
        s.remove(3);
        assert_eq!(s.focused, Some(2));
        s.remove(2);
        s.remove(1);
        assert_eq!(s.focused, None);
    }
    #[test]
    fn workspace_preserves_focus_and_order() {
        let mut s = state();
        s.add(client(1, 0));
        s.add(client(2, 1));
        s.switch_workspace(1);
        assert_eq!(s.focused, Some(2));
        s.switch_workspace(0);
        assert_eq!(s.focused, Some(1));
    }
    #[test]
    fn move_has_single_owner() {
        let mut s = state();
        s.add(client(1, 0));
        s.move_focused_to_workspace(2);
        assert!(s.visible().is_empty());
        s.switch_workspace(2);
        assert_eq!(s.visible(), vec![1]);
        assert_eq!(s.client(1).unwrap().workspace, 2);
    }
    #[test]
    fn reorder_and_promote_are_stable() {
        let mut s = state();
        for w in 1..=3 {
            s.add(client(w, 0));
        }
        s.reorder(-1);
        assert_eq!(s.visible(), vec![1, 3, 2]);
        s.promote();
        assert_eq!(s.visible(), vec![3, 1, 2]);
    }
    #[test]
    fn fullscreen_round_trip_state() {
        let mut s = state();
        s.add(client(1, 0));
        let c = s.client_mut(1).unwrap();
        c.saved_geometry = Some(Rect {
            x: 1,
            y: 2,
            width: 3,
            height: 4,
        });
        c.saved_floating = c.floating;
        c.fullscreen = true;
        c.fullscreen = false;
        c.floating = c.saved_floating;
        assert_eq!(c.saved_geometry.take().unwrap().x, 1);
    }
}
