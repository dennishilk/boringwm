use anyhow::Context;
use log::info;
use std::process::Command;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::ConnectionExt;
use x11rb::protocol::xproto::*;
use x11rb::protocol::Event;
use x11rb::rust_connection::RustConnection;
use x11rb::CURRENT_TIME;

use crate::keys;
use crate::layout;
use crate::state::WmState;

const BORDER_WIDTH: u32 = 2;
const BORDER_FOCUSED: u32 = 0x88CCFF;
const BORDER_NORMAL: u32 = 0x333333;

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
    let _ = Command::new("sh")
        .arg("-c")
        .arg("$HOME/.config/boringwm/autostart.sh")
        .spawn();

    keys::grab_keys(&conn, root);
    let multimedia_keycodes = keys::multimedia_keycodes(&conn);
    keys::grab_multimedia_keys(&conn, root, &multimedia_keycodes);
    conn.flush()?;

    info!("boringwm running");

    let mut state = WmState::new();

    loop {
        let event = conn.wait_for_event()?;
        match event {
            Event::MapRequest(e) => {
                let w = e.window;

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

                state.windows.push(w);
                state.focused = state.windows.len() - 1;

                let _ = conn.map_window(w);
                focus(&conn, &state);
                retile(&conn, &screen, &state);
            }

            Event::DestroyNotify(e) => {
                state.windows.retain(|&x| x != e.window);
                state.focused = 0;
                focus(&conn, &state);
                retile(&conn, &screen, &state);
            }

            Event::KeyPress(e) => {
                handle_key(&conn, &mut state, &multimedia_keycodes, e.detail);
            }

            Event::FocusIn(e) => {
                if let Some(index) = state.windows.iter().position(|&w| w == e.event) {
                    if state.focused != index {
                        state.focused = index;
                        focus(&conn, &state);
                    }
                }
            }

            _ => {}
        }
    }
}

fn handle_key(
    conn: &RustConnection,
    state: &mut WmState,
    multimedia: &keys::MultimediaKeycodes,
    keycode: u8,
) {
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
            let _ = Command::new("rofi")
                .args([
                    "-show",
                    "drun",
                    "-modi",
                    "drun,wifi:nmcli rofi wifi-menu",
                ])
                .spawn();
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

        _ if multimedia.volume_up.contains(&keycode) => {
            adjust_volume(VolumeAction::Up);
        }

        _ if multimedia.volume_down.contains(&keycode) => {
            adjust_volume(VolumeAction::Down);
        }

        _ if multimedia.volume_mute.contains(&keycode) => {
            adjust_volume(VolumeAction::ToggleMute);
        }

        _ => {}
    }
}

enum VolumeAction {
    Up,
    Down,
    ToggleMute,
}

fn adjust_volume(action: VolumeAction) {
    match action {
        VolumeAction::Up => {
            let _ = Command::new("pactl")
                .args(["set-sink-volume", "@DEFAULT_SINK@", "+5%"])
                .status();
        }
        VolumeAction::Down => {
            let _ = Command::new("pactl")
                .args(["set-sink-volume", "@DEFAULT_SINK@", "-5%"])
                .status();
        }
        VolumeAction::ToggleMute => {
            let _ = Command::new("pactl")
                .args(["set-sink-mute", "@DEFAULT_SINK@", "toggle"])
                .status();
        }
    }

    let muted = current_mute_state();
    let volume = if muted == Some(true) {
        None
    } else {
        current_volume_percentage()
    };
    show_volume_osd(volume, muted);
}

fn current_volume_percentage() -> Option<u8> {
    let output = Command::new("pactl")
        .args(["get-sink-volume", "@DEFAULT_SINK@"])
        .output()
        .ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    first_percentage(&stdout)
}

fn current_mute_state() -> Option<bool> {
    let output = Command::new("pactl")
        .args(["get-sink-mute", "@DEFAULT_SINK@"])
        .output()
        .ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.contains("yes") {
        Some(true)
    } else if stdout.contains("no") {
        Some(false)
    } else {
        None
    }
}

fn first_percentage(text: &str) -> Option<u8> {
    for token in text.split_whitespace() {
        if let Some(percent) = token.strip_suffix('%') {
            if let Ok(value) = percent.parse::<u8>() {
                return Some(value);
            }
        }
    }
    None
}

fn show_volume_osd(volume: Option<u8>, muted: Option<bool>) {
    let mut command = Command::new("notify-send");
    command.args([
        "-a",
        "BoringWM",
        "-u",
        "low",
        "-h",
        "string:x-canonical-private-synchronous:volume",
    ]);

    if let Some(value) = volume {
        command.args(["-h", &format!("int:value:{value}")]);
    }

    let summary = "Audio";
    let body = match muted {
        Some(true) => "Stumm",
        Some(false) => match volume {
            Some(value) => return spawn_notify(command, summary, &format!("{value}%")),
            None => "Aktiv",
        },
        None => match volume {
            Some(value) => return spawn_notify(command, summary, &format!("{value}%")),
            None => "Geändert",
        },
    };

    spawn_notify(command, summary, body);
}

fn spawn_notify(mut command: Command, summary: &str, body: &str) {
    let _ = command.args([summary, body]).spawn();
}

fn close_window(conn: &RustConnection, window: Window) {
    let wm_protocols = conn
        .intern_atom(false, b"WM_PROTOCOLS")
        .ok()
        .and_then(|cookie| cookie.reply().ok())
        .map(|reply| reply.atom);

    let wm_delete = conn
        .intern_atom(false, b"WM_DELETE_WINDOW")
        .ok()
        .and_then(|cookie| cookie.reply().ok())
        .map(|reply| reply.atom);

    let supports_delete = match (wm_protocols, wm_delete) {
        (Some(wm_protocols), Some(wm_delete)) => {
            let mut supported = false;
            if let Ok(cookie) = conn.get_property(false, window, wm_protocols, AtomEnum::ATOM, 0, 32)
            {
                if let Ok(prop) = cookie.reply() {
                    if let Some(values) = prop.value32() {
                        supported = values.any(|atom| atom == wm_delete);
                    }
                }
            }
            if supported {
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
            supported
        }
        _ => false,
    };

    if !supports_delete {
        let _ = conn.kill_client(window);
    }
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
