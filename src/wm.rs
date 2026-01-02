use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;
use x11rb::protocol::Event;
use x11rb::rust_connection::RustConnection;
use x11rb::CURRENT_TIME;

use crate::keys;
use crate::layout;

pub struct WindowManager {
    pub conn: RustConnection,
    pub screen_num: usize,
    pub windows: Vec<Window>,
    pub master_ratio: f32,
}

impl WindowManager {
    pub fn new(conn: RustConnection, screen_num: usize) -> Self {
        Self {
            conn,
            screen_num,
            windows: Vec::new(),
            master_ratio: 0.6,
        }
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        let screen = &self.conn.setup().roots[self.screen_num];
        let root = screen.root;

        // Select events on root window
        self.conn.change_window_attributes(
            root,
            &ChangeWindowAttributesAux::new().event_mask(
                EventMask::SUBSTRUCTURE_REDIRECT
                    | EventMask::SUBSTRUCTURE_NOTIFY
                    | EventMask::KEY_PRESS,
            ),
        )?;
        self.conn.flush()?;

        // Grab keys
        keys::grab_keys(&self.conn, root);

        log::info!("boringwm running");

        loop {
            let event = self.conn.wait_for_event()?;
            self.handle_event(event, screen)?;
        }
    }

    fn handle_event(
        &mut self,
        event: Event,
        screen: &Screen,
    ) -> anyhow::Result<()> {
        match event {
            Event::MapRequest(e) => {
                self.on_map_request(e, screen)?;
            }

            Event::DestroyNotify(e) => {
                self.on_destroy(e);
            }

            Event::KeyPress(e) => {
                self.on_key_press(e, screen)?;
            }

            _ => {}
        }
        Ok(())
    }

    fn on_map_request(
        &mut self,
        e: MapRequestEvent,
        screen: &Screen,
    ) -> anyhow::Result<()> {
        let win = e.window;

        // Add to managed windows
        if !self.windows.contains(&win) {
            self.windows.push(win);
        }

        self.conn.map_window(win)?;
        self.relayout(screen);

        self.conn.flush()?;
        Ok(())
    }

    fn on_destroy(&mut self, e: DestroyNotifyEvent) {
        self.windows.retain(|&w| w != e.window);
    }

    fn on_key_press(
        &mut self,
        e: KeyPressEvent,
        screen: &Screen,
    ) -> anyhow::Result<()> {
        let keysym = keys::keycode_to_keysym(&self.conn, e.detail);

        match keysym {
            keys::KEY_TERMINAL => {
                self.spawn("kitty");
            }

            keys::KEY_QUIT => {
                self.close_focused();
            }

            keys::KEY_NEXT => {
                self.focus_next();
            }

            keys::KEY_PREV => {
                self.focus_prev();
            }

            _ => {}
        }

        self.relayout(screen);
        self.conn.flush()?;
        Ok(())
    }

    fn relayout(&self, screen: &Screen) {
        layout::tile(
            &self.conn,
            screen.width_in_pixels,
            screen.height_in_pixels,
            &self.windows,
            self.master_ratio,
        );
    }

    fn spawn(&self, cmd: &str) {
        let _ = self.conn.spawn(cmd);
    }

    fn close_focused(&self) {
        if let Some(&win) = self.windows.last() {
            let _ = self.conn.send_event(
                false,
                win,
                EventMask::NO_EVENT,
                ClientMessageEvent {
                    response_type: CLIENT_MESSAGE_EVENT,
                    format: 32,
                    sequence: 0,
                    window: win,
                    type_: self
                        .conn
                        .intern_atom(false, b"WM_PROTOCOLS")
                        .unwrap()
                        .reply()
                        .unwrap()
                        .atom,
                    data: ClientMessageData::from_data32([
                        self.conn
                            .intern_atom(false, b"WM_DELETE_WINDOW")
                            .unwrap()
                            .reply()
                            .unwrap()
                            .atom,
                        CURRENT_TIME,
                        0,
                        0,
                        0,
                    ]),
                },
            );
        }
    }

    fn focus_next(&mut self) {
        if self.windows.len() > 1 {
            self.windows.rotate_left(1);
        }
    }

    fn focus_prev(&mut self) {
        if self.windows.len() > 1 {
            self.windows.rotate_right(1);
        }
    }
}
