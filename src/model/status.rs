use anyhow::{Context, Result};
use git2::{Repository, Status, StatusOptions};
use tiny_skia::Color;

use crate::config::StatusStyle;

pub struct StatusEntry {
    pub path: String,
    pub staged: StatusKind,
    pub unstaged: StatusKind,
}

impl StatusEntry {
    pub fn from_repo(repo: &Repository, pathspecs: &[String]) -> Result<Vec<Self>> {
        let mut opts = StatusOptions::new();
        opts.include_untracked(true).recurse_untracked_dirs(true);
        for ps in pathspecs {
            opts.pathspec(ps);
        }

        let statuses = repo
            .statuses(Some(&mut opts))
            .context("failed to get status")?;

        let entries = statuses
            .iter()
            .map(|e| {
                let s = e.status();
                Self {
                    path: e.path().unwrap_or("???").to_owned(),
                    staged: classify(s, STAGED_TABLE),
                    unstaged: classify(s, UNSTAGED_TABLE),
                }
            })
            .collect();

        Ok(entries)
    }
}

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
    pub fn label(self) -> &'static str {
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

    pub fn fg_color(self, style: &StatusStyle) -> Color {
        match self {
            Self::None => style.path_fg,
            Self::Added => style.added_fg,
            Self::Modified => style.modified_fg,
            Self::Deleted => style.deleted_fg,
            Self::Renamed => style.renamed_fg,
            Self::TypeChange => style.typechange_fg,
            Self::Conflict => style.conflict_fg,
        }
    }

    pub fn bg_color(self, style: &StatusStyle) -> Option<Color> {
        match self {
            Self::Added => Some(style.added_bg),
            Self::Modified => Some(style.modified_bg),
            Self::Deleted => Some(style.deleted_bg),
            Self::Conflict => Some(style.conflict_bg),
            _ => None,
        }
    }
}

// 按顺序首个命中的 flag 胜出，保留原 if/else-if 链的优先级
const STAGED_TABLE: &[(Status, StatusKind)] = &[
    (Status::INDEX_NEW, StatusKind::Added),
    (Status::INDEX_MODIFIED, StatusKind::Modified),
    (Status::INDEX_DELETED, StatusKind::Deleted),
    (Status::INDEX_RENAMED, StatusKind::Renamed),
    (Status::INDEX_TYPECHANGE, StatusKind::TypeChange),
];

const UNSTAGED_TABLE: &[(Status, StatusKind)] = &[
    (Status::WT_NEW, StatusKind::Added),
    (Status::WT_MODIFIED, StatusKind::Modified),
    (Status::WT_DELETED, StatusKind::Deleted),
    (Status::WT_RENAMED, StatusKind::Renamed),
    (Status::WT_TYPECHANGE, StatusKind::TypeChange),
    (Status::CONFLICTED, StatusKind::Conflict),
];

fn classify(status: Status, table: &[(Status, StatusKind)]) -> StatusKind {
    table
        .iter()
        .find(|(flag, _)| status.contains(*flag))
        .map(|(_, kind)| *kind)
        .unwrap_or(StatusKind::None)
}
