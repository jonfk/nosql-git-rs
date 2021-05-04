use git_ops::GitDataStore;

pub fn main() {
    let store = GitDataStore::new("repo", "master");
    let history_iter = store.history().expect("history()");

    let file_history_iter = history_iter.iter_path("library/alloc").expect("iter_path");

    for entry in file_history_iter {
        let entry = entry.expect("entry");
        println!("{} {:?}", entry.commit_id, entry.message);
    }
}
