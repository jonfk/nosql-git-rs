use crate::{error::GitDataStoreError, GitDataStore};
use actix_web::{get, put, web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[get("/{commit_id}/{file_path:.*}")]
pub async fn get_data(
    store: web::Data<Arc<GitDataStore>>,
    path_params: web::Path<(String, String)>,
) -> Result<HttpResponse, GitDataStoreError> {
    let (commit_id, file_path) = path_params.into_inner();
    let git_data = store.read(&commit_id, &file_path)?;

    Ok(HttpResponse::Ok().json(git_data))
}

#[derive(Serialize, Deserialize)]
pub struct PutDataReq {
    data: String,
}

#[derive(Serialize, Deserialize)]
pub struct PutDataResp {
    commit_id: String,
}

#[put("/{commit_id}/{file_path:.*}")]
pub async fn put_data(
    store: web::Data<Arc<GitDataStore>>,
    path_params: web::Path<(String, String)>,
    data: web::Json<PutDataReq>,
) -> HttpResponse {
    let (commit_id, file_path) = path_params.into_inner();
    let new_commit_id = store.put(&commit_id, &file_path, &data.data);

    HttpResponse::Ok().json(PutDataResp {
        commit_id: new_commit_id,
    })
}
