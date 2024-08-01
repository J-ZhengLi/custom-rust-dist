use std::path::PathBuf;

use custom_rust_dist::utils;

#[test]
fn walk_dir() {
    let mut dir_to_walk = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dir_to_walk.push("tests");
    dir_to_walk.push("data");
    dir_to_walk.push("dir_to_walk");

    let entries = utils::walk_dir(&dir_to_walk).unwrap();

    assert_eq!(
        entries,
        vec![
            dir_to_walk.join("file_in_root"),
            dir_to_walk.join("sub_folder_1"),
            dir_to_walk.join("sub_folder_1").join("file_in_folder_1"),
            dir_to_walk.join("sub_folder_2"),
            dir_to_walk.join("sub_folder_2").join("file_in_folder_2"),
        ]
    );
}
