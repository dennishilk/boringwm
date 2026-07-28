use crate::{
    commands,
    config::Config,
    keys,
    layout::{self, Rect},
    state::{Client, WmState},
};
use anyhow::{Context, Result};
use log::{debug, info, warn};
use std::{collections::HashSet, env, process::Command};
use x11rb::wrapper::ConnectionExt as _;
use x11rb::{
    connection::Connection,
    protocol::{xproto::*, Event},
    rust_connection::RustConnection,
    CURRENT_TIME,
};

#[derive(Clone, Copy)]
struct Atoms {
    wm_protocols: Atom,
    wm_delete: Atom,
    wm_take_focus: Atom,
    net_supported: Atom,
    net_supporting_wm_check: Atom,
    net_wm_name: Atom,
    utf8_string: Atom,
    net_active_window: Atom,
    net_client_list: Atom,
    net_client_list_stacking: Atom,
    net_wm_state: Atom,
    net_wm_state_fullscreen: Atom,
    net_wm_window_type: Atom,
    net_wm_window_type_dialog: Atom,
    net_number_of_desktops: Atom,
    net_current_desktop: Atom,
    net_wm_desktop: Atom,
    net_desktop_names: Atom,
}

impl Atoms {
    fn new(conn: &RustConnection) -> Result<Self> {
        fn atom(conn: &RustConnection, name: &[u8]) -> Result<Atom> {
            Ok(conn.intern_atom(false, name)?.reply()?.atom)
        }
        Ok(Self {
            wm_protocols: atom(conn, b"WM_PROTOCOLS")?,
            wm_delete: atom(conn, b"WM_DELETE_WINDOW")?,
            wm_take_focus: atom(conn, b"WM_TAKE_FOCUS")?,
            net_supported: atom(conn, b"_NET_SUPPORTED")?,
            net_supporting_wm_check: atom(conn, b"_NET_SUPPORTING_WM_CHECK")?,
            net_wm_name: atom(conn, b"_NET_WM_NAME")?,
            utf8_string: atom(conn, b"UTF8_STRING")?,
            net_active_window: atom(conn, b"_NET_ACTIVE_WINDOW")?,
            net_client_list: atom(conn, b"_NET_CLIENT_LIST")?,
            net_client_list_stacking: atom(conn, b"_NET_CLIENT_LIST_STACKING")?,
            net_wm_state: atom(conn, b"_NET_WM_STATE")?,
            net_wm_state_fullscreen: atom(conn, b"_NET_WM_STATE_FULLSCREEN")?,
            net_wm_window_type: atom(conn, b"_NET_WM_WINDOW_TYPE")?,
            net_wm_window_type_dialog: atom(conn, b"_NET_WM_WINDOW_TYPE_DIALOG")?,
            net_number_of_desktops: atom(conn, b"_NET_NUMBER_OF_DESKTOPS")?,
            net_current_desktop: atom(conn, b"_NET_CURRENT_DESKTOP")?,
            net_wm_desktop: atom(conn, b"_NET_WM_DESKTOP")?,
            net_desktop_names: atom(conn, b"_NET_DESKTOP_NAMES")?,
        })
    }
    fn supported(self) -> [Atom; 13] {
        [
            self.net_supported,
            self.net_supporting_wm_check,
            self.net_wm_name,
            self.net_active_window,
            self.net_client_list,
            self.net_client_list_stacking,
            self.net_wm_state,
            self.net_wm_state_fullscreen,
            self.net_wm_window_type,
            self.net_number_of_desktops,
            self.net_current_desktop,
            self.net_wm_desktop,
            self.net_desktop_names,
        ]
    }
}

struct Wm {
    conn: RustConnection,
    root: Window,
    atoms: Atoms,
    config: Config,
    state: WmState,
    check_window: Window,
    running: bool,
    restart: bool,
    ignored_unmaps: HashSet<Window>,
}

