use std::sync::Arc;

use actix_web::{get, web, HttpResponse};

use crate::GitDataStore;

#[get("/{commit_id}/{file_path:.*}")]
pub async fn get_data(
    store: web::Data<Arc<GitDataStore>>,
    path_params: web::Path<(String, String)>,
) -> HttpResponse {
    let (commit_id, file_path) = path_params.into_inner();
    let git_data = store.read(&commit_id, &file_path);

    HttpResponse::Ok().json(git_data)
}
