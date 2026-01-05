use anyhow::Context;
use log::{info, warn};
use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};
use std::os::unix::fs::PermissionsExt;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::ConnectionExt;
use x11rb::protocol::xproto::*;
use x11rb::protocol::Event;
use x11rb::rust_connection::RustConnection;
use x11rb::CURRENT_TIME;

use crate::keys;
use crate::layout;
use crate::state::WmState;

const BORDER_WIDTH: u32 = 3;
const BORDER_FOCUSED: u32 = 0x88C0D0;
const BORDER_NORMAL: u32 = 0x2E3440;

pub fn run() -> anyhow::Result<()> {
    let (conn, screen_num) = x11rb::connect(None).context("X11 connect failed")?;
    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;

    conn.change_window_attributes(
        root,
        &ChangeWindowAttributesAux::new().event_mask(
            EventMask::SUBSTRUCTURE_REDIRECT
                | EventMask::SUBSTRUCTURE_NOTIFY
                | EventMask::KEY_PRESS,
        ),
    )?;

    // Autostart (user controlled)
    run_autostart();

    keys::grab_keys(&conn, root);
    conn.flush()?;

    info!("boringwm running");

    let mut state = WmState::new();

    loop {
        let event = conn.wait_for_event()?;
        match event {
            Event::MapRequest(e) => {
                let w = e.window;

                if let Ok(attrs) = conn.get_window_attributes(w) {
                    if let Ok(reply) = attrs.reply() {
                        if reply.override_redirect {
                            let _ = conn.map_window(w);
                            continue;
                        }
                    }
                }

                if is_fullscreen(&conn, &screen, w) {
                    let _ = conn.map_window(w);
                    continue;
                }

                let _ = conn.change_window_attributes(
                    w,
                    &ChangeWindowAttributesAux::new()
                        .border_pixel(BORDER_NORMAL)
                        .event_mask(EventMask::FOCUS_CHANGE),
                );

                let _ =
                    conn.configure_window(w, &ConfigureWindowAux::new().border_width(BORDER_WIDTH));

                if !state.windows.contains(&w) {
                    state.windows.push(w);
                }
                state.focused = state.windows.len().saturating_sub(1);

                let _ = conn.map_window(w);
                focus(&conn, &state);
                retile(&conn, &screen, &state);
            }

            Event::DestroyNotify(e) => {
                state.remove_window(e.window);
                focus(&conn, &state);
                retile(&conn, &screen, &state);
            }

            Event::UnmapNotify(e) => {
                state.remove_window(e.window);
                focus(&conn, &state);
                retile(&conn, &screen, &state);
            }

            Event::ConfigureRequest(e) => {
                if !state.windows.contains(&e.window) {
                    let aux = ConfigureWindowAux::new()
                        .x(e.x.map(|value| value as i32))
                        .y(e.y.map(|value| value as i32))
                        .width(e.width)
                        .height(e.height)
                        .border_width(e.border_width)
                        .sibling(e.sibling)
                        .stack_mode(e.stack_mode);
                    let _ = conn.configure_window(e.window, &aux);
                }
            }

            Event::KeyPress(e) => {
                handle_key(&conn, &screen, &mut state, e.detail);
            }

            _ => {}
        }
    }
}

fn handle_key(conn: &RustConnection, screen: &Screen, state: &mut WmState, keycode: u8) {
    match keycode {
        // Terminal
        keys::KEY_RETURN => {
            let _ = Command::new("kitty").spawn();
        }

        // File manager
        keys::KEY_T => {
            let _ = Command::new("thunar").spawn();
        }

        // Browser
        keys::KEY_B => {
            let _ = Command::new("firefox").spawn();
        }

        // App launcher (rofi)
        keys::KEY_D => {
            let _ = Command::new("boringwm-rofi").spawn();
        }

        // Close focused window
        keys::KEY_Q => {
            if let Some(&w) = state.windows.get(state.focused) {
                close_window(conn, w);
            }
        }

        // Focus navigation
        keys::KEY_J => {
            state.focus_next();
            focus(conn, state);
        }

        keys::KEY_K => {
            state.focus_prev();
            focus(conn, state);
        }

        _ => {}
    }

    retile(conn, screen, state);
}

