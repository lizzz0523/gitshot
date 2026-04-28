use std::path::PathBuf;
use std::process;

use git2::{DiffFormat, DiffOptions, Repository};
use tiny_skia::Pixmap;

use crate::config::{Config, DiffStyle, Style};
use crate::inline_diff::{InlineRange, annotate_inline_diffs};
use crate::renderer::Renderer;

struct DiffLine {
    origin: char,
    content: String,
    inline_ranges: Vec<InlineRange>,
}

pub fn run(config: &Config, paths: &[String], whitespace: bool) {
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

    let mut opts = DiffOptions::new();
    opts.include_untracked(true)
        .recurse_untracked_dirs(true)
        .show_untracked_content(true)
        .ignore_whitespace(!whitespace);

    for path_str in paths {
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

    // Collect raw lines first (without inline ranges)
    let mut raw_lines: Vec<(char, String, Vec<InlineRange>)> = Vec::new();
    diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
        let origin = line.origin();
        let content = String::from_utf8_lossy(line.content()).into_owned();

        if origin == 'F' && content.starts_with("diff --git") && !raw_lines.is_empty() {
            raw_lines.push(('\0', String::new(), Vec::new()));
        }

        raw_lines.push((origin, content, Vec::new()));
        true
    })
    .unwrap_or_else(|e| {
        eprintln!("error: failed to collect diff: {e}");
        process::exit(1);
    });

    if raw_lines.is_empty() {
        process::exit(0);
    }

    // Compute inline diffs for paired -/+ lines
    annotate_inline_diffs(&mut raw_lines);

    let lines: Vec<DiffLine> = raw_lines
        .into_iter()
        .map(|(origin, content, inline_ranges)| DiffLine {
            origin,
            content,
            inline_ranges,
        })
        .collect();

    let renderer = Renderer::new(config);
    let path = render_diff(&renderer, &lines, config);
    println!("{path}");
}

fn render_diff(renderer: &Renderer, lines: &[DiffLine], config: &Config) -> String {
    let style = &config.style;
    let (img_w, img_h) = layout_size(renderer, lines, style);

    let mut pixmap = Pixmap::new(img_w, img_h).expect("failed to create pixmap");
    pixmap.fill(style.canvas_bg);

    draw_lines(renderer, &mut pixmap, lines, img_w, style);

    Renderer::save_pixmap(&pixmap)
}

fn layout_size(renderer: &Renderer, lines: &[DiffLine], style: &Style) -> (u32, u32) {
    let max_line_w = lines
        .iter()
        .map(|l| renderer.measure_text_width(&format_line(l)))
        .fold(0.0f32, f32::max);

    let img_w =
        ((max_line_w + style.img_padding * 2.0).ceil() as u32).clamp(400, style.max_img_width);
    let img_h = (lines.len() as f32 * style.line_height + style.img_padding * 2.0).ceil() as u32;
    (img_w, img_h)
}

fn draw_lines(
    renderer: &Renderer,
    pixmap: &mut Pixmap,
    lines: &[DiffLine],
    img_w: u32,
    style: &Style,
) {
    let diff_style = &style.diff_style;

    for (i, line) in lines.iter().enumerate() {
        let y_top = style.img_padding + i as f32 * style.line_height;

        if line.origin == '\0' {
            renderer.draw_line_bg(
                pixmap,
                y_top,
                img_w,
                style.line_height,
                diff_style.separator_bg,
            );
            continue;
        }

        match line.origin {
            '+' => {
                renderer.draw_line_bg(pixmap, y_top, img_w, style.line_height, diff_style.added_bg);
                draw_inline_ranges(
                    renderer,
                    pixmap,
                    y_top,
                    style,
                    line,
                    diff_style.added_inline_bg,
                );
            }
            '-' => {
                renderer.draw_line_bg(
                    pixmap,
                    y_top,
                    img_w,
                    style.line_height,
                    diff_style.deleted_bg,
                );
                draw_inline_ranges(
                    renderer,
                    pixmap,
                    y_top,
                    style,
                    line,
                    diff_style.deleted_inline_bg,
                );
            }
            _ => {}
        }

        let text = format_line(line);
        let fg = line_color(line, &text, diff_style);
        renderer.draw_text(
            pixmap,
            &text,
            style.img_padding,
            renderer.centered_baseline(y_top, style.line_height),
            fg,
        );
    }
}

/// Draw darker background rectangles for inline-highlighted character ranges.
fn draw_inline_ranges(
    renderer: &Renderer,
    pixmap: &mut Pixmap,
    y_top: f32,
    style: &Style,
    line: &DiffLine,
    color: tiny_skia::Color,
) {
    if line.inline_ranges.is_empty() {
        return;
    }

    let prefix = match line.origin {
        '+' | '-' => line.origin.to_string(),
        _ => String::new(),
    };
    let content: Vec<char> = line.content.trim_end_matches('\n').chars().collect();
    let prefix_w = renderer.measure_text_width(&prefix);

    for range in &line.inline_ranges {
        let before: String = content.iter().take(range.start).collect();
        let range_str: String = content.iter().skip(range.start).take(range.end - range.start).collect();

        let x_start = style.img_padding + prefix_w + renderer.measure_text_width(&before);
        let x_end = x_start + renderer.measure_text_width(&range_str);

        renderer.draw_rect_bg(pixmap, x_start, y_top, x_end - x_start, style.line_height, color);
    }
}

fn format_line(line: &DiffLine) -> String {
    let content = line.content.trim_end_matches('\n');
    match line.origin {
        '+' | '-' | ' ' => format!("{}{content}", line.origin),
        _ => content.to_string(),
    }
}

fn line_color(line: &DiffLine, text: &str, diff: &DiffStyle) -> (u8, u8, u8) {
    match line.origin {
        '+' => diff.added_fg,
        '-' => diff.deleted_fg,
        'H' => diff.hunk_fg,
        'F' if text.starts_with("diff --git") => diff.file_fg,
        _ => diff.default_fg,
    }
}
