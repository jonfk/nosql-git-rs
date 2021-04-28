use crate::error::GitDataStoreError;
use chrono::{FixedOffset, TimeZone};
use git2::{Commit, Oid, Repository, Time};
use serde::Serialize;

#[derive(Serialize)]
pub struct History {
    commits: Vec<HistoryEntry>,
}

#[derive(Serialize)]
pub struct HistoryEntry {
    timestamp_seconds: i64,
    commit_id: String,
    author: String,
    stats: HistoryStats,
}

#[derive(Serialize)]
pub struct HistoryStats {
    files_changed: usize,
    insertions: usize,
    deletions: usize,
}

pub fn git_log(repo: &Repository, skip: usize, limit: usize) -> Result<History, GitDataStoreError> {
    let mut rev_walk = repo.revwalk()?;
    rev_walk.push_head()?;

    let history: Result<Vec<HistoryEntry>, GitDataStoreError> = rev_walk
        .into_iter()
        .skip(skip)
        .enumerate()
        .take_while(|(i, _rev)| *i < limit)
        .map(|(_i, rev)| map_rev(repo, rev))
        .collect();

    Ok(History { commits: history? })
}

fn map_rev(
    repo: &Repository,
    rev: Result<Oid, git2::Error>,
) -> Result<HistoryEntry, GitDataStoreError> {
    let rev = rev?;

    let commit = repo.find_commit(rev)?;
    let commit_tree = commit.tree()?;

    let parent = commit.parents().next();
    let parent_tree = parent.map(|p| p.tree()).transpose()?;

    let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&commit_tree), None)?;
    let diff_stats = diff.stats()?;
    let stats = HistoryStats {
        files_changed: diff_stats.files_changed(),
        insertions: diff_stats.insertions(),
        deletions: diff_stats.deletions(),
    };

    let x = Ok(HistoryEntry {
        timestamp_seconds: commit.time().seconds(),
        commit_id: commit.id().to_string(),
        author: commit.author().to_string(),
        stats: stats,
    });
    x
}

pub fn print_commit(commit: Commit) -> String {
    format!(
        "commit {}\nAuthor: {}\nDate: {}\n\n{}\n",
        commit.id().to_string(),
        commit.author().to_string(),
        print_commit_time(&commit.time()),
        commit.summary().unwrap_or(""),
    )
}

pub fn print_commit_time(time: &Time) -> String {
    let datetime = FixedOffset::east(time.offset_minutes() * 60).timestamp(time.seconds(), 0);
    format!("{}", datetime)
}
