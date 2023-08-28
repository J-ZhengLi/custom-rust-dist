use cfg_if::cfg_if;

use crate::common::run;

cfg_if! {
    if #[cfg(feature = "cli")] {
        mod cli_config;
        mod cli_init;
    }
}

#[test]
fn tests_path_created() {
    run(|cfg| {
        println!("cfg: {:?}", &cfg);

        assert!(cfg.data_dir.is_dir());
        assert!(cfg.home.path().is_dir());
        assert!(cfg.exe_path.is_file());
    })
}
