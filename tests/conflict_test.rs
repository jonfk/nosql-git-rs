use nosql_git::{clone, error::GitDataStoreError, GitDataStore};
use tempfile::TempDir;

mod util;

#[test]
fn conflict_test() {
    let tmp_dir = TempDir::new_in(util::TEST_REPOS_DIR).expect("tmp_dir");
    let tmp_repo_path = tmp_dir.path();

    clone::init(&tmp_repo_path, false).expect("clone::init");
    let store = GitDataStore::new(&tmp_repo_path.to_string_lossy(), "master");

    let doc1_path = "docs/doc1";
    let version_after_doc1 = store
        .put_latest(doc1_path, "test data 1")
        .expect("put_latest doc1");

    let doc1_path = "docs/doc1";
    let _version_after_update1_doc1 = store
        .put(&version_after_doc1, doc1_path, "test data 2", false)
        .expect("put 1 doc1");

    let doc1_path = "docs/doc1";
    let update2_result = store.put(&version_after_doc1, doc1_path, "test data 3", false);

    assert!(update2_result.is_err());
    let conflict_err = update2_result.err().unwrap();
    assert!(
        matches!(conflict_err, GitDataStoreError::ConflictOnWrite{path, ..} if path == doc1_path)
    );
}
