use clap::Parser;
use git2::{DiffFormat, DiffOptions, Repository};
use rusttype::{point, Font, Scale};
use std::fs;
use std::path::PathBuf;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};
use tiny_skia::{Color, FillRule, Paint, PathBuilder, Pixmap, Rect, Transform};

/// Render git diff as a PNG image
#[derive(Parser)]
#[command(name = "diffshot", version, about)]
struct Cli {
    /// Path(s) to diff (file or directory). Defaults to current directory.
    #[arg(default_values_t = vec![".".to_string()])]
    paths: Vec<String>,
}

struct DiffLine {
    origin: char,
    content: String,
}

const FONT_PATH: &str = "/System/Library/Fonts/Monaco.ttf";
const FONT_SIZE: f32 = 13.0;
const LINE_HEIGHT: f32 = 20.0;
const PADDING: f32 = 16.0;
const MAX_IMG_WIDTH: u32 = 1800;

fn main() {
    let cli = Cli::parse();

    let target: PathBuf = if cli.paths.len() == 1 && cli.paths[0] == "." {
        std::env::current_dir().unwrap_or_else(|e| {
            eprintln!("error: cannot get current directory: {e}");
            process::exit(1);
        })
    } else {
        PathBuf::from(&cli.paths[0])
    };

    let repo = Repository::discover(&target).unwrap_or_else(|e| {
        eprintln!("error: not a git repository: {e}");
        process::exit(1);
    });

    let workdir = repo.workdir().unwrap_or_else(|| {
        eprintln!("error: bare repository has no working directory");
        process::exit(1);
    });

    let mut opts = DiffOptions::new();
    opts.include_untracked(true)
        .recurse_untracked_dirs(true)
        .show_untracked_content(true);

    for path_str in &cli.paths {
        let p = PathBuf::from(path_str);
        if let Ok(canonical) = p.canonicalize()
            && let Ok(rel) = canonical.strip_prefix(workdir)
        {
            opts.pathspec(rel.to_string_lossy().into_owned());
        }
    }

    let diff = repo
        .diff_index_to_workdir(None, Some(&mut opts))
        .unwrap_or_else(|e| {
            eprintln!("error: failed to get diff: {e}");
            process::exit(1);
        });

    let mut lines = Vec::new();
    diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
        let origin = line.origin();
        let content = String::from_utf8_lossy(line.content()).into_owned();
        lines.push(DiffLine { origin, content });
        true
    })
    .unwrap_or_else(|e| {
        eprintln!("error: failed to collect diff: {e}");
        process::exit(1);
    });

    if lines.is_empty() {
        process::exit(0);
    }

    let font_data = fs::read(FONT_PATH).unwrap_or_else(|e| {
        eprintln!("error: failed to load font: {e}");
        process::exit(1);
    });
    let font = Font::try_from_vec(font_data).unwrap_or_else(|| {
        eprintln!("error: failed to parse font");
        process::exit(1);
    });

    let scale = Scale {
        x: FONT_SIZE,
        y: FONT_SIZE,
    };

    let max_line_w = lines
        .iter()
        .map(|l| measure_text_width(&font, &format_line(l), scale))
        .fold(0.0f32, f32::max);

    let img_w = ((max_line_w + PADDING * 2.0).ceil() as u32).clamp(400, MAX_IMG_WIDTH);
    let img_h = (lines.len() as f32 * LINE_HEIGHT + PADDING * 2.0).ceil() as u32;

    let mut pixmap = Pixmap::new(img_w, img_h).expect("failed to create pixmap");
    pixmap.fill(Color::from_rgba8(24, 24, 27, 255));

    let ascent = font.v_metrics(scale).ascent;

    for (i, line) in lines.iter().enumerate() {
        let y_top = PADDING + i as f32 * LINE_HEIGHT;

        match line.origin {
            '+' => draw_line_bg(&mut pixmap, y_top, img_w, Color::from_rgba8(46, 160, 67, 30)),
            '-' => draw_line_bg(&mut pixmap, y_top, img_w, Color::from_rgba8(248, 81, 73, 30)),
            _ => {}
        }

        let text = format_line(line);
        let fg = line_color(line, &text);
        draw_text(&mut pixmap, &font, &text, PADDING, y_top + ascent, scale, fg);
    }

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let path = format!("/tmp/diffshot_{ts}.png");

    let png_data = pixmap.encode_png().unwrap_or_else(|e| {
        eprintln!("error: failed to encode PNG: {e}");
        process::exit(1);
    });
    fs::write(&path, png_data).unwrap_or_else(|e| {
        eprintln!("error: failed to write PNG: {e}");
        process::exit(1);
    });

    println!("{path}");
}

fn format_line(line: &DiffLine) -> String {
    let content = line.content.trim_end_matches('\n');
    match line.origin {
        '+' | '-' | ' ' => format!("{}{content}", line.origin),
        _ => content.to_string(),
    }
}

fn measure_text_width(font: &Font, text: &str, scale: Scale) -> f32 {
    let mut width = 0.0;
    for c in text.chars() {
        let glyph = font.glyph(c).scaled(scale);
        width += glyph.h_metrics().advance_width;
    }
    width
}

fn line_color(line: &DiffLine, text: &str) -> (u8, u8, u8) {
    match line.origin {
        '+' => (63, 185, 80),
        '-' => (248, 81, 73),
        'H' => (187, 128, 230),
        'F' if text.starts_with("diff --git") => (88, 166, 255),
        _ => (201, 209, 217),
    }
}

fn draw_line_bg(pixmap: &mut Pixmap, y: f32, width: u32, color: Color) {
    let rect = Rect::from_xywh(0.0, y, width as f32, LINE_HEIGHT).expect("invalid rect");
    let path = PathBuilder::from_rect(rect);
    let mut paint = Paint::default();
    paint.set_color(color);
    pixmap.fill_path(
        &path,
        &paint,
        FillRule::Winding,
        Transform::identity(),
        None,
    );
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn draw_text(
    pixmap: &mut Pixmap,
    font: &Font,
    text: &str,
    x: f32,
    y: f32,
    scale: Scale,
    color: (u8, u8, u8),
) {
    for glyph in font.layout(text, scale, point(x, y)) {
        if let Some(bb) = glyph.pixel_bounding_box() {
            glyph.draw(|gx, gy, v| {
                let px = bb.min.x + gx as i32;
                let py = bb.min.y + gy as i32;
                if px >= 0 && py >= 0 {
                    let px_u = px as u32;
                    let py_u = py as u32;
                    if px_u < pixmap.width() && py_u < pixmap.height() {
                        let idx = ((py_u * pixmap.width() + px_u) * 4) as usize;
                        let data = pixmap.data_mut();
                        let bg_r = data[idx] as f32;
                        let bg_g = data[idx + 1] as f32;
                        let bg_b = data[idx + 2] as f32;

                        data[idx] = (bg_r * (1.0 - v) + f32::from(color.0) * v) as u8;
                        data[idx + 1] = (bg_g * (1.0 - v) + f32::from(color.1) * v) as u8;
                        data[idx + 2] = (bg_b * (1.0 - v) + f32::from(color.2) * v) as u8;
                    }
                }
            });
        }
    }
}
