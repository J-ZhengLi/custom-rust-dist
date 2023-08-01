use crate::common::{run, utils};

#[test]
fn new_default_config_should_not_create() {
    run(|cfg| {
        cfg.execute(&["config", "--default"]);

        assert!(!cfg.conf_path.is_file());
    })
}

#[test]
fn new_config_with_cargo_home_set() {
    run(|cfg| {
        let cargo_home = cfg.home.path().join(".cargo");
        let cargo_home_str = utils::path_to_str(&cargo_home);

        cfg.execute(&["config", "--cargo-home", cargo_home_str]);

        let cfg_content = cfg.read_config();
        assert_eq!(
            cfg_content.trim(),
            &format!("[settings]\ncargo_home = \"{cargo_home_str}\"")
        );
    })
}

#[test]
fn import_full_config() {
    run(|cfg| {
        let conf_path = cfg.data_dir.join("all_settings.toml");
        let conf_path_str = utils::path_to_str(&conf_path);

        cfg.execute(&["config", "--input", conf_path_str]);

        // eliminate line ending difference between different OS by removing whitespaces
        let imported_content: String = cfg.read_config().split_whitespace().collect();
        let expected: String = cfg
            .read_data("all_settings.toml")
            .split_whitespace()
            .collect();
        assert_eq!(imported_content, expected);
    })
}

#[test]
fn config_with_cli_args() {
    run(|cfg| {
        let cargo_home = cfg.home.path().join(".cargo");
        let rustup_home = cfg.home.path().join(".rustup");
        let args = &[
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

        assert_eq!(
            cfg_content.trim(),
            format!(
                r#"[settings]
cargo_home = "{}"
rustup_home = "{}"
rustup_dist_server = "http://example.com/"
rustup_update_root = "http://example.com/rustup/"
proxy = "https://my:1234@my.proxy.com:1234"
no_proxy = "localhost,*.example.com"

[settings.cargo]
git_fetch_with_cli = true
check_revoke = false
default_registry = "mirror"

[settings.cargo.registries.mirror]
index = "http://mirror.example.com/""#,
                utils::path_to_str(&cargo_home),
                utils::path_to_str(&rustup_home),
            )
        )
    })
}

#[test]
fn add_and_rm_registry() {
    run(|cfg| {
        cfg.execute(&[
            "config",
            "registry",
            "add",
            "http://mirror.a.com/",
            "--name",
            "a",
        ]);
        cfg.execute(&[
            "config",
            "registry",
            "add",
            "http://mirror.b.com/",
            "--name",
            "b",
        ]);
        cfg.execute(&["config", "registry", "add", "http://mirror.c.com/"]);

        let cfg_content = cfg.read_config();

        // due to the nature of hashmap, we can't really predict the order of added registries
        // but we can perform a little trick to manually sort the registries.
        let mut splited_output: Vec<_> = cfg_content.trim().split("\n\n").collect();
        splited_output.sort_by(|a, b| a.cmp(b));
        assert_eq!(
            splited_output.join("\n\n"),
            r#"[settings.cargo.registries."mirror.c.com"]
index = "http://mirror.c.com/"

[settings.cargo.registries.a]
index = "http://mirror.a.com/"

[settings.cargo.registries.b]
index = "http://mirror.b.com/""#
        );

        cfg.execute(&["config", "registry", "rm", "b"]);
        cfg.execute(&["config", "registry", "rm", "\"mirror.c.com\""]);

        let cfg_content = utils::read_to_string(&cfg.conf_path);
        assert_eq!(
            cfg_content.trim(),
            r#"[settings.cargo.registries.a]
index = "http://mirror.a.com/""#
        );
    })
}

#[test]
fn restore_config_to_default() {
    run(|cfg| {
        cfg.execute(&["config", "--rustup-dist-server", "http://a.com/"]);

        let cfg_content = cfg.read_config();
        assert_eq!(
            cfg_content.trim(),
            r#"[settings]
rustup_dist_server = "http://a.com/""#
        );

        cfg.execute(&["-y", "config", "--default"]);
        let cfg_content = utils::read_to_string(&cfg.conf_path);
        assert_eq!(cfg_content.trim(), "[settings]");
    })
}
