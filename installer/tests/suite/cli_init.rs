use crate::common::{run, APPNAME};

use std::collections::HashMap;

#[test]
fn init_default_no_rustup() {
    // a clean env is needed
    let env = HashMap::from([("PATH", ""), ("CARGO_HOME", ""), ("RUSTUP_HOME", "")]);

    run(|cfg| {
        let rustup_update_root =
            url::Url::from_directory_path(cfg.mocked_server_root.join("rustup")).unwrap();

        cfg.execute_with_env(
            &[
                "-y",
                "init",
                "--rustup-update-root",
                rustup_update_root.as_str(),
            ],
            env,
        );

        let app_dir = cfg.home.path().join(APPNAME);
        let downloads_dir = app_dir.join("downloads");
        let cargo_dir = app_dir.join(".cargo");
        let rustup_dir = app_dir.join(".rustup");

        // check if config file was created
        assert!(cfg.conf_path.is_file());
        let config_file_content = cfg.read_config();
        assert_eq!(
            config_file_content,
            format!(
                "[settings]
install_dir = \"{}\"
cargo_home = \"{}\"
rustup_home = \"{}\"
rustup_update_root = \"{}\"
",
                app_dir.to_str().unwrap(),
                cargo_dir.to_str().unwrap(),
                rustup_dir.to_str().unwrap(),
                rustup_update_root.as_str(),
            )
        );

        // check essential directories are created
        assert!(app_dir.is_dir());
        assert!(downloads_dir.is_dir());
        assert!(cargo_dir.is_dir());
        assert!(cargo_dir.join("bin").is_dir());
        assert!(rustup_dir.is_dir());
    });
}
