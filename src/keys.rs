use x11rb::protocol::xproto::*;
use x11rb::protocol::xproto::ConnectionExt;
use x11rb::rust_connection::RustConnection;

pub const MOD: ModMask = ModMask::M4;

// Keycodes (US layout)
pub const KEY_RETURN: u8 = 36;
pub const KEY_Q: u8 = 24;
pub const KEY_J: u8 = 44;
pub const KEY_K: u8 = 45;
pub const KEY_T: u8 = 28;
pub const KEY_B: u8 = 56;
pub const KEY_D: u8 = 40;

pub fn grab_keys(conn: &RustConnection, root: Window) {
    let keys = [
        KEY_RETURN,
        KEY_Q,
        KEY_J,
        KEY_K,
        KEY_T,
        KEY_B,
        KEY_D,
    ];

    let mods = [
        MOD,
        MOD | ModMask::M2,
        MOD | ModMask::LOCK,
        MOD | ModMask::M2 | ModMask::LOCK,
    ];

    for &m in &mods {
        for &keycode in &keys {
            let _ = conn.grab_key(
                false,
                root,
                m,
                keycode,
                GrabMode::ASYNC,
                GrabMode::ASYNC,
            );
        }
    }
}
