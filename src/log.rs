use crate::error::GitDataStoreError;
use chrono::{FixedOffset, TimeZone};
use git2::{Commit, Oid, Repository, Revwalk, Time};
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct HistoryEntry {
    timestamp_seconds: i64,
    commit_id: String,
    message: Option<String>,
    author: String,
    stats: HistoryStats,
}

#[derive(Serialize, Debug)]
pub struct HistoryStats {
    files_changed: usize,
    insertions: usize,
    deletions: usize,
}

pub struct HistoryIterator {
    repo: Repository,
}

impl HistoryIterator {
    fn new(repo: Repository) -> Self {
        HistoryIterator { repo: repo }
    }

    pub fn iter<'repo>(
        &'repo self,
    ) -> Result<
        impl Iterator<Item = Result<HistoryEntry, GitDataStoreError>> + 'repo,
        GitDataStoreError,
    > {
        let mut rev_walk = self.repo.revwalk()?;
        rev_walk.push_head();
        Ok(rev_walk.map(move |rev| map_rev(&self.repo, rev)))
    }
}

pub fn git_log<'repo>(repo: Repository) -> Result<HistoryIterator, GitDataStoreError> {
    Ok(HistoryIterator::new(repo))
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
        message: commit.message().map(|m| m.to_string()),
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
