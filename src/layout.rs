use x11rb::protocol::xproto::*;
use x11rb::protocol::xproto::ConnectionExt;
use x11rb::rust_connection::RustConnection;

// Fixed boring gaps and borders (pixels)
const GAP: i32 = 8;
const BORDER_WIDTH: u32 = 1;

pub fn tile(
    conn: &RustConnection,
    screen_width: u16,
    screen_height: u16,
    windows: &[Window],
    master_ratio: f32,
) {
    if windows.is_empty() {
        return;
    }

    let sw = screen_width as i32;
    let sh = screen_height as i32;

    // Single window
    if windows.len() == 1 {
        let _ = conn.configure_window(
            windows[0],
            &ConfigureWindowAux::new()
                .x(Some(GAP))
                .y(Some(GAP))
                .width(Some((sw - 2 * GAP) as u32))
                .height(Some((sh - 2 * GAP) as u32))
                .border_width(Some(BORDER_WIDTH)),
        );
        return;
    }

    let master_width = (sw as f32 * master_ratio) as i32;
    let stack_width = sw - master_width;

    let stack_count = windows.len() - 1;
    let stack_height =
        (sh - (stack_count as i32 + 1) * GAP) / stack_count as i32;

    // Master window
    let _ = conn.configure_window(
        windows[0],
        &ConfigureWindowAux::new()
            .x(Some(GAP))
            .y(Some(GAP))
            .width(Some((master_width - 2 * GAP) as u32))
            .height(Some((sh - 2 * GAP) as u32))
            .border_width(Some(BORDER_WIDTH)),
    );

    // Stack windows
    for (i, w) in windows.iter().skip(1).enumerate() {
        let y = GAP + i as i32 * (stack_height + GAP);

        let _ = conn.configure_window(
            *w,
            &ConfigureWindowAux::new()
                .x(Some(master_width + GAP))
                .y(Some(y))
                .width(Some((stack_width - 2 * GAP) as u32))
                .height(Some(stack_height as u32))
                .border_width(Some(BORDER_WIDTH)),
        );
    }
}
