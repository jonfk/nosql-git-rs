use anyhow::Result;
use git2::{Commit, Repository, Time};

pub fn git_log(repo: Repository) -> Result<()> {
    let mut rev_walk = repo.revwalk()?;
    rev_walk.push_head()?;

    for rev in rev_walk {
        let rev = rev?;
        let commit = repo.find_commit(rev)?;
        println!("{}", print_commit(commit)?);
    }
    Ok(())
}

pub fn print_commit(commit: Commit) -> Result<String> {
    Ok(format!(
        "commit {}\nAuthor: {}\nDate: {}s {}\n\n{}\n",
        commit.id().to_string(),
        commit.author().to_string(),
        commit.time().seconds(),
        commit.time().offset_minutes(),
        commit.summary().unwrap_or(""),
    ))
}

pub fn print_commit_time(time: Time) -> String {
    format!("")
}
