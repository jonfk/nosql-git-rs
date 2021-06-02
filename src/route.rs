use crate::{error::GitDataStoreError, history::HistoryEntry, GitDataStore};
use actix_web::{body::Body, get, post,  web, HttpResponse, delete};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[get("/commits/{commit_id}/{file_path:.*}")]
pub async fn get_data(
    store: web::Data<Arc<GitDataStore>>,
    path_params: web::Path<(String, String)>,
) -> Result<HttpResponse, GitDataStoreError> {
    let (commit_id, file_path) = path_params.into_inner();

    Ok(match store.read(&commit_id, &file_path)? {
        Some(git_data) => HttpResponse::Ok().json(git_data),
        None => HttpResponse::NotFound().body(Body::None),
    })
}

#[get("/latest/{file_path:.*}")]
pub async fn get_latest_data(
    store: web::Data<Arc<GitDataStore>>,
    path_params: web::Path<(String,)>,
) -> Result<HttpResponse, GitDataStoreError> {
    let file_path = path_params.into_inner().0;
    Ok(match store.read_latest(&file_path)? {
        Some(git_data) => HttpResponse::Ok().json(git_data),
        None => HttpResponse::NotFound().body(Body::None),
    })
}

#[derive(Serialize, Deserialize)]
pub struct PutDataReq {
    data: String,
    overwrite: Option<bool>,
    commit_msg: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct PutDataResp {
    commit_id: String,
}

#[post("/commits/{commit_id}/{file_path:.*}")]
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
        data.overwrite.unwrap_or(false),
        None,
        data.commit_msg.as_deref(),
    )?;

    Ok(HttpResponse::Ok().json(PutDataResp {
        commit_id: new_commit_id,
    }))
}

#[post("/latest/{file_path:.*}")]
pub async fn put_latest_data(
    store: web::Data<Arc<GitDataStore>>,
    path_params: web::Path<(String,)>,
    data: web::Json<PutDataReq>,
) -> Result<HttpResponse, GitDataStoreError> {
    let file_path = path_params.into_inner().0;
    let new_commit_id =
        store.put_latest(&file_path, &data.data, None, data.commit_msg.as_deref())?;

    Ok(HttpResponse::Ok().json(PutDataResp {
        commit_id: new_commit_id,
    }))
}

#[derive(Serialize, Deserialize)]
pub struct HistoryReqQuery {
    first: usize,
    after: usize,
    path: Option<String>,
}

#[derive(Serialize)]
pub struct HistoryResp {
    entries: Vec<HistoryEntry>,
}

#[get("/history")]
pub async fn history(
    store: web::Data<Arc<GitDataStore>>,
    web::Query(history_req): web::Query<HistoryReqQuery>,
) -> Result<HttpResponse, GitDataStoreError> {
    let history = store.history()?;
    let entries: Result<Vec<_>, GitDataStoreError> = if let Some(path) = history_req.path {
        history
            .iter_path(&path)?
            .skip(history_req.after)
            .take(history_req.first + 1)
            .collect()
    } else {
        history
            .iter()?
            .skip(history_req.after)
            .take(history_req.first + 1)
            .collect()
    };
    let entries = entries?;

    Ok(HttpResponse::Ok().json(HistoryResp {
        entries: entries,
    }))
}

#[derive(Serialize, Deserialize)]
pub struct DeleteReq {
    overwrite: Option<bool>,
    commit_msg: Option<String>,
}

#[delete("/commits/{commit_id}/{file_path:.*}")]
pub async fn delete(
    store: web::Data<Arc<GitDataStore>>,
    path_params: web::Path<(String, String)>,
    data: web::Json<PutDataReq>,
) -> Result<HttpResponse, GitDataStoreError> {
    let (commit_id, file_path) = path_params.into_inner();
    let new_commit_id = store.delete(
        &commit_id,
        &file_path,
        data.overwrite.unwrap_or(false),
        None,
        data.commit_msg.as_deref(),
    )?;

    Ok(HttpResponse::Ok().json(PutDataResp {
        commit_id: new_commit_id,
    }))
}

#[delete("/latest/{file_path:.*}")]
pub async fn delete_latest(
    store: web::Data<Arc<GitDataStore>>,
    path_params: web::Path<(String,)>,
    data: web::Json<DeleteReq>,
) -> Result<HttpResponse, GitDataStoreError> {
    let file_path = path_params.into_inner().0;
    let new_commit_id = store.delete_latest(
        &file_path,
        None,
        data.commit_msg.as_deref(),
    )?;

    Ok(HttpResponse::Ok().json(PutDataResp {
        commit_id: new_commit_id,
    }))
}