pub fn run() -> Result<()> {
    let config = Config::load()?;
    let (conn, screen_num) = x11rb::connect(None).context("cannot connect to X11 display")?;
    let screen = conn.setup().roots[screen_num].clone();
    let root = screen.root;
    let atoms = Atoms::new(&conn).context("cannot initialize X11 atoms")?;
    conn.change_window_attributes(
        root,
        &ChangeWindowAttributesAux::new().event_mask(
            EventMask::SUBSTRUCTURE_REDIRECT
                | EventMask::SUBSTRUCTURE_NOTIFY
                | EventMask::PROPERTY_CHANGE
                | EventMask::KEY_PRESS,
        ),
    )?
    .check()
    .context("another window manager is probably running")?;
    let check_window = conn.generate_id()?;
    conn.create_window(
        0,
        check_window,
        root,
        0,
        0,
        1,
        1,
        0,
        WindowClass::INPUT_OUTPUT,
        screen.root_visual,
        &CreateWindowAux::new(),
    )?;
    let monitor = Rect {
        x: 0,
        y: 0,
        width: screen.width_in_pixels.into(),
        height: screen.height_in_pixels.into(),
    };
    let mut wm = Wm {
        conn,
        root,
        atoms,
        config: config.clone(),
        state: WmState::new(config.workspaces, vec![monitor], config.master_ratio),
        check_window,
        running: true,
        restart: false,
        ignored_unmaps: HashSet::new(),
    };
    wm.publish_root_properties()?;
    keys::grab_keys(&wm.conn, root)?;
    if let Some(path) = &wm.config.autostart {
        commands::autostart(path);
    }
    wm.adopt_existing()?;
    wm.conn.flush()?;
    info!(
        "BoringWM started with {} workspace(s)",
        wm.state.workspace_count()
    );
    while wm.running {
        match wm.conn.wait_for_event() {
            Ok(event) => wm.handle(event),
            Err(error) => return Err(error).context("X11 connection failed"),
        }
    }
    wm.shutdown()?;
    if wm.restart {
        let exe = env::current_exe().context("cannot locate executable for restart")?;
        Command::new(exe).args(env::args_os().skip(1)).exec();
    }
    Ok(())
}

#[cfg(unix)]
trait CommandExec {
    fn exec(&mut self);
}
#[cfg(unix)]
impl CommandExec for Command {
    fn exec(&mut self) {
        use std::os::unix::process::CommandExt;
        let error = CommandExt::exec(self);
        log::error!("restart failed: {error}");
    }
}

