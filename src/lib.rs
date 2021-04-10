use git2::{Oid, Repository};
use serde::Serialize;
use std::path::Path;

pub mod clone;
pub mod commit;
pub mod commit_to_branch;
pub mod log;
pub mod route;

pub struct GitDataStore {
    repo_path: String,
}

#[derive(Serialize)]
pub enum GitData {
    Dir { entries: Vec<String> },
    File { data: String },
}

impl GitDataStore {
    pub fn new(repo_path: &str) -> Self {
        GitDataStore {
            repo_path: repo_path.to_string(),
        }
    }

    pub fn read(&self, commit_id: &str, path: &str) -> GitData {
        let repo = Repository::open(&self.repo_path).expect("open repo");

        let commit = repo
            .find_commit(Oid::from_str(commit_id).expect("Oid::from_str"))
            .expect("find_commit");

        let tree = commit.tree().expect("commit.tree()");

        let entry = tree.get_path(Path::new(path)).expect("tree.get_path");

        match entry.kind().expect("tree entry does not have kind") {
            git2::ObjectType::Tree => {
                let obj = entry.to_object(&repo).expect("entry.to_obj()");
                let tree = obj.as_tree().expect("tree is not a tree");
                GitData::Dir {
                    entries: tree
                        .into_iter()
                        .filter_map(|e| e.name().map(|name| name.to_string()))
                        .collect(),
                }
            }
            git2::ObjectType::Blob => {
                let obj = entry.to_object(&repo).expect("entry.to_obj()");
                let blob = obj.as_blob().expect("blob is not blob");

                GitData::File {
                    data: String::from_utf8(blob.content().to_owned())
                        .expect("blob.content() not utf8"),
                }
            }
            _ => {
                unreachable!("Impossible entry.kind() {:?}", entry.kind())
            }
        }
    }

    pub fn put(&self, parent_commit_id: &str, path: &str, data: &str) {}
}
