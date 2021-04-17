#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;

use actix_slog::StructuredLogger;
use actix_web::{middleware::Logger, App, HttpServer};
use clap::Clap;
use git_ops::{route, GitDataStore};
use slog::Drain;
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
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    let root_log = slog::Logger::root(drain, o!());
    let _log_guard = slog_stdlog::init().unwrap();

    let config = Config::parse();

    let data_store = Arc::new(GitDataStore::new(&config.path, &config.branch));

    HttpServer::new(move || {
        App::new()
            .wrap(StructuredLogger::new(
                root_log.new(o!("log_type" => "access")),
            ))
            .data(data_store.clone())
            .service(route::get_data)
            .service(route::put_data)
            .service(route::get_latest_data)
    })
    .bind("127.0.0.1:8081")?
    .run()
    .await
}
