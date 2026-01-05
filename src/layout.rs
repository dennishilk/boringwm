use x11rb::protocol::xproto::ConnectionExt;
use x11rb::protocol::xproto::*;
use x11rb::rust_connection::RustConnection;

// i3/Hyprland-inspired gaps (pixels)
const OUTER_GAP: i32 = 12;
const INNER_GAP: i32 = 8;

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
    let area_w = sw - 2 * OUTER_GAP;
    let area_h = sh - 2 * OUTER_GAP;

    // Single window
    if windows.len() == 1 {
        let _ = conn.configure_window(
            windows[0],
            &ConfigureWindowAux::new()
                .x(Some(OUTER_GAP))
                .y(Some(OUTER_GAP))
                .width(Some(area_w as u32))
                .height(Some(area_h as u32))
                .border_width(Some(border_width)),
        );
        return;
    }

    let master_width = ((area_w - INNER_GAP) as f32 * master_ratio) as i32;
    let stack_width = area_w - INNER_GAP - master_width;

    let stack_count = windows.len() - 1;
    let stack_height = (area_h - (stack_count as i32 - 1) * INNER_GAP) / stack_count as i32;

    // Master window
    let _ = conn.configure_window(
        windows[0],
        &ConfigureWindowAux::new()
            .x(Some(OUTER_GAP))
            .y(Some(OUTER_GAP))
            .width(Some(master_width as u32))
            .height(Some(area_h as u32))
            .border_width(Some(border_width)),
    );

    // Stack windows
    for (i, w) in windows.iter().skip(1).enumerate() {
        let y = OUTER_GAP + i as i32 * (stack_height + INNER_GAP);

        let _ = conn.configure_window(
            *w,
            &ConfigureWindowAux::new()
                .x(Some(OUTER_GAP + master_width + INNER_GAP))
                .y(Some(y))
                .width(Some(stack_width as u32))
                .height(Some(stack_height as u32))
                .border_width(Some(border_width)),
        );
    }
}
