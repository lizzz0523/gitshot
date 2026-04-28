use tiny_skia::Color;

use super::{ColorScheme, ConfigFile};

pub struct Style {
    pub font_path: String,
    pub font_size: f32,
    pub line_height: f32,
    pub img_padding: f32,
    pub max_img_width: u32,
    pub canvas_bg: Color,
    pub diff_style: DiffStyle,
    pub status_style: StatusStyle,
}

impl Style {
    pub fn from_config(cfg: &ConfigFile) -> Self {
        let (canvas_bg, diff_style, status_style) = cfg.color_scheme.build();
        Self {
            font_path: cfg.font_path.clone(),
            font_size: cfg.font_size,
            line_height: cfg.line_height,
            img_padding: cfg.img_padding,
            max_img_width: cfg.max_img_width,
            canvas_bg,
            diff_style,
            status_style,
        }
    }
}

impl ColorScheme {
    fn build(self) -> (Color, DiffStyle, StatusStyle) {
        match self {
            Self::Dark => (
                Color::from_rgba8(24, 24, 27, 255),
                DiffStyle::dark(),
                StatusStyle::dark(),
            ),
            Self::Light => (
                Color::from_rgba8(255, 255, 255, 255),
                DiffStyle::light(),
                StatusStyle::light(),
            ),
        }
    }
}

// ── DiffStyle ───────────────────────────────────────────────────────

pub struct DiffStyle {
    pub section_title_fg: Color,
    pub added_bg: Color,
    pub deleted_bg: Color,
    pub added_inline_bg: Color,
    pub deleted_inline_bg: Color,
    pub separator_bg: Color,
    pub added_fg: Color,
    pub deleted_fg: Color,
    pub hunk_fg: Color,
    pub file_fg: Color,
    pub default_fg: Color,
}

impl DiffStyle {
    fn dark() -> Self {
        Self {
            section_title_fg: Color::from_rgba8(88, 166, 255, 255),
            added_bg: Color::from_rgba8(46, 160, 67, 30),
            deleted_bg: Color::from_rgba8(248, 81, 73, 30),
            added_inline_bg: Color::from_rgba8(46, 160, 67, 70),
            deleted_inline_bg: Color::from_rgba8(248, 81, 73, 70),
            separator_bg: Color::from_rgba8(48, 54, 61, 255),
            added_fg: Color::from_rgba8(63, 185, 80, 255),
            deleted_fg: Color::from_rgba8(248, 81, 73, 255),
            hunk_fg: Color::from_rgba8(187, 128, 230, 255),
            file_fg: Color::from_rgba8(88, 166, 255, 255),
            default_fg: Color::from_rgba8(201, 209, 217, 255),
        }
    }

    fn light() -> Self {
        Self {
            section_title_fg: Color::from_rgba8(4, 81, 165, 255),
            added_bg: Color::from_rgba8(46, 160, 67, 25),
            deleted_bg: Color::from_rgba8(248, 81, 73, 25),
            added_inline_bg: Color::from_rgba8(46, 160, 67, 55),
            deleted_inline_bg: Color::from_rgba8(248, 81, 73, 55),
            separator_bg: Color::from_rgba8(216, 222, 228, 255),
            added_fg: Color::from_rgba8(26, 127, 55, 255),
            deleted_fg: Color::from_rgba8(207, 34, 46, 255),
            hunk_fg: Color::from_rgba8(111, 78, 167, 255),
            file_fg: Color::from_rgba8(4, 81, 165, 255),
            default_fg: Color::from_rgba8(31, 35, 40, 255),
        }
    }
}

pub struct StatusStyle {
    pub title_fg: Color,
    pub path_fg: Color,
    pub added_fg: Color,
    pub added_bg: Color,
    pub modified_fg: Color,
    pub modified_bg: Color,
    pub deleted_fg: Color,
    pub deleted_bg: Color,
    pub renamed_fg: Color,
    pub typechange_fg: Color,
    pub conflict_fg: Color,
    pub conflict_bg: Color,
}

impl StatusStyle {
    fn dark() -> Self {
        Self {
            title_fg: Color::from_rgba8(88, 166, 255, 255),
            path_fg: Color::from_rgba8(201, 209, 217, 255),
            added_fg: Color::from_rgba8(63, 185, 80, 255),
            added_bg: Color::from_rgba8(46, 160, 67, 25),
            modified_fg: Color::from_rgba8(210, 153, 34, 255),
            modified_bg: Color::from_rgba8(210, 153, 34, 25),
            deleted_fg: Color::from_rgba8(248, 81, 73, 255),
            deleted_bg: Color::from_rgba8(248, 81, 73, 25),
            renamed_fg: Color::from_rgba8(88, 166, 255, 255),
            typechange_fg: Color::from_rgba8(187, 128, 230, 255),
            conflict_fg: Color::from_rgba8(248, 81, 73, 255),
            conflict_bg: Color::from_rgba8(248, 81, 73, 25),
        }
    }

    fn light() -> Self {
        Self {
            title_fg: Color::from_rgba8(4, 81, 165, 255),
            path_fg: Color::from_rgba8(31, 35, 40, 255),
            added_fg: Color::from_rgba8(26, 127, 55, 255),
            added_bg: Color::from_rgba8(46, 160, 67, 18),
            modified_fg: Color::from_rgba8(154, 103, 0, 255),
            modified_bg: Color::from_rgba8(210, 153, 34, 18),
            deleted_fg: Color::from_rgba8(207, 34, 46, 255),
            deleted_bg: Color::from_rgba8(248, 81, 73, 18),
            renamed_fg: Color::from_rgba8(4, 81, 165, 255),
            typechange_fg: Color::from_rgba8(111, 78, 167, 255),
            conflict_fg: Color::from_rgba8(207, 34, 46, 255),
            conflict_bg: Color::from_rgba8(248, 81, 73, 18),
        }
    }
}
