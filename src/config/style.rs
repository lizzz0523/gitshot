use tiny_skia::Color;

use super::ConfigFile;

pub struct Style {
    pub font_path: String,
    pub font_size: f32,
    pub line_height: f32,
    pub padding: f32,
    pub max_img_width: u32,
    pub canvas_bg: Color,
    pub diff_style: DiffStyle,
    pub status_style: StatusStyle,
}

impl Style {
    pub fn from_config(cfg: &ConfigFile) -> Self {
        let (canvas_bg, diff_style, status_style) = match cfg.color_scheme {
            super::ColorScheme::Dark => (
                Color::from_rgba8(24, 24, 27, 255),
                DiffStyle::dark(),
                StatusStyle::dark(),
            ),
            super::ColorScheme::Light => (
                Color::from_rgba8(255, 255, 255, 255),
                DiffStyle::light(),
                StatusStyle::light(),
            ),
        };

        Self {
            font_path: cfg.font_path.clone(),
            font_size: cfg.font_size,
            line_height: cfg.line_height,
            padding: cfg.padding,
            max_img_width: cfg.max_img_width,
            canvas_bg,
            diff_style,
            status_style,
        }
    }
}

// ── DiffStyle ───────────────────────────────────────────────────────

pub struct DiffStyle {
    pub added_bg: Color,
    pub deleted_bg: Color,
    pub separator_bg: Color,
    pub added_fg: (u8, u8, u8),
    pub deleted_fg: (u8, u8, u8),
    pub hunk_fg: (u8, u8, u8),
    pub file_fg: (u8, u8, u8),
    pub default_fg: (u8, u8, u8),
}

impl DiffStyle {
    fn dark() -> Self {
        Self {
            added_bg: Color::from_rgba8(46, 160, 67, 30),
            deleted_bg: Color::from_rgba8(248, 81, 73, 30),
            separator_bg: Color::from_rgba8(48, 54, 61, 255),
            added_fg: (63, 185, 80),
            deleted_fg: (248, 81, 73),
            hunk_fg: (187, 128, 230),
            file_fg: (88, 166, 255),
            default_fg: (201, 209, 217),
        }
    }

    fn light() -> Self {
        Self {
            added_bg: Color::from_rgba8(46, 160, 67, 25),
            deleted_bg: Color::from_rgba8(248, 81, 73, 25),
            separator_bg: Color::from_rgba8(216, 222, 228, 255),
            added_fg: (26, 127, 55),
            deleted_fg: (207, 34, 46),
            hunk_fg: (111, 78, 167),
            file_fg: (4, 81, 165),
            default_fg: (31, 35, 40),
        }
    }
}

// ── StatusStyle ─────────────────────────────────────────────────────

pub struct StatusStyle {
    pub title_fg: (u8, u8, u8),
    pub path_fg: (u8, u8, u8),
    pub added_fg: (u8, u8, u8),
    pub added_bg: Color,
    pub modified_fg: (u8, u8, u8),
    pub modified_bg: Color,
    pub deleted_fg: (u8, u8, u8),
    pub deleted_bg: Color,
    pub renamed_fg: (u8, u8, u8),
    pub typechange_fg: (u8, u8, u8),
    pub conflict_fg: (u8, u8, u8),
    pub conflict_bg: Color,
}

impl StatusStyle {
    fn dark() -> Self {
        Self {
            title_fg: (88, 166, 255),
            path_fg: (201, 209, 217),
            added_fg: (63, 185, 80),
            added_bg: Color::from_rgba8(46, 160, 67, 25),
            modified_fg: (210, 153, 34),
            modified_bg: Color::from_rgba8(210, 153, 34, 25),
            deleted_fg: (248, 81, 73),
            deleted_bg: Color::from_rgba8(248, 81, 73, 25),
            renamed_fg: (88, 166, 255),
            typechange_fg: (187, 128, 230),
            conflict_fg: (248, 81, 73),
            conflict_bg: Color::from_rgba8(248, 81, 73, 25),
        }
    }

    fn light() -> Self {
        Self {
            title_fg: (4, 81, 165),
            path_fg: (31, 35, 40),
            added_fg: (26, 127, 55),
            added_bg: Color::from_rgba8(46, 160, 67, 18),
            modified_fg: (154, 103, 0),
            modified_bg: Color::from_rgba8(210, 153, 34, 18),
            deleted_fg: (207, 34, 46),
            deleted_bg: Color::from_rgba8(248, 81, 73, 18),
            renamed_fg: (4, 81, 165),
            typechange_fg: (111, 78, 167),
            conflict_fg: (207, 34, 46),
            conflict_bg: Color::from_rgba8(248, 81, 73, 18),
        }
    }
}
