use crate::common::{run, utils};

#[test]
fn new_default_config_should_not_create() {
    run(|cfg| {
        cfg.execute(&["config", "--default"]);

        let cfg_file = cfg.home.path().join(".installer-config");
        assert!(cfg.home.path().is_dir());
        assert!(!cfg_file.is_file());
    })
}

#[test]
fn new_config_with_cargo_home_set() {
    run(|cfg| {
        let cargo_home = cfg.home.path().join(".cargo");
        let cargo_home_str = cargo_home.to_str().unwrap();
        println!("home2: {:?}", &cfg.home);

        cfg.execute(&["config", "--cargo-home", cargo_home_str]);

        let cfg_file = cfg.home.path().join(".installer-config");
        assert!(cfg_file.is_file());
        let cfg_content = utils::read_to_string(cfg_file);
        assert_eq!(
            cfg_content.trim(),
            &format!("[settings]\ncargo_home = '{cargo_home_str}'")
        );
    })
}
