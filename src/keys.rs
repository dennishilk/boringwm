use x11rb::protocol::xproto::*;
use x11rb::protocol::xproto::ConnectionExt;
use x11rb::rust_connection::RustConnection;

pub const MOD: ModMask = ModMask::M4;
pub const IGNORE_MODS: ModMask = ModMask::M2 | ModMask::LOCK;
// M2   = NumLock
// LOCK = CapsLock


// Hardcoded MVP keycodes (US layout, works in Xephyr & real HW)
pub const KEY_RETURN: u8 = 36; // Enter
pub const KEY_Q: u8 = 24;      // q
pub const KEY_J: u8 = 44;      // j
pub const KEY_K: u8 = 45;      // k;

pub fn grab_keys(conn: &RustConnection, root: Window) {
    let keys = [KEY_RETURN, KEY_Q, KEY_J, KEY_K];

    for keycode in keys {
        let _ = conn.grab_key(
            false,
            root,
            MOD,
            keycode,
            GrabMode::ASYNC,
            GrabMode::ASYNC,
        );
    }
}
