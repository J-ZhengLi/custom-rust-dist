use crate::common::{run_with_profile, utils, Profile};

#[test]
fn new_config_with_cargo_home_set() {
    run_with_profile(Profile::InitDefault, |cfg| {
        let cargo_home = cfg.home.path().join(".cargo");
        let cargo_home_str = utils::path_to_str(&cargo_home);

        cfg.execute(&["-y", "config", "--cargo-home", cargo_home_str]);

        let cfg_content = cfg.read_config();
        assert!(cfg_content.contains(&format!("cargo_home = \"{cargo_home_str}\"")));
    })
}

#[test]
fn import_full_config() {
    run_with_profile(Profile::InitDefault, |cfg| {
        let conf_path = cfg.data_dir.join("settings_with_no_previous_env.toml");
        let conf_path_str = utils::path_to_str(&conf_path);

        cfg.execute(&["-y", "config", "--input", conf_path_str]);

        // eliminate line ending difference between different OS by removing whitespaces
        let imported_content: String = cfg.read_config().split_whitespace().collect();
        let expected: String = cfg
            .read_data("settings_with_no_previous_env.toml")
            .split_whitespace()
            .collect();
        assert_eq!(imported_content, expected);
    })
}

#[test]
fn config_with_cli_args() {
    run_with_profile(Profile::InitDefault, |cfg| {
        let cargo_home = cfg.home.path().join(".cargo");
        let rustup_home = cfg.home.path().join(".rustup");
        let args = &[
            "-y",
            "config",
            "--cargo-home",
            utils::path_to_str(&cargo_home),
            "--rustup-home",
            utils::path_to_str(&rustup_home),
            "--rustup-dist-server",
            "http://example.com/",
            "--rustup-update-root",
            "http://example.com/rustup/",
            "--proxy",
            "https://my:1234@my.proxy.com:1234",
            "--no-proxy",
            "localhost,*.example.com",
            "--git-fetch-with-cli=true",
            "--check-revoke=false",
        ];

        cfg.execute(args);
        // add mirror registry
        cfg.execute(&[
            "-y",
            "config",
            "registry",
            "add",
            "http://mirror.example.com/",
            "--name",
            "mirror",
        ]);
        // set 'mirror' as default registry
        cfg.execute(&["config", "registry", "default", "mirror"]);

        let cfg_content = cfg.read_config();

        assert!(cfg_content.contains(&format!(
            r#"
cargo_home = "{}"
rustup_home = "{}"
rustup_dist_server = "http://example.com/"
rustup_update_root = "http://example.com/rustup/"
proxy = "https://my:1234@my.proxy.com:1234"
no_proxy = "localhost,*.example.com""#,
            utils::path_to_str(&cargo_home),
            utils::path_to_str(&rustup_home),
        )));

        // TODO: read and verify cargo config file
    })
}

#[test]
fn add_and_rm_registry() {
    run_with_profile(Profile::InitDefault, |cfg| {
        cfg.execute(&[
            "-y",
            "config",
            "registry",
            "add",
            "http://mirror.a.com/",
            "--name",
            "a",
        ]);
        cfg.execute(&[
            "-y",
            "config",
            "registry",
            "add",
            "http://mirror.b.com/",
            "--name",
            "b",
        ]);
        cfg.execute(&["config", "registry", "add", "http://mirror.c.com/"]);

        // TODO: read and verify CARGO_HOME/config file content

        cfg.execute(&["-y", "config", "registry", "rm", "b"]);
        cfg.execute(&["-y", "config", "registry", "rm", "\"mirror.c.com\""]);

        // TODO: read and verify CARGO_HOME/config file content
    })
}
