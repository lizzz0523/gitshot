use crate::renderer::{Renderer, LINE_HEIGHT, MAX_IMG_WIDTH, PADDING};
use tiny_skia::{Color, Pixmap};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StatusKind {
    None,
    Added,
    Modified,
    Deleted,
    Renamed,
    TypeChange,
    Conflict,
}

impl StatusKind {
    fn label(self) -> &'static str {
        match self {
            Self::None => "  ",
            Self::Added => "A ",
            Self::Modified => "M ",
            Self::Deleted => "D ",
            Self::Renamed => "R ",
            Self::TypeChange => "T ",
            Self::Conflict => "U ",
        }
    }

    fn color(self) -> (u8, u8, u8) {
        match self {
            Self::None => (201, 209, 217),
            Self::Added => (63, 185, 80),
            Self::Modified => (210, 153, 34),
            Self::Deleted => (248, 81, 73),
            Self::Renamed => (88, 166, 255),
            Self::TypeChange => (187, 128, 230),
            Self::Conflict => (248, 81, 73),
        }
    }

    fn bg_color(self) -> Option<Color> {
        match self {
            Self::Added => Some(Color::from_rgba8(46, 160, 67, 25)),
            Self::Modified => Some(Color::from_rgba8(210, 153, 34, 25)),
            Self::Deleted | Self::Conflict => Some(Color::from_rgba8(248, 81, 73, 25)),
            _ => None,
        }
    }
}

pub struct StatusEntry {
    pub path: String,
    pub staged: StatusKind,
    pub unstaged: StatusKind,
}

struct StatusSection {
    title: &'static str,
    entries: Vec<(StatusKind, String)>,
}

pub(super) fn render_status(renderer: &Renderer, entries: &[StatusEntry]) -> String {
    let mut staged = Vec::new();
    let mut unstaged = Vec::new();

    for entry in entries {
        if entry.staged != StatusKind::None {
            staged.push((entry.staged, entry.path.clone()));
        }
        if entry.unstaged != StatusKind::None {
            unstaged.push((entry.unstaged, entry.path.clone()));
        }
    }

    let sections = [
        StatusSection {
            title: "Staged changes",
            entries: staged,
        },
        StatusSection {
            title: "Unstaged changes",
            entries: unstaged,
        },
    ];

    let indicator_w = renderer.measure_text_width("XX  ");

    let max_path_w = entries
        .iter()
        .map(|e| renderer.measure_text_width(&e.path))
        .fold(0.0f32, f32::max);

    let max_title_w = sections
        .iter()
        .map(|s| renderer.measure_text_width(s.title))
        .fold(0.0f32, f32::max);

    let max_line_w = max_title_w.max(max_path_w + indicator_w);

    let img_w =
        ((max_line_w + PADDING * 2.0).ceil() as u32).clamp(400, MAX_IMG_WIDTH);

    let mut row_count = 0;
    for section in &sections {
        if !section.entries.is_empty() {
            row_count += 2 + section.entries.len(); // title + blank + entries
        }
    }
    // if both sections have entries, add a blank row between them
    let has_staged = !sections[0].entries.is_empty();
    let has_unstaged = !sections[1].entries.is_empty();
    if has_staged && has_unstaged {
        row_count += 1;
    }

    let content_h = row_count as f32 * LINE_HEIGHT;
    let img_h = (content_h + PADDING * 2.0).ceil() as u32;

    let mut pixmap = Pixmap::new(img_w, img_h).expect("failed to create pixmap");
    pixmap.fill(Color::from_rgba8(24, 24, 27, 255));

    let title_fg: (u8, u8, u8) = (88, 166, 255);
    let mut y = PADDING;

    let mut first = true;
    for section in &sections {
        if section.entries.is_empty() {
            continue;
        }

        // Blank line between sections
        if first {
            first = false;
        } else {
            y += LINE_HEIGHT;
        }

        // Section title
        renderer.draw_text(&mut pixmap, section.title, PADDING, renderer.centered_baseline(y), title_fg);
        y += LINE_HEIGHT;

        // Blank line after title
        y += LINE_HEIGHT;

        // Entries
        for (kind, path) in &section.entries {
            if let Some(bg) = kind.bg_color() {
                renderer.draw_line_bg(&mut pixmap, y, img_w, bg);
            }

            renderer.draw_text(
                &mut pixmap,
                kind.label(),
                PADDING,
                renderer.centered_baseline(y),
                kind.color(),
            );

            renderer.draw_text(
                &mut pixmap,
                path,
                PADDING + indicator_w,
                renderer.centered_baseline(y),
                (201, 209, 217),
            );

            y += LINE_HEIGHT;
        }
    }

    Renderer::save_pixmap(&pixmap)
}
