use anyhow::Result;
use git2::{FileMode, IndexEntry, IndexTime, Oid, Repository, Signature};
use std::{fs, path::PathBuf};

pub struct ToCommit {
    pub message: String,
    pub path: String,
}

pub fn commit(repo: &Repository, to_commit: &ToCommit) -> Result<()> {
    let current_head = repo
        .head()?
        .target()
        .ok_or(anyhow::format_err!("HEAD does not point to anything"))?;

    let head_commit = repo.find_commit(current_head)?;

    let data = fs::read_to_string(&to_commit.path)?;

    let mut index = repo.index()?;

    index.add_frombuffer(&make_index_entry(&to_commit.path), data.as_bytes())?;

    index.write()?;

    let tree_oid = index.write_tree()?;
    let tree = repo.find_tree(tree_oid)?;

    let author_commiter = author_committer()?;

    repo.commit(
        Some("HEAD"),
        &author_commiter,
        &author_commiter,
        &to_commit.message,
        &tree,
        &[&head_commit],
    )?;

    Ok(())
}

pub fn get_filename(path: &str) -> Result<String> {
    Ok(PathBuf::from(path)
        .file_name()
        .and_then(|s| s.to_str().to_owned())
        .ok_or(anyhow::format_err!(
            "filename of {} could not be found",
            path
        ))?
        .to_owned())
}

pub fn author_committer() -> Result<Signature<'static>> {
    Ok(Signature::now("Jonathan Fok kan", "jfokkan@gmail.com")?)
}

pub fn make_index_entry(path: &str) -> IndexEntry {
    IndexEntry {
        ctime: IndexTime::new(0, 0),
        mtime: IndexTime::new(0, 0),
        dev: 0,
        ino: 0,
        mode: FileMode::Blob.into(),
        uid: 0,
        gid: 0,
        file_size: 0,
        id: Oid::from_bytes(&[0; 20]).unwrap(),
        flags: 0,
        flags_extended: 0,
        path: path.into(),
    }
}
