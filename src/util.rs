/// Sanitize paths such that it should prevent
/// [path traversal attacks](https://owasp.org/www-community/attacks/Path_Traversal)
pub fn sanitize_path(path: &str) -> PathBuf {
    PathBuf::from(path.trim_start_matches("/"))
        .into_iter()
        .filter_map(|component| {
            component.to_str().and_then(|comp| {
                if comp == "." || comp == ".." {
                    None
                } else {
                    Some(comp)
                }
            })
        })
        .collect()
}
