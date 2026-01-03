use x11rb::connection::Connection;
use x11rb::keysym::{Keysym, XK_Return, XK_b, XK_d, XK_j, XK_k, XK_q, XK_t};
use x11rb::protocol::xproto::ConnectionExt;
use x11rb::protocol::xproto::*;
use x11rb::rust_connection::RustConnection;

pub const MOD: ModMask = ModMask::M4;

pub const KEYSYM_AUDIO_LOWER_VOLUME: Keysym = 0x1008FF11;
pub const KEYSYM_AUDIO_MUTE: Keysym = 0x1008FF12;
pub const KEYSYM_AUDIO_RAISE_VOLUME: Keysym = 0x1008FF13;

#[derive(Debug, Clone)]
pub struct Keybindings {
    pub terminal: Vec<u8>,
    pub close: Vec<u8>,
    pub focus_next: Vec<u8>,
    pub focus_prev: Vec<u8>,
    pub file_manager: Vec<u8>,
    pub browser: Vec<u8>,
    pub launcher: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct MultimediaKeycodes {
    pub volume_down: Vec<u8>,
    pub volume_mute: Vec<u8>,
    pub volume_up: Vec<u8>,
}

pub fn grab_keys(conn: &RustConnection, root: Window, bindings: &Keybindings) {
    let keys: Vec<u8> = bindings
        .terminal
        .iter()
        .chain(bindings.close.iter())
        .chain(bindings.focus_next.iter())
        .chain(bindings.focus_prev.iter())
        .chain(bindings.file_manager.iter())
        .chain(bindings.browser.iter())
        .chain(bindings.launcher.iter())
        .copied()
        .collect();

    let mods = [
        MOD,
        MOD | ModMask::M2,
        MOD | ModMask::LOCK,
        MOD | ModMask::M2 | ModMask::LOCK,
    ];

    for &m in &mods {
        for &keycode in &keys {
            let _ = conn.grab_key(false, root, m, keycode, GrabMode::ASYNC, GrabMode::ASYNC);
        }
    }
}

pub fn grab_multimedia_keys(conn: &RustConnection, root: Window, multimedia: &MultimediaKeycodes) {
    let mods = [
        ModMask::default(),
        ModMask::M2,
        ModMask::LOCK,
        ModMask::M2 | ModMask::LOCK,
    ];

    for &m in &mods {
        for &keycode in multimedia
            .volume_down
            .iter()
            .chain(multimedia.volume_mute.iter())
            .chain(multimedia.volume_up.iter())
        {
            let _ = conn.grab_key(false, root, m, keycode, GrabMode::ASYNC, GrabMode::ASYNC);
        }
    }
}

pub fn multimedia_keycodes(conn: &RustConnection) -> MultimediaKeycodes {
    MultimediaKeycodes {
        volume_down: keycodes_for_keysym(conn, KEYSYM_AUDIO_LOWER_VOLUME),
        volume_mute: keycodes_for_keysym(conn, KEYSYM_AUDIO_MUTE),
        volume_up: keycodes_for_keysym(conn, KEYSYM_AUDIO_RAISE_VOLUME),
    }
}

pub fn keybindings(conn: &RustConnection) -> Keybindings {
    Keybindings {
        terminal: keycodes_for_keysym(conn, XK_Return),
        close: keycodes_for_keysym(conn, XK_q),
        focus_next: keycodes_for_keysym(conn, XK_j),
        focus_prev: keycodes_for_keysym(conn, XK_k),
        file_manager: keycodes_for_keysym(conn, XK_t),
        browser: keycodes_for_keysym(conn, XK_b),
        launcher: keycodes_for_keysym(conn, XK_d),
    }
}

fn keycodes_for_keysym(conn: &RustConnection, keysym: Keysym) -> Vec<u8> {
    let setup = conn.setup();
    let min_keycode = setup.min_keycode;
    let max_keycode = setup.max_keycode;
    let keycode_count = max_keycode.saturating_sub(min_keycode) + 1;

    let reply = match conn.get_keyboard_mapping(min_keycode, keycode_count) {
        Ok(cookie) => match cookie.reply() {
            Ok(reply) => reply,
            Err(_) => return Vec::new(),
        },
        Err(_) => return Vec::new(),
    };

    let mut keycodes = Vec::new();
    let keysyms_per_keycode = reply.keysyms_per_keycode as usize;
    for (index, chunk) in reply.keysyms.chunks(keysyms_per_keycode).enumerate() {
        if chunk.iter().any(|&sym| sym == keysym) {
            let keycode = min_keycode.saturating_add(index as u8);
            keycodes.push(keycode);
        }
    }

    keycodes
}
