mod cli_config;

use crate::common::run;

#[test]
fn tests_path_created() {
    run(|cfg| {
        println!("exe path: {}", &cfg.exe_path.display());

        assert!(cfg.data_dir.is_dir());
        assert!(cfg.home_dir.is_dir());
        assert!(cfg.exe_path.is_file());
    })
}
