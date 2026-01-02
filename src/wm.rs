use std::process::Command;

use x11rb::connection::Connection;
use x11rb::protocol::Event;
use x11rb::protocol::xproto::*;
use x11rb::rust_connection::RustConnection;

use crate::keys::*;
use crate::log;

pub fn run(
    conn: RustConnection,
    root: Window,
) -> Result<(), Box<dyn std::error::Error>> {
    // Listen for window + key events
    let mask = EventMask::SUBSTRUCTURE_REDIRECT
        | EventMask::SUBSTRUCTURE_NOTIFY
        | EventMask::KEY_PRESS;

    // Become window manager (fails if another WM is running)
    conn.change_window_attributes(
        root,
        &ChangeWindowAttributesAux::new().event_mask(mask),
    )?
    .check()?;

    // 🔑 CRITICAL: grab keybindings (this was missing)
    grab_keys(&conn, root);

    conn.flush()?;

    log::info("boringwm-daily running");

    loop {
        let event = conn.wait_for_event()?;

        match event {
            // -------------------------
            // Key handling
            // -------------------------
            Event::KeyPress(e) => {
                handle_key(e);
            }

            // -------------------------
            // New window
            // -------------------------
            Event::MapRequest(e) => {
                let _ = conn.map_window(e.window);
                let _ = conn.flush();
            }

            // -------------------------
            // Window destroyed
            // -------------------------
            Event::DestroyNotify(_e) => {
                // no state yet (intentionally boring)
            }

            _ => {}
        }
    }
}

// --------------------------------------------------
// Key handling
// --------------------------------------------------
fn handle_key(event: KeyPressEvent) {
    match event.detail {
        // Apps
        KEY_RETURN => spawn("kitty"),
        KEY_B => spawn("google-chrome"),
        KEY_F => spawn("thunar"),
        KEY_Q => std::process::exit(0),

        // Audio
        KEY_VOL_UP => spawn("pactl set-sink-volume @DEFAULT_SINK@ +5%"),
        KEY_VOL_DOWN => spawn("pactl set-sink-volume @DEFAULT_SINK@ -5%"),
        KEY_MUTE => spawn("pactl set-sink-mute @DEFAULT_SINK@ toggle"),

        // Brightness
        KEY_BRIGHT_UP => spawn("brightnessctl set +10%"),
        KEY_BRIGHT_DOWN => spawn("brightnessctl set 10%-"),

        _ => {}
    }
}

// --------------------------------------------------
// Spawn helper
// --------------------------------------------------
fn spawn(cmd: &str) {
    let _ = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .spawn();
}
