use std::{collections::HashMap, path::Path, time::Instant};

use crate::error::GitDataStoreError;
use chrono::{FixedOffset, TimeZone};
use git2::{Commit, DiffOptions, Oid, Repository, Revwalk, Time};
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct HistoryEntry {
    pub timestamp_seconds: i64,
    pub commit_id: String,
    pub message: Option<String>,
    pub author: String,
    pub stats: HistoryStats,
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
        rev_walk.push_head()?;
        Ok(rev_walk.map(move |rev| map_rev(&self.repo, rev)))
    }

    pub fn iter_path<'repo>(
        &'repo self,
        path: &str,
    ) -> Result<FileHistoryIterator, GitDataStoreError> {
        let mut rev_walk = self.repo.revwalk()?;
        rev_walk.push_head()?;

        Ok(FileHistoryIterator {
            repo: &self.repo,
            rev_walker: rev_walk,
            commits_2_path: HashMap::new(),
            path: path.to_string(),
        })
    }
}

/// Algorithm taken from https://github.com/libgit2/libgit2sharp/blob/f916e79575bea0a99d3c67249090f51ff62d4e23/LibGit2Sharp/Core/FileHistory.cs
pub struct FileHistoryIterator<'repo> {
    repo: &'repo Repository,
    rev_walker: Revwalk<'repo>,
    commits_2_path: HashMap<Oid, String>,
    path: String,
}

impl<'repo> Iterator for FileHistoryIterator<'repo> {
    type Item = Result<HistoryEntry, GitDataStoreError>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(rev) = self.rev_walker.next() {
                match self.map_file_history_rev(rev) {
                    Ok(history_entry_opt) => {
                        if let Some(history_entry) = history_entry_opt {
                            return Some(Ok(history_entry));
                        }
                        // else go to next iteration
                    }
                    Err(err) => return Some(Err(err)),
                }
            } else {
                return None;
            }
        }
    }
}

impl<'repo> FileHistoryIterator<'repo> {
    fn map_file_history_rev(
        &mut self,
        rev: Result<Oid, git2::Error>,
    ) -> Result<Option<HistoryEntry>, GitDataStoreError> {
        // let now = Instant::now();
        let rev = rev?;
        //println!("rev {}", now.elapsed().as_nanos());
        let current_commit = self.repo.find_commit(rev.clone())?;
        //println!("find_commit {}", now.elapsed().as_nanos());

        let current_path = self
            .commits_2_path
            .get(&current_commit.id())
            .map(|p| p.clone())
            .unwrap_or(self.path.clone());
        //println!("current_path {}", now.elapsed().as_nanos());
        let current_tree = current_commit.tree()?;
        //println!("current_tree {}", now.elapsed().as_nanos());
        let current_tree_entry = get_path_from_tree(&current_tree, Path::new(&current_path))?;
        //println!("current_tree_entry {}", now.elapsed().as_nanos());

        if current_tree_entry.is_none() {
            // println!(
            //     "[SKIP] current_tree_entry is_none {}",
            //     now.elapsed().as_nanos()
            // );
            return Ok(None);
        }

        if current_commit.parents().count() == 0 {
            // println!(
            //     "[TAKE] current_commit no parents {}",
            //     now.elapsed().as_nanos()
            // );
            return Ok(Some(map_rev(self.repo, Ok(rev.clone()))?));
        } else {
            determine_parent_path(
                &self.repo,
                &mut self.commits_2_path,
                &current_commit,
                &current_path,
            )?;
            //println!("determine_parent_path {}", now.elapsed().as_nanos());
            // the original solution skipped the commit if it had more than 1 parent. Not sure why
            if current_commit.parents().count() > 1 {
                // println!(
                //     "[SKIP] current_commit parents not 1 {}",
                //     now.elapsed().as_nanos()
                // );
                return Ok(None);
            }

            let parent_commit = current_commit.parents().next().unwrap(); // todo remove unwrap
                                                                          //println!("parent_commit {}", now.elapsed().as_nanos());
            let parent_path = self
                .commits_2_path
                .get(&parent_commit.id())
                .unwrap_or(&self.path);
            //println!("parent_path {}", now.elapsed().as_nanos());
            let parent_tree = parent_commit.tree()?;
            //println!("parent_tree {}", now.elapsed().as_nanos());
            let parent_tree_entry = get_path_from_tree(&parent_tree, Path::new(&parent_path))?;
            //println!("parent_tree_entry {}", now.elapsed().as_nanos());

            if parent_tree_entry.is_none()
                || parent_tree_entry.unwrap().id() != current_tree_entry.unwrap().id()
                || *parent_path != current_path
            {
                //println!("[TAKE] {}", now.elapsed().as_nanos());
                return Ok(Some(map_rev(self.repo, Ok(rev.clone()))?));
            }
        }
        //println!("[SKIP] {}", now.elapsed().as_nanos());
        return Ok(None);
    }
}

fn get_path_from_tree<'tree>(
    tree: &'tree git2::Tree,
    path: &Path,
) -> Result<Option<git2::TreeEntry<'tree>>, GitDataStoreError> {
    match tree.get_path(path) {
        Ok(tree_entry) => Ok(Some(tree_entry)),
        Err(err) => {
            if err.code() == git2::ErrorCode::NotFound && err.class() == git2::ErrorClass::Tree {
                Ok(None)
            } else {
                Err(GitDataStoreError::Git2(err))
            }
        }
    }
}

fn determine_parent_path(
    repo: &Repository,
    commits_2_path: &mut HashMap<Oid, String>,
    current_commit: &Commit,
    current_path: &str,
) -> Result<(), GitDataStoreError> {
    //let now = Instant::now();
    let parent_commits: Vec<_> = current_commit
        .parents()
        .filter(|commit| commits_2_path.get(&commit.id()).is_none())
        .collect();
    //println!("get parent_commits {}", now.elapsed().as_nanos());
    for parent_commit in parent_commits {
        if let std::collections::hash_map::Entry::Vacant(entry) =
            commits_2_path.entry(parent_commit.id())
        {
            //let now = Instant::now();
            entry.insert(parent_path(
                &repo,
                &current_commit,
                &parent_commit,
                &current_path,
            )?);
            //println!("entry.insert {}", now.elapsed().as_nanos());
        }
    }
    Ok(())
}

fn parent_path(
    repo: &Repository,
    current_commit: &Commit,
    parent_commit: &Commit,
    current_path: &str,
) -> Result<String, GitDataStoreError> {
    let mut diff_options = DiffOptions::new();
    diff_options.pathspec(current_path);
    let diff = repo.diff_tree_to_tree(
        Some(&parent_commit.tree()?),
        Some(&current_commit.tree()?),
        Some(&mut diff_options),
    )?;

    let new_path = if let Some(file_rename_change) = diff
        .deltas()
        .filter(|delta| {
            delta
                .new_file()
                .path()
                .map(|new_path| new_path == Path::new(current_path))
                .unwrap_or(false)
        })
        .next()
    {
        if file_rename_change.status() == git2::Delta::Renamed {
            file_rename_change
                .old_file()
                .path()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or(current_path.to_string())
        } else {
            current_path.to_string()
        }
    } else {
        current_path.to_string()
    };
    Ok(new_path)
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
