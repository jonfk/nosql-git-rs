use actix_web::{dev::HttpResponseBuilder, http::StatusCode, HttpResponse};
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GitDataStoreError {
    #[error("Git2 Error {}", .0)]
    Git2(#[from] git2::Error),

    #[error("Invalid revision could not be found {}", .0)]
    RevNotFound(String),

    #[error("Path could not be found {}", .0)]
    PathNotFound(String),

    #[error("Blob contains non-utf8 content. commit_id: {}, path: {}", .commit_id, .path)]
    NonUtf8Blob { commit_id: String, path: String },

    #[error("Another commit has updated this path since the parent provided. parent_commit_id: {}, path: {}", .parent_commit_id, .path)]
    ConflictOnWrite {
        path: String,
        parent_commit_id: String,
    },
}

#[derive(Serialize)]
pub struct ErrorJson {
    error: String,
}

impl actix_web::error::ResponseError for GitDataStoreError {
    fn error_response(&self) -> HttpResponse {
        let error_str = self.to_string();
        // TODO add error logger
        println!("[ERROR] {}", self);
        HttpResponseBuilder::new(self.status_code()).json(ErrorJson { error: error_str })
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            GitDataStoreError::Git2(..) => StatusCode::INTERNAL_SERVER_ERROR,
            GitDataStoreError::NonUtf8Blob { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            GitDataStoreError::RevNotFound(..) => StatusCode::NOT_FOUND,
            GitDataStoreError::PathNotFound(..) => StatusCode::NOT_FOUND,
            GitDataStoreError::ConflictOnWrite { .. } => StatusCode::CONFLICT,
        }
    }
}
