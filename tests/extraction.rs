use rim::utils::{self, Extractable};
use std::path::{Path, PathBuf};
use tempfile::TempDir;

fn extract_to_temp(filename: &str, skip_prefix: bool) -> (PathBuf, TempDir) {
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

    let mut extractable = Extractable::load(path.as_path()).unwrap();

    if skip_prefix {
        let path = extractable
            .extract_then_skip_solo_dir(temp_dir.path(), None::<&str>)
            .expect("failed to extract");
        (path, temp_dir)
    } else {
        extractable
            .extract_to(temp_dir.path())
            .expect("failed to extract");
        (temp_dir.path().to_path_buf(), temp_dir)
    }
}

fn assert_normal_archive(extracted: &Path) {
    assert!(extracted.join("aaa.txt").is_file());
    assert!(extracted.join("bbb.txt").is_file());
    assert!(extracted.join("f1").is_dir());
    assert!(extracted.join("f1").join("aaa.txt").is_file());
    assert!(extracted.join("f1").join("bbb.txt").is_file());
    assert!(extracted.join("f2").is_dir());
    assert!(extracted.join("f2").join("aaa.txt").is_file());
    assert!(extracted.join("f3").is_dir());
    assert!(extracted.join("f3").join("aaa.txt").is_file());
    assert!(extracted.join("f3").join("bbb.md").is_file());
    assert!(extracted.join("f3").join("ccc").is_file());
}

fn assert_extracted_with_prefixes(extracted: &Path) {
    assert!(extracted.join("aaa.txt").is_file());
    assert!(extracted.join("bbb.txt").is_file());
    assert!(extracted.join("f1").is_dir());
    assert!(extracted.join("f1").join("aaa.txt").is_file());
    assert!(extracted.join("f1").join("bbb.txt").is_file());
    assert!(extracted.join("f2").is_dir());
    assert!(extracted.join("f2").join("aaa.txt").is_file());
    assert!(extracted.join("f3").is_dir());
    assert!(extracted.join("f3").join("aaa.txt").is_file());
    assert!(extracted.join("f3").join("bbb.md").is_file());
    assert!(extracted.join("f3").join("ccc").is_file());
}

#[test]
fn extracting_simple_zip() {
    let extracted_dir = extract_to_temp("simple_zip.zip", false);
    let path = extracted_dir.0;

    assert!(path.join("aaa.txt").is_file());
    assert!(path.join("bbb.txt").is_file());
    assert!(path.join("ccc.txt").is_file());
}

#[test]
fn extracting_normal_zip() {
    let temp_dir = extract_to_temp("zip_with_sub_folders.zip", false);
    assert_normal_archive(&temp_dir.0);
}

#[test]
fn extracting_simple_7z() {
    let temp_dir = extract_to_temp("simple_7z.7z", false);

    assert!(temp_dir.0.join("aaa.txt").is_file());
    assert!(temp_dir.0.join("bbb.txt").is_file());
    assert!(temp_dir.0.join("ccc.txt").is_file());
}

#[test]
fn extracting_normal_7z() {
    let temp_dir = extract_to_temp("7z_with_sub_folders.7z", false);
    assert_normal_archive(&temp_dir.0);
}

#[test]
fn extracting_simple_gz() {
    let temp_dir = extract_to_temp("simple_gz.tar.gz", true);

    assert!(temp_dir.0.join("aaa.txt").is_file());
    assert!(temp_dir.0.join("bbb.txt").is_file());
    assert!(temp_dir.0.join("ccc.txt").is_file());
}

#[test]
fn extracting_single_file_gz() {
    let temp_dir = extract_to_temp("single_file.tar.gz", false);

    println!("content: {:#?}", utils::walk_dir(&temp_dir.0, true));
    assert!(temp_dir.0.join("aaa.txt").is_file());
}

#[test]
fn extracting_normal_gz() {
    let temp_dir = extract_to_temp("gz_with_sub_folders.tar.gz", true);
    assert_normal_archive(&temp_dir.0);
}

#[test]
fn extracting_simple_xz() {
    let temp_dir = extract_to_temp("simple_xz.tar.xz", true);

    assert!(temp_dir.0.join("aaa.txt").is_file());
    assert!(temp_dir.0.join("bbb.txt").is_file());
    assert!(temp_dir.0.join("ccc.txt").is_file());
}

#[test]
fn extracting_normal_xz() {
    let temp_dir = extract_to_temp("xz_with_sub_folders.tar.xz", true);
    assert_normal_archive(&temp_dir.0);
}

#[test]
fn extracting_xz_with_prefix() {
    let temp_dir = extract_to_temp("xz_with_prefixes.tar.xz", true);
    assert_extracted_with_prefixes(&temp_dir.0);
}

#[test]
fn extracting_7z_with_prefix() {
    let temp_dir = extract_to_temp("7z_with_prefixes.7z", true);

    println!("content: {:#?}", utils::walk_dir(&temp_dir.0, true));
    assert_extracted_with_prefixes(&temp_dir.0);
}

#[test]
fn extracting_zip_with_prefix() {
    let temp_dir = extract_to_temp("zip_with_prefixes.zip", true);
    assert_extracted_with_prefixes(&temp_dir.0);
}
