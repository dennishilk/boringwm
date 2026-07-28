use anyhow::Context;
use x11rb::{connection::Connection, protocol::xproto::*, rust_connection::RustConnection};

pub const MOD: ModMask = ModMask::M4;
pub const SHIFT: ModMask = ModMask::SHIFT;
pub const KEY_RETURN: u8 = 36;
pub const KEY_Q: u8 = 24;
pub const KEY_J: u8 = 44;
pub const KEY_K: u8 = 45;
pub const KEY_T: u8 = 28;
pub const KEY_B: u8 = 56;
pub const KEY_D: u8 = 40;
pub const KEY_F: u8 = 41;
pub const KEY_SPACE: u8 = 65;
pub const KEY_H: u8 = 43;
pub const KEY_L: u8 = 46;
pub const KEY_M: u8 = 58;
pub const KEY_R: u8 = 27;
pub const KEY_E: u8 = 26;
pub const DIGITS: [u8; 9] = [10, 11, 12, 13, 14, 15, 16, 17, 18];

pub fn normalized(state: KeyButMask) -> ModMask {
    ModMask::from(u16::from(state) & !(u16::from(ModMask::M2) | u16::from(ModMask::LOCK)))
}

pub fn grab_keys(conn: &RustConnection, root: Window) -> anyhow::Result<()> {
    let mut bindings = vec![
        KEY_RETURN, KEY_Q, KEY_J, KEY_K, KEY_T, KEY_B, KEY_D, KEY_F, KEY_SPACE, KEY_H, KEY_L,
        KEY_M, KEY_R, KEY_E,
    ];
    bindings.extend(DIGITS);
    for modifiers in [MOD, MOD | SHIFT] {
        for ignored in [
            ModMask::default(),
            ModMask::M2,
            ModMask::LOCK,
            ModMask::M2 | ModMask::LOCK,
        ] {
            for key in &bindings {
                conn.grab_key(
                    false,
                    root,
                    modifiers | ignored,
                    *key,
                    GrabMode::ASYNC,
                    GrabMode::ASYNC,
                )
                .context("failed to request key grab")?;
            }
        }
    }
    conn.flush().context("failed to install key grabs")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lock_modifiers_are_ignored() {
        assert_eq!(
            normalized(KeyButMask::MOD4 | KeyButMask::MOD2 | KeyButMask::LOCK),
            MOD
        );
    }
}
