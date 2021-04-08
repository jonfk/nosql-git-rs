use crate::commit::{author_committer, make_index_entry};
use anyhow::Result;
use git2::{Oid, Reference, Repository};

pub struct CommitToBranch {
    pub path: String,
    pub message: String,
    pub data: String,
    pub branch_name: String,
}

pub fn commit_to_branch(req: &CommitToBranch, repo: &Repository) -> Result<()> {
    let primary_branch = "master";
    let head_commit = repo.revparse_single(primary_branch)?;

    let branch_ref = create_branch(&repo, &req.branch_name, head_commit.id())?;

    let mut index = repo.index()?;

    index.add_frombuffer(&make_index_entry(&req.path), req.data.as_bytes())?;

    // no need to write the index since we actually only want to write to a branch
    //index.write()?;

    let tree_oid = index.write_tree()?;
    let tree = repo.find_tree(tree_oid)?;

    let author_commiter = author_committer()?;

    repo.commit(
        Some(
            branch_ref
                .name()
                .ok_or(anyhow::format_err!("branch name not utf-8"))?,
        ),
        &author_commiter,
        &author_commiter,
        &req.message,
        &tree,
        &[&head_commit
            .into_commit()
            .expect("head of primary branch not a commit")],
    )?;

    Ok(())
}

pub fn create_branch<'repo>(
    repo: &'repo Repository,
    branch_name: &str,
    oid: Oid,
) -> Result<Reference<'repo>> {
    Ok(repo.reference(
        &format!("refs/heads/{}", branch_name),
        oid,
        false,
        "creating branch",
    )?)
}
