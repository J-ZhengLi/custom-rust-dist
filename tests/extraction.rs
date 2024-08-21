use custom_rust_dist::utils::{self, Extractable};
use std::path::PathBuf;
use tempfile::TempDir;

fn extract_to_temp(filename: &str) -> TempDir {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("data");
    path.push(filename);

    let cache_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("cache");
    utils::ensure_dir(&cache_dir).unwrap();

    let temp_dir = tempfile::Builder::new()
        .prefix("extract_test_")
        .tempdir_in(&cache_dir)
        .unwrap();

    let extractable = Extractable::try_from(path.as_path()).unwrap();
    extractable
        .extract_to(temp_dir.path())
        .expect("failed to extract");

    temp_dir
}

#[test]
fn extracting_simple_zip() {
    let extracted_dir = extract_to_temp("simple_zip.zip");
    let path = extracted_dir.path();

    assert!(path.join("aaa.txt").is_file());
    assert!(path.join("bbb.txt").is_file());
    assert!(path.join("ccc.txt").is_file());
}

#[test]
fn extracting_normal_zip() {
    let temp_dir = extract_to_temp("zip_with_sub_folders.zip");

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

#[test]
fn extracting_simple_7z() {
    let temp_dir = extract_to_temp("simple_7z.7z");

    assert!(temp_dir.path().join("aaa.txt").is_file());
    assert!(temp_dir.path().join("bbb.txt").is_file());
    assert!(temp_dir.path().join("ccc.txt").is_file());
}

#[test]
fn extracting_normal_7z() {
    let temp_dir = extract_to_temp("7z_with_sub_folders.7z");

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

#[test]
fn extracting_simple_gz() {
    let temp_dir = extract_to_temp("simple_gz.tar.gz");

    assert!(temp_dir.path().join("aaa.txt").is_file());
    assert!(temp_dir.path().join("bbb.txt").is_file());
    assert!(temp_dir.path().join("ccc.txt").is_file());
}

#[test]
fn extracting_single_file_gz() {
    let temp_dir = extract_to_temp("single_file.tar.gz");

    assert!(temp_dir.path().join("aaa.txt").is_file());
}

#[test]
fn extracting_normal_gz() {
    let temp_dir = extract_to_temp("gz_with_sub_folders.tar.gz");

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

#[test]
fn extracting_simple_xz() {
    let temp_dir = extract_to_temp("simple_xz.tar.xz");

    assert!(temp_dir.path().join("aaa.txt").is_file());
    assert!(temp_dir.path().join("bbb.txt").is_file());
    assert!(temp_dir.path().join("ccc.txt").is_file());
}

#[test]
fn extracting_normal_xz() {
    let temp_dir = extract_to_temp("xz_with_sub_folders.tar.xz");

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

#[test]
fn extracting_xz_with_prefix() {
    let temp_dir = extract_to_temp("xz_with_prefixes.tar.xz");

    let dir_contents = utils::walk_dir(temp_dir.path(), true).unwrap();
    println!("contents: {:#?}", dir_contents);

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

#[test]
fn extracting_7z_with_prefix() {
    let temp_dir = extract_to_temp("7z_with_prefixes.7z");

    let dir_contents = utils::walk_dir(temp_dir.path(), true).unwrap();
    println!("contents: {:#?}", dir_contents);

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
