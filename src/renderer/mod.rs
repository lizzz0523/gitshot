pub mod diff;
pub mod status;

use rusttype::{Font, Scale, point};
use std::fs;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};
use tiny_skia::{Color, FillRule, Paint, PathBuilder, Pixmap, Rect, Transform};

const FONT_PATH: &str = "/System/Library/Fonts/Monaco.ttf";
const FONT_SIZE: f32 = 13.0;
const LINE_HEIGHT: f32 = 20.0;
const PADDING: f32 = 16.0;
const MAX_IMG_WIDTH: u32 = 1800;

pub struct Renderer {
    font: Font<'static>,
    scale: Scale,
}

impl Renderer {
    pub fn new() -> Self {
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
        Self { font, scale }
    }

    pub fn render_diff(&self, lines: &[diff::DiffLine]) -> String {
        diff::render_diff(self, lines)
    }

    pub fn render_status(&self, entries: &[status::StatusEntry]) -> String {
        status::render_status(self, entries)
    }

    /// Compute the baseline y for text vertically centered within a row
    /// starting at `y_top` with height `LINE_HEIGHT`.
    fn centered_baseline(&self, y_top: f32) -> f32 {
        let metrics = self.font.v_metrics(self.scale);
        let text_height = metrics.ascent - metrics.descent;
        y_top + (LINE_HEIGHT - text_height) / 2.0 + metrics.ascent
    }

    fn measure_text_width(&self, text: &str) -> f32 {
        let mut width = 0.0;
        for c in text.chars() {
            let glyph = self.font.glyph(c).scaled(self.scale);
            width += glyph.h_metrics().advance_width;
        }
        width
    }

    fn draw_line_bg(&self, pixmap: &mut Pixmap, y: f32, width: u32, color: Color) {
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

    fn draw_text(&self, pixmap: &mut Pixmap, text: &str, x: f32, y: f32, color: (u8, u8, u8)) {
        for glyph in self.font.layout(text, self.scale, point(x, y)) {
            if let Some(bb) = glyph.pixel_bounding_box() {
                glyph.draw(|gx, gy, v| {
                    let px = bb.min.x + gx as i32;
                    let py = bb.min.y + gy as i32;
                    if let (Ok(px_u), Ok(py_u)) = (u32::try_from(px), u32::try_from(py))
                        && px_u < pixmap.width()
                        && py_u < pixmap.height()
                    {
                        let idx = ((py_u * pixmap.width() + px_u) * 4) as usize;
                        let data = pixmap.data_mut();
                        let bg_r = f32::from(data[idx]);
                        let bg_g = f32::from(data[idx + 1]);
                        let bg_b = f32::from(data[idx + 2]);

                        data[idx] = blend_channel(bg_r, color.0, v);
                        data[idx + 1] = blend_channel(bg_g, color.1, v);
                        data[idx + 2] = blend_channel(bg_b, color.2, v);
                    }
                });
            }
        }
    }

    fn save_pixmap(pixmap: &Pixmap) -> String {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let path = format!("/tmp/gitshot_{ts}.png");

        let png_data = pixmap.encode_png().unwrap_or_else(|e| {
            eprintln!("error: failed to encode PNG: {e}");
            process::exit(1);
        });
        fs::write(&path, png_data).unwrap_or_else(|e| {
            eprintln!("error: failed to write PNG: {e}");
            process::exit(1);
        });

        path
    }
}

/// Blend a foreground channel onto a background channel with the given alpha.
fn blend_channel(bg: f32, fg: u8, alpha: f32) -> u8 {
    let result = bg * (1.0 - alpha) + f32::from(fg) * alpha;
    result.round().clamp(0.0, 255.0) as u8
}