impl Wm {
    fn publish_root_properties(&self) -> Result<()> {
        let s = self.atoms.supported();
        self.conn.change_property32(
            PropMode::REPLACE,
            self.root,
            self.atoms.net_supported,
            AtomEnum::ATOM,
            &s,
        )?;
        for w in [self.root, self.check_window] {
            self.conn.change_property32(
                PropMode::REPLACE,
                w,
                self.atoms.net_supporting_wm_check,
                AtomEnum::WINDOW,
                &[self.check_window],
            )?;
        }
        self.conn.change_property8(
            PropMode::REPLACE,
            self.check_window,
            self.atoms.net_wm_name,
            self.atoms.utf8_string,
            b"BoringWM",
        )?;
        self.conn.change_property32(
            PropMode::REPLACE,
            self.root,
            self.atoms.net_number_of_desktops,
            AtomEnum::CARDINAL,
            &[self.state.workspace_count() as u32],
        )?;
        let names = (1..=self.state.workspace_count())
            .flat_map(|n| [b'0' + n as u8, 0])
            .collect::<Vec<_>>();
        self.conn.change_property8(
            PropMode::REPLACE,
            self.root,
            self.atoms.net_desktop_names,
            self.atoms.utf8_string,
            &names,
        )?;
        self.sync_properties();
        Ok(())
    }
    fn adopt_existing(&mut self) -> Result<()> {
        let children = self.conn.query_tree(self.root)?.reply()?.children;
        for w in children {
            let attrs = match self.conn.get_window_attributes(w)?.reply() {
                Ok(a) => a,
                Err(_) => continue,
            };
            if attrs.override_redirect || attrs.map_state == MapState::UNMAPPED {
                continue;
            }
            self.manage(w, true);
        }
        Ok(())
    }
    fn handle(&mut self, event: Event) {
        let result = match event {
            Event::MapRequest(e) => {
                self.manage(e.window, false);
                Ok(())
            }
            Event::DestroyNotify(e) => {
                self.unmanage(e.window);
                Ok(())
            }
            Event::UnmapNotify(e) => {
                if !self.ignored_unmaps.remove(&e.window) {
                    self.unmanage(e.window)
                }
                Ok(())
            }
            Event::ConfigureRequest(e) => self.configure_request(e),
            Event::EnterNotify(e) => {
                if self.state.contains(e.event) {
                    self.state.set_focus(Some(e.event));
                    self.apply_focus()
                }
                Ok(())
            }
            Event::ClientMessage(e) => {
                self.client_message(e);
                Ok(())
            }
            Event::PropertyNotify(e) => {
                if self.state.contains(e.window) && e.atom == self.atoms.net_wm_state {
                    self.read_fullscreen(e.window)
                }
                Ok(())
            }
            Event::KeyPress(e) => {
                self.key(e.detail, keys::normalized(e.state));
                Ok(())
            }
            _ => Ok(()),
        };
        if let Err(error) = result {
            warn!("recoverable X11 operation failed: {error:#}");
        }
    }
    fn manage(&mut self, w: Window, existing: bool) {
        if self.state.contains(w) {
            if !existing {
                let _ = self.conn.map_window(w);
            }
            return;
        }
        let attrs = match self.conn.get_window_attributes(w) {
            Ok(cookie) => match cookie.reply() {
                Ok(value) => value,
                Err(error) => {
                    debug!("window {w:#x} disappeared before manage: {error}");
                    return;
                }
            },
            Err(e) => {
                debug!("window {w:#x} disappeared before manage: {e}");
                return;
            }
        };
        if attrs.override_redirect {
            return;
        }
        let geometry_reply = match self.conn.get_geometry(w) {
            Ok(cookie) => match cookie.reply() {
                Ok(value) => value,
                Err(_) => return,
            },
            Err(_) => return,
        };
        let geometry = Rect {
            x: geometry_reply.x.into(),
            y: geometry_reply.y.into(),
            width: geometry_reply.width.into(),
            height: geometry_reply.height.into(),
        };
        let transient = self
            .conn
            .get_property(false, w, AtomEnum::WM_TRANSIENT_FOR, AtomEnum::WINDOW, 0, 1)
            .ok()
            .and_then(|c| c.reply().ok())
            .and_then(|p| p.value32()?.next());
        let dialog = self
            .property_atoms(w, self.atoms.net_wm_window_type)
            .contains(&self.atoms.net_wm_window_type_dialog);
        let fullscreen = self
            .property_atoms(w, self.atoms.net_wm_state)
            .contains(&self.atoms.net_wm_state_fullscreen);
        let monitor = self.monitor_for(geometry);
        let floating = transient.is_some() || dialog;
        let client = Client {
            window: w,
            workspace: self.state.current_workspace,
            monitor,
            floating,
            fullscreen,
            geometry,
            saved_geometry: fullscreen.then_some(geometry),
            saved_floating: floating,
        };
        if !self.state.add(client) {
            return;
        }
        let mask = EventMask::ENTER_WINDOW
            | EventMask::FOCUS_CHANGE
            | EventMask::PROPERTY_CHANGE
            | EventMask::STRUCTURE_NOTIFY;
        let _ = self.conn.change_window_attributes(
            w,
            &ChangeWindowAttributesAux::new()
                .border_pixel(self.config.unfocused_border)
                .event_mask(mask),
        );
        let _ = self.conn.configure_window(
            w,
            &ConfigureWindowAux::new().border_width(if fullscreen {
                0
            } else {
                self.config.border_width
            }),
        );
        let _ = self.conn.change_property32(
            PropMode::REPLACE,
            w,
            self.atoms.net_wm_desktop,
            AtomEnum::CARDINAL,
            &[self.state.current_workspace as u32],
        );
        let _ = self.conn.map_window(w);
        self.arrange();
        self.apply_focus();
        self.sync_properties();
        debug!("managed window {w:#x}");
    }
    fn unmanage(&mut self, w: Window) {
        if self.state.remove(w).is_some() {
            debug!("unmanaged window {w:#x}");
            self.arrange();
            self.apply_focus();
            self.sync_properties();
        }
    }
    fn configure_request(&self, e: ConfigureRequestEvent) -> Result<()> {
        if !self.state.contains(e.window) || self.state.client(e.window).is_some_and(|c| c.floating)
        {
            let mut a = ConfigureWindowAux::new();
            if e.value_mask.contains(ConfigWindow::X) {
                a = a.x(e.x as i32)
            }
            if e.value_mask.contains(ConfigWindow::Y) {
                a = a.y(e.y as i32)
            }
            if e.value_mask.contains(ConfigWindow::WIDTH) {
                a = a.width(e.width as u32)
            }
            if e.value_mask.contains(ConfigWindow::HEIGHT) {
                a = a.height(e.height as u32)
            }
            if e.value_mask.contains(ConfigWindow::BORDER_WIDTH) {
                a = a.border_width(e.border_width as u32)
            }
            if e.value_mask.contains(ConfigWindow::SIBLING) {
                a = a.sibling(e.sibling)
            }
            if e.value_mask.contains(ConfigWindow::STACK_MODE) {
                a = a.stack_mode(e.stack_mode)
            }
            self.conn.configure_window(e.window, &a)?;
        } else {
            self.send_configure(e.window);
        }
        Ok(())
    }
    fn arrange(&mut self) {
        for monitor in 0..self.state.monitors.len() {
            let ids = self.state.tiled_on(monitor);
            let rects = layout::master_stack(
                self.state.monitors[monitor],
                ids.len(),
                self.config.gaps,
                self.config.border_width,
                self.state.master_ratio,
            );
            for (w, r) in ids.into_iter().zip(rects) {
                if let Some(c) = self.state.client_mut(w) {
                    c.geometry = r
                }
                let _ = self.conn.configure_window(
                    w,
                    &ConfigureWindowAux::new()
                        .x(r.x)
                        .y(r.y)
                        .width(r.width)
                        .height(r.height)
                        .border_width(self.config.border_width),
                );
            }
        }
        for w in self.state.visible() {
            if let Some(c) = self.state.client(w) {
                if c.fullscreen {
                    let r = self.state.monitors[c.monitor];
                    let _ = self.conn.configure_window(
                        w,
                        &ConfigureWindowAux::new()
                            .x(r.x)
                            .y(r.y)
                            .width(r.width)
                            .height(r.height)
                            .border_width(0)
                            .stack_mode(StackMode::ABOVE),
                    );
                }
            }
        }
        let _ = self.conn.flush();
    }
    fn apply_focus(&self) {
        for w in self.state.visible() {
            let color = if Some(w) == self.state.focused {
                self.config.focused_border
            } else {
                self.config.unfocused_border
            };
            let _ = self
                .conn
                .change_window_attributes(w, &ChangeWindowAttributesAux::new().border_pixel(color));
        }
        let target = self.state.focused.unwrap_or(self.root);
        let _ = self
            .conn
            .set_input_focus(InputFocus::POINTER_ROOT, target, CURRENT_TIME);
        if target != self.root {
            let _ = self.conn.configure_window(
                target,
                &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE),
            );
            if self
                .property_atoms(target, self.atoms.wm_protocols)
                .contains(&self.atoms.wm_take_focus)
            {
                let event = ClientMessageEvent::new(
                    32,
                    target,
                    self.atoms.wm_protocols,
                    [self.atoms.wm_take_focus, CURRENT_TIME, 0, 0, 0],
                );
                let _ = self
                    .conn
                    .send_event(false, target, EventMask::NO_EVENT, event);
            }
        }
        self.sync_properties();
    }
    fn key(&mut self, key: u8, mods: ModMask) {
        if !mods.contains(keys::MOD) {
            return;
        }
        let shift = mods.contains(keys::SHIFT);
        if let Some(ws) = keys::DIGITS.iter().position(|k| *k == key) {
            if shift {
                if let Some(w) = self.state.move_focused_to_workspace(ws) {
                    self.ignored_unmaps.insert(w);
                    let _ = self.conn.unmap_window(w);
                    self.arrange();
                    self.apply_focus();
                    self.sync_properties();
                }
            } else {
                self.switch_workspace(ws)
            }
            return;
        }
        match (key, shift) {
            (keys::KEY_RETURN, false) => self.spawn(self.config.terminal.clone()),
            (keys::KEY_T, false) => self.spawn(self.config.file_manager.clone()),
            (keys::KEY_B, false) => self.spawn(self.config.browser.clone()),
            (keys::KEY_D, false) => self.spawn(self.config.launcher.clone()),
            (keys::KEY_Q, false) => self.close_focused(),
            (keys::KEY_J, false) => {
                self.state.focus_cycle(1);
                self.apply_focus()
            }
            (keys::KEY_K, false) => {
                self.state.focus_cycle(-1);
                self.apply_focus()
            }
            (keys::KEY_J, true) => {
                self.state.reorder(1);
                self.arrange()
            }
            (keys::KEY_K, true) => {
                self.state.reorder(-1);
                self.arrange()
            }
            (keys::KEY_M, false) => {
                self.state.promote();
                self.arrange()
            }
            (keys::KEY_H, false) => {
                self.state.master_ratio = (self.state.master_ratio - 0.05).max(0.2);
                self.arrange()
            }
            (keys::KEY_L, false) => {
                self.state.master_ratio = (self.state.master_ratio + 0.05).min(0.8);
                self.arrange()
            }
            (keys::KEY_F, false) => {
                if let Some(w) = self.state.focused {
                    self.set_fullscreen(w, !self.state.client(w).is_some_and(|c| c.fullscreen))
                }
            }
            (keys::KEY_SPACE, false) => {
                if let Some(w) = self.state.focused {
                    if let Some(c) = self.state.client_mut(w) {
                        c.floating = !c.floating;
                    }
                    self.arrange()
                }
            }
            (keys::KEY_E, true) => self.running = false,
            (keys::KEY_R, true) => {
                self.restart = true;
                self.running = false
            }
            _ => {}
        }
    }
    fn spawn(&self, c: Vec<String>) {
        if let Err(e) = commands::spawn(&c) {
            warn!("{e:#}")
        }
    }
    fn close_focused(&self) {
        let Some(w) = self.state.focused else { return };
        if self
            .property_atoms(w, self.atoms.wm_protocols)
            .contains(&self.atoms.wm_delete)
        {
            let event = ClientMessageEvent::new(
                32,
                w,
                self.atoms.wm_protocols,
                [self.atoms.wm_delete, CURRENT_TIME, 0, 0, 0],
            );
            let _ = self.conn.send_event(false, w, EventMask::NO_EVENT, event);
        } else if let Err(e) = self.conn.kill_client(w) {
            warn!("cannot close {w:#x}: {e}")
        }
    }
    fn client_message(&mut self, e: ClientMessageEvent) {
        if e.type_ == self.atoms.net_wm_state {
            let d = e.data.as_data32();
            if d[1] == self.atoms.net_wm_state_fullscreen
                || d[2] == self.atoms.net_wm_state_fullscreen
            {
                let current = self.state.client(e.window).is_some_and(|c| c.fullscreen);
                let wanted = match d[0] {
                    0 => false,
                    1 => true,
                    2 => !current,
                    _ => return,
                };
                self.set_fullscreen(e.window, wanted)
            }
        } else if e.type_ == self.atoms.net_current_desktop {
            self.switch_workspace(e.data.as_data32()[0] as usize)
        } else if e.type_ == self.atoms.net_active_window && self.state.contains(e.window) {
            let ws = self
                .state
                .client(e.window)
                .map(|c| c.workspace)
                .unwrap_or(0);
            self.switch_workspace(ws);
            self.state.set_focus(Some(e.window));
            self.apply_focus()
        }
    }
    fn read_fullscreen(&mut self, w: Window) {
        let wanted = self
            .property_atoms(w, self.atoms.net_wm_state)
            .contains(&self.atoms.net_wm_state_fullscreen);
        self.set_fullscreen(w, wanted)
    }
    fn set_fullscreen(&mut self, w: Window, wanted: bool) {
        let Some(c) = self.state.client_mut(w) else {
            return;
        };
        if c.fullscreen == wanted {
            return;
        }
        if wanted {
            c.saved_geometry = Some(c.geometry);
            c.saved_floating = c.floating;
            c.fullscreen = true;
        } else {
            c.fullscreen = false;
            c.floating = c.saved_floating;
            if let Some(r) = c.saved_geometry.take() {
                c.geometry = r;
            }
        }
        let values = if wanted {
            vec![self.atoms.net_wm_state_fullscreen]
        } else {
            Vec::new()
        };
        let _ = self.conn.change_property32(
            PropMode::REPLACE,
            w,
            self.atoms.net_wm_state,
            AtomEnum::ATOM,
            &values,
        );
        self.arrange();
        self.apply_focus();
    }
    fn switch_workspace(&mut self, ws: usize) {
        if !self.state.switch_workspace(ws) {
            return;
        }
        for c in self.state.clients() {
            if c.workspace == ws {
                let _ = self.conn.map_window(c.window);
            } else {
                self.ignored_unmaps.insert(c.window);
                let _ = self.conn.unmap_window(c.window);
            }
        }
        self.arrange();
        self.apply_focus();
        self.sync_properties();
    }
    fn property_atoms(&self, w: Window, property: Atom) -> Vec<Atom> {
        self.conn
            .get_property(false, w, property, AtomEnum::ATOM, 0, 64)
            .ok()
            .and_then(|c| c.reply().ok())
            .and_then(|p| p.value32().map(Iterator::collect))
            .unwrap_or_default()
    }
    fn monitor_for(&self, r: Rect) -> usize {
        let x = r.x.saturating_add((r.width / 2) as i32);
        let y = r.y.saturating_add((r.height / 2) as i32);
        self.state
            .monitors
            .iter()
            .position(|m| m.contains(x, y))
            .unwrap_or(0)
    }
    fn send_configure(&self, w: Window) {
        if let Some(c) = self.state.client(w) {
            let e = ConfigureNotifyEvent {
                response_type: CONFIGURE_NOTIFY_EVENT,
                sequence: 0,
                event: w,
                window: w,
                above_sibling: x11rb::NONE,
                x: c.geometry.x as i16,
                y: c.geometry.y as i16,
                width: c.geometry.width as u16,
                height: c.geometry.height as u16,
                border_width: self.config.border_width as u16,
                override_redirect: false,
            };
            let _ = self
                .conn
                .send_event(false, w, EventMask::STRUCTURE_NOTIFY, e);
        }
    }
    fn sync_properties(&self) {
        let clients = self.state.clients().map(|c| c.window).collect::<Vec<_>>();
        let active = self.state.focused.into_iter().collect::<Vec<_>>();
        let _ = self.conn.change_property32(
            PropMode::REPLACE,
            self.root,
            self.atoms.net_client_list,
            AtomEnum::WINDOW,
            &clients,
        );
        let _ = self.conn.change_property32(
            PropMode::REPLACE,
            self.root,
            self.atoms.net_client_list_stacking,
            AtomEnum::WINDOW,
            &clients,
        );
        let _ = self.conn.change_property32(
            PropMode::REPLACE,
            self.root,
            self.atoms.net_active_window,
            AtomEnum::WINDOW,
            &active,
        );
        let _ = self.conn.change_property32(
            PropMode::REPLACE,
            self.root,
            self.atoms.net_current_desktop,
            AtomEnum::CARDINAL,
            &[self.state.current_workspace as u32],
        );
        let _ = self.conn.flush();
    }
    fn shutdown(&self) -> Result<()> {
        for c in self.state.clients() {
            let _ = self.conn.map_window(c.window);
            let _ = self
                .conn
                .configure_window(c.window, &ConfigureWindowAux::new().border_width(0));
        }
        for p in [
            self.atoms.net_active_window,
            self.atoms.net_client_list,
            self.atoms.net_client_list_stacking,
            self.atoms.net_supporting_wm_check,
        ] {
            let _ = self.conn.delete_property(self.root, p);
        }
        let _ = self.conn.destroy_window(self.check_window);
        self.conn.flush()?;
        info!("BoringWM stopped without closing clients");
        Ok(())
    }
}
