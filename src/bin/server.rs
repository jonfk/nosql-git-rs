#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;

use actix_slog::StructuredLogger;
use actix_web::{App, HttpServer};
use clap::Clap;
use nosql_git::{clone, route, GitDataStore};
use slog::Drain;
use std::{path::Path, sync::Arc};

#[derive(Clap, Debug)]
pub struct Config {
    /// Sets the git repository path to use
    #[clap(short, long)]
    path: String,

    /// Sets the primary branch
    #[clap(short, long, default_value = "master")]
    branch: String,

    /// Clone from url if `path` does not already exists.
    /// Errors if path already exists.
    /// Currently only supports ssh
    #[clap(short, long)]
    clone: Option<String>,

    /// Initializes a repository at path if path does not already exist.
    /// Clone will take precedence over init and the init will fail.
    #[clap(short, long)]
    init: bool,

    /// Repository is created as bare when cloned or initialized.
    #[clap(long)]
    bare: bool,
}

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    let root_log = slog::Logger::root(drain, o!());
    let _log_guard = slog_stdlog::init().unwrap();

    let config = Config::parse();

    if let Some(clone_url) = config.clone {
        if Path::new(&config.path).exists() {
            error!(root_log, "Path already exists. Cannot clone"; "url" => &clone_url, "path" => &config.path);
            std::process::exit(1);
        }
        info!(root_log, "cloning from url"; "url" => &clone_url, "path" => &config.path);
        clone::clone_ssh(&clone_url, &config.path, config.bare).expect("clone");
    }

    if config.init {
        info!(root_log, "initializing repository"; "path" => &config.path);
        clone::init(&config.path, config.bare).expect("init");
    }

    let data_store = Arc::new(GitDataStore::new(&config.path, &config.branch));

    info!(root_log, "listening to :8081");
    HttpServer::new(move || {
        App::new()
            .wrap(StructuredLogger::new(
                root_log.new(o!("log_type" => "access")),
            ))
            .data(data_store.clone())
            .service(route::get_data)
            .service(route::put_data)
            .service(route::history)
            .service(route::get_latest_data)
            .service(route::put_latest_data)
    })
    .bind("127.0.0.1:8081")?
    .run()
    .await
}
