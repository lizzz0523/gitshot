use tiny_skia::Color;

#[derive(Default)]
pub struct Config {
    pub style: Style,
}

pub struct Style {
    pub font_path: String,
    pub font_size: f32,
    pub line_height: f32,
    pub padding: f32,
    pub max_img_width: u32,
    pub canvas_bg: Color,
    pub diff: DiffStyle,
    pub status: StatusStyle,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            font_path: "/System/Library/Fonts/Monaco.ttf".to_string(),
            font_size: 13.0,
            line_height: 20.0,
            padding: 16.0,
            max_img_width: 1800,
            canvas_bg: Color::from_rgba8(24, 24, 27, 255),
            diff: DiffStyle::default(),
            status: StatusStyle::default(),
        }
    }
}

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

impl Default for DiffStyle {
    fn default() -> Self {
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
}

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

impl Default for StatusStyle {
    fn default() -> Self {
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
}
