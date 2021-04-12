use thiserror::Error;

#[derive(Error, Debug)]
pub enum GitDataStoreError {
    #[error("Git2 Error")]
    Git2(#[from] git2::Error),

    #[error("Invalid revision could not be found {}", .0)]
    RevNotFound(String),

    #[error("Path could not be found {}", .0)]
    PathNotFound(String, git2::Error),

    #[error("Blob contains non-utf8 content. commit_id: {}, path: {}", .commit_id, .path)]
    NonUtf8Blob { commit_id: String, path: String },
}
