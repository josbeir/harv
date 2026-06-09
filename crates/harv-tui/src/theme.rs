use ratatui::style::Color;

pub struct Theme {
    pub bg: Color,
    pub surface: Color,
    pub fg: Color,
    pub primary: Color,
    pub success: Color,
    #[allow(dead_code)]
    pub warning: Color,
    #[allow(dead_code)]
    pub error: Color,
    pub muted: Color,
    pub border: Color,
    pub highlight: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            bg: Color::from_u32(0x0012131a),
            surface: Color::from_u32(0x001a1b26),
            fg: Color::from_u32(0x00c0caf5),
            primary: Color::from_u32(0x00f36c3d),
            success: Color::from_u32(0x009ece6a),
            warning: Color::from_u32(0x00e0af68),
            error: Color::from_u32(0x00f7768e),
            muted: Color::from_u32(0x00565f89),
            border: Color::from_u32(0x003b4261),
            highlight: Color::from_u32(0x00ffffff),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_theme_colors() {
        let t = Theme::default();
        assert_eq!(t.bg, Color::from_u32(0x0012131a));
        assert_eq!(t.primary, Color::from_u32(0x00f36c3d));
        assert_eq!(t.success, Color::from_u32(0x009ece6a));
        assert_eq!(t.highlight, Color::from_u32(0x00ffffff));
    }
}
