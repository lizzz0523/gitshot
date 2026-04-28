use git2::{Repository, Status, StatusOptions};
use std::path::PathBuf;
use std::process;

use crate::renderer::{LINE_HEIGHT, MAX_IMG_WIDTH, PADDING, Renderer};
use tiny_skia::{Color, Pixmap};

#[derive(Clone, Copy, PartialEq, Eq)]
enum StatusKind {
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

struct StatusEntry {
    path: String,
    staged: StatusKind,
    unstaged: StatusKind,
}

struct StatusSection {
    title: &'static str,
    entries: Vec<(StatusKind, String)>,
}

pub fn run(paths: &[String]) {
    let target: PathBuf = if paths.len() == 1 && paths[0] == "." {
        std::env::current_dir().unwrap_or_else(|e| {
            eprintln!("error: cannot get current directory: {e}");
            process::exit(1);
        })
    } else {
        PathBuf::from(&paths[0])
    };

    let repo = Repository::discover(&target).unwrap_or_else(|e| {
        eprintln!("error: not a git repository: {e}");
        process::exit(1);
    });

    let workdir = repo.workdir().unwrap_or_else(|| {
        eprintln!("error: bare repository has no working directory");
        process::exit(1);
    });

    let mut opts = StatusOptions::new();
    opts.include_untracked(true).recurse_untracked_dirs(true);

    for path_str in paths {
        let p = PathBuf::from(path_str);
        if let Ok(canonical) = p.canonicalize()
            && let Ok(rel) = canonical.strip_prefix(workdir)
        {
            opts.pathspec(rel.to_string_lossy().into_owned());
        }
    }

    let statuses = repo.statuses(Some(&mut opts)).unwrap_or_else(|e| {
        eprintln!("error: failed to get status: {e}");
        process::exit(1);
    });

    let mut entries = Vec::new();

    for entry in statuses.iter() {
        let path = entry.path().unwrap_or("???").to_string();
        let status = entry.status();

        let staged = classify_staged(status);
        let unstaged = classify_unstaged(status);

        entries.push(StatusEntry {
            path,
            staged,
            unstaged,
        });
    }

    if entries.is_empty() {
        process::exit(0);
    }

    let renderer = Renderer::new();
    let path = render_status(&renderer, &entries);
    println!("{path}");
}

fn classify_staged(status: Status) -> StatusKind {
    if status.contains(Status::INDEX_NEW) {
        StatusKind::Added
    } else if status.contains(Status::INDEX_MODIFIED) {
        StatusKind::Modified
    } else if status.contains(Status::INDEX_DELETED) {
        StatusKind::Deleted
    } else if status.contains(Status::INDEX_RENAMED) {
        StatusKind::Renamed
    } else if status.contains(Status::INDEX_TYPECHANGE) {
        StatusKind::TypeChange
    } else {
        StatusKind::None
    }
}

fn classify_unstaged(status: Status) -> StatusKind {
    if status.contains(Status::WT_NEW) {
        StatusKind::Added
    } else if status.contains(Status::WT_MODIFIED) {
        StatusKind::Modified
    } else if status.contains(Status::WT_DELETED) {
        StatusKind::Deleted
    } else if status.contains(Status::WT_RENAMED) {
        StatusKind::Renamed
    } else if status.contains(Status::WT_TYPECHANGE) {
        StatusKind::TypeChange
    } else if status.contains(Status::CONFLICTED) {
        StatusKind::Conflict
    } else {
        StatusKind::None
    }
}

fn render_status(renderer: &Renderer, entries: &[StatusEntry]) -> String {
    let sections = build_sections(entries);

    let (img_w, img_h, indicator_w) = layout_size(renderer, &sections, entries);
    let mut pixmap = Pixmap::new(img_w, img_h).expect("failed to create pixmap");
    pixmap.fill(Color::from_rgba8(24, 24, 27, 255));

    draw_sections(renderer, &mut pixmap, &sections, img_w, indicator_w);

    Renderer::save_pixmap(&pixmap)
}

fn build_sections(entries: &[StatusEntry]) -> [StatusSection; 2] {
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

    [
        StatusSection {
            title: "Staged changes",
            entries: staged,
        },
        StatusSection {
            title: "Unstaged changes",
            entries: unstaged,
        },
    ]
}

fn layout_size(
    renderer: &Renderer,
    sections: &[StatusSection; 2],
    entries: &[StatusEntry],
) -> (u32, u32, f32) {
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
    let img_w = ((max_line_w + PADDING * 2.0).ceil() as u32).clamp(400, MAX_IMG_WIDTH);

    let mut row_count = 0usize;
    for section in sections {
        if !section.entries.is_empty() {
            row_count += 2 + section.entries.len();
        }
    }
    let has_staged = !sections[0].entries.is_empty();
    let has_unstaged = !sections[1].entries.is_empty();
    if has_staged && has_unstaged {
        row_count += 1;
    }

    let img_h = (row_count as f32 * LINE_HEIGHT + PADDING * 2.0).ceil() as u32;
    (img_w, img_h, indicator_w)
}

fn draw_sections(
    renderer: &Renderer,
    pixmap: &mut Pixmap,
    sections: &[StatusSection; 2],
    img_w: u32,
    indicator_w: f32,
) {
    const TITLE_FG: (u8, u8, u8) = (88, 166, 255);
    const PATH_FG: (u8, u8, u8) = (201, 209, 217);

    let mut y = PADDING;
    let mut first = true;

    for section in sections {
        if section.entries.is_empty() {
            continue;
        }

        if first {
            first = false;
        } else {
            y += LINE_HEIGHT;
        }

        // Title
        renderer.draw_text(
            pixmap,
            section.title,
            PADDING,
            renderer.centered_baseline(y),
            TITLE_FG,
        );
        y += LINE_HEIGHT * 2.0; // title + blank

        // Entries
        for (kind, path) in &section.entries {
            if let Some(bg) = kind.bg_color() {
                renderer.draw_line_bg(pixmap, y, img_w, bg);
            }

            renderer.draw_text(
                pixmap,
                kind.label(),
                PADDING,
                renderer.centered_baseline(y),
                kind.color(),
            );
            renderer.draw_text(
                pixmap,
                path,
                PADDING + indicator_w,
                renderer.centered_baseline(y),
                PATH_FG,
            );

            y += LINE_HEIGHT;
        }
    }
}
