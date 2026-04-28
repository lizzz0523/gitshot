use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, anyhow};
use rusttype::{Font, Scale, point};
use tiny_skia::{Color, FillRule, Paint, PathBuilder, Pixmap, Rect, Transform};

use crate::config::Config;

pub struct Renderer {
    font: Font<'static>,
    scale: Scale,
}

impl Renderer {
    pub fn new(cfg: &Config) -> Result<Self> {
        let path = &cfg.style.font_path;
        let font_data = fs::read(path).with_context(|| format!("failed to load font: {path}"))?;
        let font =
            Font::try_from_vec(font_data).ok_or_else(|| anyhow!("failed to parse font: {path}"))?;
        let scale = Scale {
            x: cfg.style.font_size,
            y: cfg.style.font_size,
        };
        Ok(Self { font, scale })
    }

    pub fn centered_baseline(&self, y_top: f32, line_height: f32) -> f32 {
        let metrics = self.font.v_metrics(self.scale);
        let text_height = metrics.ascent - metrics.descent;
        y_top + (line_height - text_height) / 2.0 + metrics.ascent
    }

    pub fn measure_text_width(&self, text: &str) -> f32 {
        text.chars()
            .map(|c| {
                self.font
                    .glyph(c)
                    .scaled(self.scale)
                    .h_metrics()
                    .advance_width
            })
            .sum()
    }

    pub fn draw_rect(&self, pixmap: &mut Pixmap, x: f32, y: f32, w: f32, h: f32, color: Color) {
        if w <= 0.0 || h <= 0.0 {
            return;
        }
        let rect = Rect::from_xywh(x, y, w, h).expect("invalid rect");
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

    pub fn draw_text(&self, pixmap: &mut Pixmap, text: &str, x: f32, y: f32, color: Color) {
        // tiny_skia 的 Color 以 f32 0.0-1.0 存储；这里换算成 0-255 用于逐像素混合
        let fg_r = color.red() * 255.0;
        let fg_g = color.green() * 255.0;
        let fg_b = color.blue() * 255.0;
        let color_alpha = color.alpha();

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

                        let alpha = color_alpha * v;
                        data[idx] = blend_channel(bg_r, fg_r, alpha);
                        data[idx + 1] = blend_channel(bg_g, fg_g, alpha);
                        data[idx + 2] = blend_channel(bg_b, fg_b, alpha);
                    }
                });
            }
        }
    }

    pub fn save_pixmap(pixmap: &Pixmap) -> Result<String> {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is before UNIX epoch")
            .as_millis();
        let path = format!("/tmp/gitshot_{ts}.png");

        let png_data = pixmap.encode_png().context("failed to encode PNG")?;
        fs::write(&path, png_data).with_context(|| format!("failed to write PNG: {path}"))?;

        Ok(path)
    }
}

fn blend_channel(bg: f32, fg: f32, alpha: f32) -> u8 {
    let result = bg * (1.0 - alpha) + fg * alpha;
    result.round().clamp(0.0, 255.0) as u8
}
