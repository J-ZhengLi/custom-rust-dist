use std::path::PathBuf;

use custom_rust_dist::utils::{self, Extractable};

#[test]
fn extracting_simple_zip() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("data");
    path.push("simple_zip.zip");

    let cache_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("cache");
    utils::mkdirs(&cache_dir).unwrap();

    let temp_dir = tempfile::Builder::new()
        .prefix("extract_test_")
        .tempdir_in(&cache_dir)
        .unwrap();

    let extractable = Extractable::try_from(path.as_path()).unwrap();
    extractable.extract_to(temp_dir.path()).unwrap();

    assert!(temp_dir.path().join("aaa.txt").is_file());
    assert!(temp_dir.path().join("bbb.txt").is_file());
    assert!(temp_dir.path().join("ccc.txt").is_file());
}

#[test]
fn extracting_normal_zip() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("data");
    path.push("zip_with_sub_folders.zip");

    let cache_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("cache");
    utils::mkdirs(&cache_dir).unwrap();

    let temp_dir = tempfile::Builder::new()
        .prefix("extract_test_")
        .tempdir_in(&cache_dir)
        .unwrap();

    let extractable = Extractable::try_from(path.as_path()).unwrap();
    extractable.extract_to(temp_dir.path()).unwrap();

    assert!(temp_dir.path().join("aaa.txt").is_file());
    assert!(temp_dir.path().join("bbb.txt").is_file());
    assert!(temp_dir.path().join("f1").is_dir());
    assert!(temp_dir.path().join("f1").join("aaa.txt").is_file());
    assert!(temp_dir.path().join("f1").join("bbb.txt").is_file());
    assert!(temp_dir.path().join("f2").is_dir());
    assert!(temp_dir.path().join("f2").join("aaa.txt").is_file());
    assert!(temp_dir.path().join("f3").is_dir());
    assert!(temp_dir.path().join("f3").join("aaa.txt").is_file());
    assert!(temp_dir.path().join("f3").join("bbb.md").is_file());
    assert!(temp_dir.path().join("f3").join("ccc").is_file());
}
