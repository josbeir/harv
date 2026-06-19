use ratatui::style::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ThemeMode {
    Dark,
    Light,
}

pub struct Theme {
    pub mode: ThemeMode,
    pub bg: Color,
    pub surface: Color,
    pub fg: Color,
    pub primary: Color,
    pub success: Color,
    pub warning: Color,
    #[allow(dead_code)]
    pub error: Color,
    pub muted: Color,
    pub border: Color,
    pub highlight: Color,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            mode: ThemeMode::Dark,
            bg: Color::from_u32(0x0012131a),
            surface: Color::from_u32(0x00151721),
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

    pub fn light() -> Self {
        Self {
            mode: ThemeMode::Light,
            bg: Color::from_u32(0x00f5f0eb),
            surface: Color::from_u32(0x00e8e0d8),
            fg: Color::from_u32(0x002e2e2e),
            primary: Color::from_u32(0x00d45113),
            success: Color::from_u32(0x003d854d),
            warning: Color::from_u32(0x00b8860b),
            error: Color::from_u32(0x00c0392b),
            muted: Color::from_u32(0x008a8a8a),
            border: Color::from_u32(0x00c8c0b8),
            highlight: Color::from_u32(0x001a1a1a),
        }
    }

    pub fn detect() -> Self {
        match dark_light::detect() {
            Ok(dark_light::Mode::Dark) => Self::dark(),
            Ok(dark_light::Mode::Light | dark_light::Mode::Unspecified) => Self::light(),
            Err(_) => Self::dark(),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dark_theme_colors() {
        let t = Theme::dark();
        assert_eq!(t.mode, ThemeMode::Dark);
        assert_eq!(t.bg, Color::from_u32(0x0012131a));
        assert_eq!(t.primary, Color::from_u32(0x00f36c3d));
        assert_eq!(t.success, Color::from_u32(0x009ece6a));
        assert_eq!(t.highlight, Color::from_u32(0x00ffffff));
    }

    #[test]
    fn test_light_theme_colors() {
        let t = Theme::light();
        assert_eq!(t.mode, ThemeMode::Light);
        assert_eq!(t.bg, Color::from_u32(0x00f5f0eb));
        assert_eq!(t.fg, Color::from_u32(0x002e2e2e));
        assert_eq!(t.muted, Color::from_u32(0x008a8a8a));
    }

    #[test]
    fn test_default_is_dark() {
        let t = Theme::default();
        assert_eq!(t.mode, ThemeMode::Dark);
    }
}
