use actix_web::{middleware::Logger, App, HttpServer};
use clap::Clap;
use git_ops::{route, GitDataStore};
use std::sync::Arc;

#[derive(Clap, Debug)]
pub struct Config {
    /// Sets the git repository path to use
    #[clap(short, long)]
    path: String,

    /// Sets the primary branch
    #[clap(short, long, default_value = "master")]
    branch: String,
}

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    let config = Config::parse();

    let data_store = Arc::new(GitDataStore::new(&config.path, &config.branch));

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .data(data_store.clone())
            .service(route::get_data)
            .service(route::put_data)
    })
    .bind("127.0.0.1:8081")?
    .run()
    .await
}
