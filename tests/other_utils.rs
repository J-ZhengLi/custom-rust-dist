use std::path::PathBuf;

use rim::utils;

#[test]
fn walk_dir_recursive() {
    let mut dir_to_walk = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dir_to_walk.push("tests");
    dir_to_walk.push("data");
    dir_to_walk.push("dir_to_walk");

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

#[test]
fn walk_dir_shallow() {
    let mut dir_to_walk = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dir_to_walk.push("tests");
    dir_to_walk.push("data");
    dir_to_walk.push("dir_to_walk");

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

#[test]
fn target_override() {
    let target = env!("TARGET");

    println!("target: {target}");

    if let Ok(override_target) = std::env::var("HOST_TRIPPLE") {
        assert_eq!(override_target, target);
        return;
    }

    #[cfg(all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"))]
    assert_eq!(target, "x86_64-unknown-linux-gnu");
    #[cfg(all(target_arch = "x86_64", target_os = "windows", target_env = "gnu"))]
    assert_eq!(target, "x86_64-pc-windows-gnu");
    #[cfg(all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"))]
    assert_eq!(target, "x86_64-pc-windows-msvc");
}