fn run_autostart() {
    let home = match env::var("HOME") {
        Ok(value) => value,
        Err(_) => {
            warn!("HOME not set; skipping autostart");
            return;
        }
    };
    let path = PathBuf::from(home).join(".config/boringwm/autostart.sh");

    let metadata = match fs::metadata(&path) {
        Ok(metadata) => metadata,
        Err(_) => {
            warn!("autostart script not found: {}", path.display());
            return;
        }
    };

    if !metadata.is_file() {
        warn!("autostart path is not a file: {}", path.display());
        return;
    }

    let mode = metadata.permissions().mode();
    if mode & 0o111 == 0 {
        warn!("autostart script is not executable: {}", path.display());
        return;
    }

    if let Err(error) = Command::new(&path).spawn() {
        warn!("failed to launch autostart: {}", error);
    }
}

fn close_window(conn: &RustConnection, window: Window) {
    let wm_protocols = conn
        .intern_atom(false, b"WM_PROTOCOLS")
        .unwrap()
        .reply()
        .unwrap()
        .atom;

    let wm_delete = conn
        .intern_atom(false, b"WM_DELETE_WINDOW")
        .unwrap()
        .reply()
        .unwrap()
        .atom;

    let event = ClientMessageEvent {
        response_type: CLIENT_MESSAGE_EVENT,
        format: 32,
        sequence: 0,
        window,
        type_: wm_protocols,
        data: ClientMessageData::from([wm_delete, CURRENT_TIME, 0, 0, 0]),
    };

    let _ = conn.send_event(false, window, EventMask::NO_EVENT, event);
}

fn focus(conn: &RustConnection, state: &WmState) {
    for (i, &w) in state.windows.iter().enumerate() {
        let color = if i == state.focused {
            BORDER_FOCUSED
        } else {
            BORDER_NORMAL
        };

        let _ =
            conn.change_window_attributes(w, &ChangeWindowAttributesAux::new().border_pixel(color));
    }

    if let Some(&w) = state.windows.get(state.focused) {
        let _ = conn.set_input_focus(InputFocus::POINTER_ROOT, w, CURRENT_TIME);

        let _ = conn.configure_window(w, &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE));
    }
}

fn retile(conn: &RustConnection, screen: &Screen, state: &WmState) {
    layout::tile(
        conn,
        screen.width_in_pixels,
        screen.height_in_pixels,
        &state.windows,
        state.master_ratio,
        BORDER_WIDTH,
    );
    let _ = conn.flush();
}

fn is_fullscreen(conn: &RustConnection, screen: &Screen, window: Window) -> bool {
    let net_wm_state = conn
        .intern_atom(false, b"_NET_WM_STATE")
        .unwrap()
        .reply()
        .unwrap()
        .atom;

    let net_wm_state_fs = conn
        .intern_atom(false, b"_NET_WM_STATE_FULLSCREEN")
        .unwrap()
        .reply()
        .unwrap()
        .atom;

    if let Ok(reply) = conn.get_property(false, window, net_wm_state, AtomEnum::ATOM, 0, 32) {
        if let Ok(prop) = reply.reply() {
            if let Some(values) = prop.value32() {
                let atoms: Vec<u32> = values.collect();
                if atoms.contains(&net_wm_state_fs) {
                    let _ = conn.configure_window(
                        window,
                        &ConfigureWindowAux::new()
                            .x(Some(0))
                            .y(Some(0))
                            .width(Some(screen.width_in_pixels as u32))
                            .height(Some(screen.height_in_pixels as u32))
                            .border_width(0),
                    );
                    return true;
                }
            }
        }
    }
    false
}
