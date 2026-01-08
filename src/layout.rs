use x11rb::protocol::xproto::*;
use x11rb::protocol::xproto::ConnectionExt;
use x11rb::rust_connection::RustConnection;

// Fixed boring gaps (pixels)
const GAP: i32 = 8;

pub fn tile(
    conn: &RustConnection,
    screen_width: u16,
    screen_height: u16,
    windows: &[Window],
    master_ratio: f32,
    border_width: u32,
) {
    if windows.is_empty() {
        return;
    }

    let sw = screen_width as i32;
    let sh = screen_height as i32;
    let usable_width = sw - 2 * GAP;
    let usable_height = sh - 2 * GAP;

    if usable_width <= 0 || usable_height <= 0 {
        return;
    }

    // Single window
    if windows.len() == 1 {
        let safe_width = usable_width.max(0);
        let safe_height = usable_height.max(0);

        let _ = conn.configure_window(
            windows[0],
            &ConfigureWindowAux::new()
                .x(Some(GAP))
                .y(Some(GAP))
                .width(Some(safe_width as u32))
                .height(Some(safe_height as u32))
                .border_width(Some(border_width)),
        );
        return;
    }

    let master_width = ((sw as f32 * master_ratio) as i32).clamp(0, sw);
    let stack_width = (sw - master_width).max(0);

    let stack_count = windows.len() - 1;
    let stack_available_height = sh - (stack_count as i32 + 1) * GAP;
    if stack_available_height <= 0 {
        return;
    }
    let stack_height = stack_available_height / stack_count as i32;
    let safe_master_width = (master_width - 2 * GAP).max(0);
    let safe_master_height = usable_height.max(0);
    let safe_stack_width = (stack_width - 2 * GAP).max(0);
    let safe_stack_height = stack_height.max(0);

    // Master window
    let _ = conn.configure_window(
        windows[0],
        &ConfigureWindowAux::new()
            .x(Some(GAP))
            .y(Some(GAP))
            .width(Some(safe_master_width as u32))
            .height(Some(safe_master_height as u32))
            .border_width(Some(border_width)),
    );

    // Stack windows
    for (i, w) in windows.iter().skip(1).enumerate() {
        let y = GAP + i as i32 * (stack_height + GAP);

        let _ = conn.configure_window(
            *w,
            &ConfigureWindowAux::new()
                .x(Some(master_width + GAP))
                .y(Some(y))
                .width(Some(safe_stack_width as u32))
                .height(Some(safe_stack_height as u32))
                .border_width(Some(border_width)),
        );
    }
}
