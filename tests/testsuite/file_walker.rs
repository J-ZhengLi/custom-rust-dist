use rim::utils;

use rim_test_support::paths;
use rim_test_support::prelude::*;

#[rim_test]
fn walk_dir_recursive() {
    let dir_to_walk = paths::assets_home().join("dir_to_walk");

    let entries = utils::walk_dir(&dir_to_walk, true).unwrap();

    let expected = vec![
        dir_to_walk.join("file_in_root"),
        dir_to_walk.join("sub_folder_1"),
        dir_to_walk.join("sub_folder_1").join("file_in_folder_1"),
        dir_to_walk.join("sub_folder_2"),
        dir_to_walk.join("sub_folder_2").join("file_in_folder_2"),
    ];
    for exp in expected {
        assert!(entries.contains(&exp));
    }
}

#[rim_test]
fn walk_dir_shallow() {
    let dir_to_walk = paths::assets_home().join("dir_to_walk");

    let entries = utils::walk_dir(&dir_to_walk, false).unwrap();

    let expected = vec![
        dir_to_walk.join("file_in_root"),
        dir_to_walk.join("sub_folder_1"),
        dir_to_walk.join("sub_folder_2"),
    ];
    for exp in expected {
        assert!(entries.contains(&exp));
    }
}
