use actix_web::{App, HttpServer};
use clap::Clap;
use git_ops::{route, GitDataStore};
use std::sync::Arc;

#[derive(Clap, Debug)]
pub struct Config {
    /// Sets the git repository path to use
    #[clap(short, long)]
    path: String,
}

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    let config = Config::parse();

    let data_store = Arc::new(GitDataStore::new(&config.path));

    HttpServer::new(move || App::new().data(data_store.clone()).service(route::get_data))
        .bind("127.0.0.1:8081")?
        .run()
        .await
}
