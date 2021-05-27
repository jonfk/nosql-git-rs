use git_ops::{clone, GitDataStore};
use tempfile::TempDir;

mod util;

#[test]
fn delete_test() {
    let tmp_dir = TempDir::new_in(util::TEST_REPOS_DIR).expect("tmp_dir");
    let tmp_repo_path = tmp_dir.path();

    clone::init(&tmp_repo_path, false).expect("clone::init");
    let store = GitDataStore::new(&tmp_repo_path.to_string_lossy(), "master");

    let doc_path = "cods/docs/doc1.txt";
    let doc_version = store
        .put_latest(doc_path, "testdata\nlorem ipsum\n")
        .expect("put_latest");

    let read_doc = store.read_latest(doc_path).expect("read_latest");

    assert!(read_doc.is_some());

    let deleted_version = store.delete(&doc_version, doc_path, false).expect("delete");

    let read_latest_deleted_doc = store.read_latest(doc_path).expect("read_latest");

    let read_deleted_doc = store.read(&deleted_version, doc_path).expect("read_latest");

    assert!(read_deleted_doc.is_none());
    assert!(read_latest_deleted_doc.is_none());
}
