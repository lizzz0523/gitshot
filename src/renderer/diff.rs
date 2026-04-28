use crate::renderer::{Renderer, LINE_HEIGHT, MAX_IMG_WIDTH, PADDING};
use tiny_skia::{Color, Pixmap};

pub struct DiffLine {
    pub origin: char,
    pub content: String,
}

pub(super) fn render_diff(renderer: &Renderer, lines: &[DiffLine]) -> String {
    let max_line_w = lines
        .iter()
        .map(|l| renderer.measure_text_width(&format_line(l)))
        .fold(0.0f32, f32::max);

    let img_w = ((max_line_w + PADDING * 2.0).ceil() as u32).clamp(400, MAX_IMG_WIDTH);
    let img_h = (lines.len() as f32 * LINE_HEIGHT + PADDING * 2.0).ceil() as u32;

    let mut pixmap = Pixmap::new(img_w, img_h).expect("failed to create pixmap");
    pixmap.fill(Color::from_rgba8(24, 24, 27, 255));

    for (i, line) in lines.iter().enumerate() {
        let y_top = PADDING + i as f32 * LINE_HEIGHT;

        match line.origin {
            '+' => renderer.draw_line_bg(&mut pixmap, y_top, img_w, Color::from_rgba8(46, 160, 67, 30)),
            '-' => renderer.draw_line_bg(&mut pixmap, y_top, img_w, Color::from_rgba8(248, 81, 73, 30)),
            _ => {}
        }

        let text = format_line(line);
        let fg = line_color(line, &text);
        renderer.draw_text(&mut pixmap, &text, PADDING, renderer.centered_baseline(y_top), fg);
    }

    Renderer::save_pixmap(&pixmap)
}

fn format_line(line: &DiffLine) -> String {
    let content = line.content.trim_end_matches('\n');
    match line.origin {
        '+' | '-' | ' ' => format!("{}{content}", line.origin),
        _ => content.to_string(),
    }
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
