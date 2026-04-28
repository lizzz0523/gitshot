use anyhow::Result;
use tiny_skia::{Color, Pixmap};

use crate::config::Config;
use crate::model::diff::{DiffLine, LineKind};
use crate::renderer::Renderer;

pub fn run(cfg: &Config, paths: &[String], whitespace: bool) -> Result<()> {
    let (repo, pathspecs) = super::open_repo_and_pathspecs(paths)?;
    let lines = DiffLine::from_repo(&repo, &pathspecs, whitespace)?;
    if lines.is_empty() {
        return Ok(());
    }

    let renderer = Renderer::new(cfg)?;
    let path = DiffView::new(&lines, &renderer, cfg).render()?;
    println!("{path}");
    Ok(())
}

struct DiffView<'a> {
    lines: &'a [DiffLine],
    renderer: &'a Renderer,
    cfg: &'a Config,
}

impl<'a> DiffView<'a> {
    fn new(lines: &'a [DiffLine], renderer: &'a Renderer, cfg: &'a Config) -> Self {
        Self {
            lines,
            renderer,
            cfg,
        }
    }

    fn render(&self) -> Result<String> {
        let style = &self.cfg.style;
        let (img_w, img_h) = self.layout_size();

        let mut pixmap = Pixmap::new(img_w, img_h).expect("failed to create pixmap");
        pixmap.fill(style.canvas_bg);
        self.draw_lines(&mut pixmap, img_w);

        Renderer::save_pixmap(&pixmap)
    }

    fn layout_size(&self) -> (u32, u32) {
        let style = &self.cfg.style;

        let max_line_w = self
            .lines
            .iter()
            .map(|l| self.measure_line_width(l))
            .fold(0.0f32, f32::max);

        let img_w =
            ((max_line_w + style.img_padding * 2.0).ceil() as u32).clamp(400, style.max_img_width);
        let img_h =
            (self.lines.len() as f32 * style.line_height + style.img_padding * 2.0).ceil() as u32;
        (img_w, img_h)
    }

    fn measure_line_width(&self, line: &DiffLine) -> f32 {
        self.renderer.measure_text_width(line.kind.prefix())
            + self.renderer.measure_text_width(&line.content)
    }

    fn draw_lines(&self, pixmap: &mut Pixmap, img_w: u32) {
        let style = &self.cfg.style;
        let diff = &style.diff_style;

        for (i, line) in self.lines.iter().enumerate() {
            let y_top = style.img_padding + i as f32 * style.line_height;

            match line.kind {
                LineKind::Separator => {
                    self.renderer.draw_rect(
                        pixmap,
                        0.0,
                        y_top,
                        img_w as f32,
                        style.line_height,
                        diff.separator_bg,
                    );
                    continue;
                }
                LineKind::Added => {
                    self.renderer.draw_rect(
                        pixmap,
                        0.0,
                        y_top,
                        img_w as f32,
                        style.line_height,
                        diff.added_bg,
                    );
                    self.draw_inline_ranges(pixmap, y_top, line, diff.added_inline_bg);
                }
                LineKind::Deleted => {
                    self.renderer.draw_rect(
                        pixmap,
                        0.0,
                        y_top,
                        img_w as f32,
                        style.line_height,
                        diff.deleted_bg,
                    );
                    self.draw_inline_ranges(pixmap, y_top, line, diff.deleted_inline_bg);
                }
                _ => {}
            }

            // prefix + content 合并为一次 draw_text 调用，避免分两次时的 kerning 偏移
            let text = format!("{}{}", line.kind.prefix(), line.content);
            let baseline = self.renderer.centered_baseline(y_top, style.line_height);
            self.renderer.draw_text(
                pixmap,
                &text,
                style.img_padding,
                baseline,
                line.kind.color(diff),
            );
        }
    }

    fn draw_inline_ranges(&self, pixmap: &mut Pixmap, y_top: f32, line: &DiffLine, color: Color) {
        if line.inline_ranges.is_empty() {
            return;
        }
        let style = &self.cfg.style;
        let prefix = line.kind.prefix();
        let prefix_w = self.renderer.measure_text_width(prefix);
        let content = line.content.as_str();

        for r in &line.inline_ranges {
            let before = &content[..r.start];
            let span = &content[r.start..r.end];
            let x_start = style.img_padding + prefix_w + self.renderer.measure_text_width(before);
            let x_end = x_start + self.renderer.measure_text_width(span);
            self.renderer.draw_rect(
                pixmap,
                x_start,
                y_top,
                x_end - x_start,
                style.line_height,
                color,
            );
        }
    }
}
