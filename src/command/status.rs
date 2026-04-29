use std::path::{Path, PathBuf};

use anyhow::Result;
use tiny_skia::Pixmap;

use crate::cli::StatusArgs;
use crate::config::Config;
use crate::model::status::{StatusEntry, StatusKind};
use crate::renderer::Renderer;

pub fn run(cfg: &Config, args: &StatusArgs) -> Result<()> {
    let (repo, pathspecs) = super::open_repo_and_pathspecs(&args.paths)?;
    let entries = StatusEntry::from_repo(&repo, &pathspecs)?;
    if entries.is_empty() {
        return Ok(());
    }

    let renderer = Renderer::new(cfg)?;
    let path = StatusView::new(&entries, &renderer, cfg).render(args.output.output.as_deref())?;
    println!("{}", path.display());
    Ok(())
}

struct StatusView<'a> {
    entries: &'a [StatusEntry],
    renderer: &'a Renderer,
    cfg: &'a Config,
}

struct StatusEntryView<'a> {
    kind: StatusKind,
    path: &'a str,
}

struct StatusSection<'a> {
    title: &'static str,
    entries: Vec<StatusEntryView<'a>>,
}

impl<'a> StatusView<'a> {
    fn new(entries: &'a [StatusEntry], renderer: &'a Renderer, cfg: &'a Config) -> Self {
        Self {
            entries,
            renderer,
            cfg,
        }
    }

    fn render(&self, output: Option<&Path>) -> Result<PathBuf> {
        let style = &self.cfg.style;
        let sections = self.build_sections();
        let (img_w, img_h, indicator_w) = self.layout_size(&sections);

        let mut pixmap = Pixmap::new(img_w, img_h).expect("failed to create pixmap");
        pixmap.fill(style.canvas_bg);
        self.draw_sections(&mut pixmap, &sections, img_w, indicator_w);

        Renderer::save_pixmap(&pixmap, output)
    }

    fn build_sections(&self) -> [StatusSection<'_>; 2] {
        let mut staged = Vec::new();
        let mut unstaged = Vec::new();

        for entry in self.entries {
            if entry.staged != StatusKind::None {
                staged.push(StatusEntryView {
                    kind: entry.staged,
                    path: &entry.path,
                });
            }
            if entry.unstaged != StatusKind::None {
                unstaged.push(StatusEntryView {
                    kind: entry.unstaged,
                    path: &entry.path,
                });
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

    fn layout_size(&self, sections: &[StatusSection<'_>; 2]) -> (u32, u32, f32) {
        let style = &self.cfg.style;

        let indicator_w = self.renderer.measure_text_width("XX  ");
        let max_path_w = self
            .entries
            .iter()
            .map(|e| self.renderer.measure_text_width(&e.path))
            .fold(0.0f32, f32::max);
        let max_title_w = sections
            .iter()
            .map(|s| self.renderer.measure_text_width(s.title))
            .fold(0.0f32, f32::max);

        let max_line_w = max_title_w.max(max_path_w + indicator_w);
        let img_w =
            ((max_line_w + style.img_padding * 2.0).ceil() as u32).clamp(400, style.max_img_width);

        // 每段贡献 (title + blank + entries) 行，段之间再各加一空行分隔
        let non_empty = sections.iter().filter(|s| !s.entries.is_empty());
        let row_count: usize = non_empty
            .clone()
            .map(|s| 2 + s.entries.len())
            .sum::<usize>()
            + non_empty.count().saturating_sub(1);

        let img_h = (row_count as f32 * style.line_height + style.img_padding * 2.0).ceil() as u32;
        (img_w, img_h, indicator_w)
    }

    fn draw_sections(
        &self,
        pixmap: &mut Pixmap,
        sections: &[StatusSection<'_>; 2],
        img_w: u32,
        indicator_w: f32,
    ) {
        let style = &self.cfg.style;
        let status_style = &style.status_style;

        let mut y = style.img_padding;

        for (i, section) in sections
            .iter()
            .filter(|s| !s.entries.is_empty())
            .enumerate()
        {
            if i > 0 {
                y += style.line_height;
            }

            self.renderer.draw_text(
                pixmap,
                section.title,
                style.img_padding,
                self.renderer.centered_baseline(y, style.line_height),
                status_style.title_fg,
            );
            y += style.line_height * 2.0;

            for entry in &section.entries {
                if let Some(bg_color) = entry.kind.bg_color(status_style) {
                    self.renderer.draw_rect(
                        pixmap,
                        0.0,
                        y,
                        img_w as f32,
                        style.line_height,
                        bg_color,
                    );
                }

                let baseline = self.renderer.centered_baseline(y, style.line_height);
                self.renderer.draw_text(
                    pixmap,
                    entry.kind.label(),
                    style.img_padding,
                    baseline,
                    entry.kind.fg_color(status_style),
                );
                self.renderer.draw_text(
                    pixmap,
                    entry.path,
                    style.img_padding + indicator_w,
                    baseline,
                    status_style.path_fg,
                );

                y += style.line_height;
            }
        }
    }
}
