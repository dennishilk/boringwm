//! Pure master/stack geometry calculations.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub fn contains(self, x: i32, y: i32) -> bool {
        x >= self.x
            && y >= self.y
            && x < self.x.saturating_add(self.width as i32)
            && y < self.y.saturating_add(self.height as i32)
    }
}

/// Calculate deterministic master/stack rectangles inside a monitor work area.
/// Remainder pixels are assigned to the first stack clients.
pub fn master_stack(area: Rect, count: usize, gap: u32, border: u32, ratio: f32) -> Vec<Rect> {
    if count == 0 || area.width == 0 || area.height == 0 {
        return Vec::new();
    }
    let gap = gap
        .min(area.width.saturating_sub(1) / 2)
        .min(area.height.saturating_sub(1) / 2);
    let inner = |outer: u32| outer.saturating_sub(border.saturating_mul(2)).max(1);
    let x = area.x.saturating_add(gap as i32);
    let y = area.y.saturating_add(gap as i32);
    let width = area.width.saturating_sub(gap.saturating_mul(2)).max(1);
    let height = area.height.saturating_sub(gap.saturating_mul(2)).max(1);
    if count == 1 {
        return vec![Rect {
            x,
            y,
            width: inner(width),
            height: inner(height),
        }];
    }

    let ratio = ratio.clamp(0.2, 0.8);
    let column_gap = gap.min(width.saturating_sub(2));
    let columns = width.saturating_sub(column_gap).max(2);
    let master_width = ((columns as f32 * ratio).round() as u32).clamp(1, columns - 1);
    let stack_width = columns - master_width;
    let mut result = vec![Rect {
        x,
        y,
        width: inner(master_width),
        height: inner(height),
    }];
    let stack_count = (count - 1) as u32;
    let gaps = gap
        .saturating_mul(stack_count.saturating_sub(1))
        .min(height.saturating_sub(stack_count));
    let available = height.saturating_sub(gaps).max(stack_count);
    let base = available / stack_count;
    let remainder = available % stack_count;
    let mut stack_y = y;
    for index in 0..stack_count {
        let outer_height = base + u32::from(index < remainder);
        result.push(Rect {
            x: x.saturating_add(master_width as i32)
                .saturating_add(column_gap as i32),
            y: stack_y,
            width: inner(stack_width),
            height: inner(outer_height),
        });
        stack_y = stack_y
            .saturating_add(outer_height as i32)
            .saturating_add(gap as i32);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn area() -> Rect {
        Rect {
            x: 10,
            y: 20,
            width: 1920,
            height: 1080,
        }
    }

    #[test]
    fn zero_clients() {
        assert!(master_stack(area(), 0, 8, 2, 0.6).is_empty());
    }
    #[test]
    fn one_client() {
        assert_eq!(
            master_stack(area(), 1, 8, 2, 0.6),
            vec![Rect {
                x: 18,
                y: 28,
                width: 1900,
                height: 1060
            }]
        );
    }
    #[test]
    fn expected_counts() {
        for n in [2, 3, 10] {
            assert_eq!(master_stack(area(), n, 8, 2, 0.6).len(), n);
        }
    }
    #[test]
    fn monitor_offset_is_preserved() {
        assert!(master_stack(area(), 3, 8, 2, 0.6)
            .iter()
            .all(|r| r.x >= 10 && r.y >= 20));
    }
    #[test]
    fn narrow_and_small_are_nonzero() {
        for a in [
            Rect {
                x: 0,
                y: 0,
                width: 20,
                height: 200,
            },
            Rect {
                x: 0,
                y: 0,
                width: 3,
                height: 3,
            },
        ] {
            assert!(master_stack(a, 10, 8, 2, 0.6)
                .iter()
                .all(|r| r.width > 0 && r.height > 0));
        }
    }
    #[test]
    fn gaps_and_borders_do_not_overlap_columns() {
        let r = master_stack(area(), 3, 12, 4, 0.6);
        assert!(r[0].x + r[0].width as i32 + 8 < r[1].x);
    }
    #[test]
    fn ratio_is_clamped() {
        assert_eq!(
            master_stack(area(), 2, 8, 2, -5.0),
            master_stack(area(), 2, 8, 2, 0.2)
        );
        assert_eq!(
            master_stack(area(), 2, 8, 2, 5.0),
            master_stack(area(), 2, 8, 2, 0.8)
        );
    }
    #[test]
    fn remainder_pixels_reach_bottom() {
        let r = master_stack(
            Rect {
                x: 0,
                y: 0,
                width: 801,
                height: 603,
            },
            10,
            7,
            1,
            0.6,
        );
        let last = r.last().unwrap();
        assert!(last.y + last.height as i32 + 2 <= 603);
    }
}
