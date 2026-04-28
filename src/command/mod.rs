use std::env;
use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};
use git2::Repository;

pub mod diff;
pub mod status;

pub(crate) fn open_repo_and_pathspecs(paths: &[String]) -> Result<(Repository, Vec<String>)> {
    let start: PathBuf = match paths.first() {
        Some(p) if p == "." => env::current_dir().context("cannot get current directory")?,
        Some(p) => PathBuf::from(p),
        None => env::current_dir().context("cannot get current directory")?,
    };

    let repo = Repository::discover(&start)
        .with_context(|| format!("not a git repository: {}", start.display()))?;

    let workdir = repo
        .workdir()
        .ok_or_else(|| anyhow!("bare repository has no working directory"))?
        .to_path_buf();

    // 任意位于 worktree 内的路径都能 discover 到同一 repo，所以只取首个
    // 作为起点；canonicalize/strip_prefix 失败的路径静默丢弃。
    let pathspecs = paths
        .iter()
        .filter_map(|p| {
            let canonical = PathBuf::from(p).canonicalize().ok()?;
            let rel = canonical.strip_prefix(&workdir).ok()?;
            Some(rel.to_string_lossy().into_owned())
        })
        .collect();

    Ok((repo, pathspecs))
}
