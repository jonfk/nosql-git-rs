use anyhow::Result;
use git2::{Cred, RemoteCallbacks, Repository};
use std::env;
use std::path::Path;

pub fn clone_ssh<T: AsRef<Path>>(url: &str, path: T) -> Result<Repository> {
    // Prepare callbacks.
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        Cred::ssh_key(
            username_from_url.unwrap(),
            None,
            std::path::Path::new(&format!("{}/.ssh/id_rsa", env::var("HOME").unwrap())),
            None,
        )
    });

    // Prepare fetch options.
    let mut fo = git2::FetchOptions::new();
    fo.remote_callbacks(callbacks);

    // Prepare builder.
    let mut builder = git2::build::RepoBuilder::new();
    builder.fetch_options(fo);

    // Clone the project.
    Ok(builder.clone(url, path.as_ref())?)
}
