use crate::{error::GitDataStoreError, log::HistoryEntry, GitDataStore};
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

#[get("/{file_path:.*}")]
pub async fn get_latest_data(
    store: web::Data<Arc<GitDataStore>>,
    path_params: web::Path<(String,)>,
) -> Result<HttpResponse, GitDataStoreError> {
    let file_path = path_params.into_inner().0;
    let git_data = store.read_latest(&file_path)?;

    Ok(HttpResponse::Ok().json(git_data))
}

#[derive(Serialize, Deserialize)]
pub struct PutDataReq {
    data: String,
    resolve_conflict_my_favor: Option<bool>,
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
) -> Result<HttpResponse, GitDataStoreError> {
    let (commit_id, file_path) = path_params.into_inner();
    let new_commit_id = store.put(
        &commit_id,
        &file_path,
        &data.data,
        data.resolve_conflict_my_favor.unwrap_or(false),
    )?;

    Ok(HttpResponse::Ok().json(PutDataResp {
        commit_id: new_commit_id,
    }))
}

#[derive(Serialize, Deserialize)]
pub struct HistoryReqQuery {
    first: usize,
    after: usize,
}

#[derive(Serialize)]
pub struct HistoryResp {
    entries: Vec<HistoryEntry>,
    has_next: bool,
}

#[get("/commits")]
pub async fn commits(
    store: web::Data<Arc<GitDataStore>>,
    web::Query(history_req): web::Query<HistoryReqQuery>,
) -> Result<HttpResponse, GitDataStoreError> {
    let history = store.history()?;
    let entries: Result<Vec<_>, GitDataStoreError> = history
        .iter()?
        .skip(history_req.after)
        .take(history_req.first + 1)
        .collect();
    let mut entries = entries?;
    let has_next = entries.len() == (history_req.first + 1);
    entries.truncate(history_req.first);

    Ok(HttpResponse::Ok().json(HistoryResp {
        entries: entries,
        has_next: has_next,
    }))
}
