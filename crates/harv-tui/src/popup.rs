use ratatui::layout::{Constraint, Layout, Rect};

/// Create a popup rectangle centered in `r`, with the given width/height percentages.
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let v_margin = (100u16.saturating_sub(percent_y)) / 2;
    let popup_layout = Layout::vertical([
        Constraint::Percentage(v_margin),
        Constraint::Percentage(percent_y),
        Constraint::Percentage(v_margin),
    ])
    .split(r);

    let h_margin = (100u16.saturating_sub(percent_x)) / 2;
    Layout::horizontal([
        Constraint::Percentage(h_margin),
        Constraint::Percentage(percent_x),
        Constraint::Percentage(h_margin),
    ])
    .split(popup_layout[1])[1]
}

/// Create a popup rectangle centered in `r`, with fixed width and height in cells.
pub fn centered_rect_fixed(width: u16, height: u16, r: Rect) -> Rect {
    let x = r.x + (r.width.saturating_sub(width)) / 2;
    let y = r.y + (r.height.saturating_sub(height)) / 2;
    Rect {
        x,
        y,
        width,
        height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Rect;

    #[test]
    fn test_centered_rect_fixed_even() {
        let area = Rect::new(0, 0, 100, 40);
        let result = centered_rect_fixed(20, 10, area);
        assert_eq!(result.x, 40);
        assert_eq!(result.y, 15);
        assert_eq!(result.width, 20);
        assert_eq!(result.height, 10);
    }

    #[test]
    fn test_centered_rect_fixed_odd() {
        let area = Rect::new(0, 0, 99, 39);
        let result = centered_rect_fixed(20, 10, area);
        assert_eq!(result.x, 39);
        assert_eq!(result.y, 14);
        assert_eq!(result.width, 20);
        assert_eq!(result.height, 10);
    }

    #[test]
    fn test_centered_rect_fixed_too_small() {
        let area = Rect::new(0, 0, 5, 5);
        let result = centered_rect_fixed(20, 10, area);
        assert_eq!(result.x, 0);
        assert_eq!(result.y, 0);
    }

    #[test]
    fn test_centered_rect_percentage() {
        let area = Rect::new(0, 0, 100, 100);
        let result = centered_rect(50, 50, area);
        assert_eq!(result.x, 25);
        assert_eq!(result.y, 25);
        assert_eq!(result.width, 50);
        assert_eq!(result.height, 50);
    }

    #[test]
    fn test_centered_rect_full() {
        let area = Rect::new(0, 0, 100, 100);
        let result = centered_rect(100, 100, area);
        assert_eq!(result.x, 0);
        assert_eq!(result.y, 0);
    }
}
