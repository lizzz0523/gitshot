use git2::{Repository, Status, StatusOptions};
use std::path::PathBuf;
use std::process;

use crate::renderer::{status::StatusEntry, status::StatusKind, Renderer};

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
    let path = renderer.render_status(&entries);
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
