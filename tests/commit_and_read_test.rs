use git_ops::{clone, GitDataStore};
use tempfile::TempDir;

mod util;

#[test]
fn commit_and_read_test() {
    let tmp_dir = TempDir::new_in(util::TEST_REPOS_DIR).expect("tmp_dir");
    let tmp_repo_path = tmp_dir.path();

    clone::init(&tmp_repo_path, false).expect("clone::init");

    let store = GitDataStore::new(&tmp_repo_path.to_string_lossy(), "master");

    let doc1_path = "docs/doc1";
    let version_after_doc1 = store
        .put_latest(doc1_path, "test data 1")
        .expect("put_latest doc1");

    let doc2_path = "docs/doc2";
    let doc2_data = "completely different data\nhello\nblah\n";
    let version_after_doc2 = store
        .put(&version_after_doc1, doc2_path, doc2_data, false)
        .expect("put doc2");
    println!("doc2 saved");

    let doc1_data_update = "new doc1 data updated\nnothing related to before";

    let version_after_doc1_update = store
        .put(&version_after_doc1, doc1_path, doc1_data_update, false)
        .expect("put doc1 update");

    let doc1_latest_result = store
        .read_latest(doc1_path)
        .expect("read_latest doc1")
        .unwrap();
    let doc1_updated_result = store
        .read(&version_after_doc1_update, doc1_path)
        .expect("read doc1")
        .unwrap();

    assert_eq!(doc1_latest_result, doc1_updated_result);
    assert_eq!(doc1_latest_result.data, doc1_updated_result.data);
    assert!(doc1_latest_result.data.is_file());
    assert!(doc1_updated_result.data.is_file());

    let doc2_latest_result = store
        .read_latest(doc2_path)
        .expect("read_latest doc2")
        .unwrap();
    let doc2_created_result = store
        .read(&version_after_doc2, doc2_path)
        .expect("read doc2")
        .unwrap();

    assert_eq!(doc2_latest_result.data, doc2_created_result.data);
    assert_eq!(doc2_latest_result.data.file().unwrap(), doc2_data);
    assert_eq!(doc2_created_result.data.file().unwrap(), doc2_data);

    //Ok(())
}
