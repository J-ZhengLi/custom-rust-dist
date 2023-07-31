mod cli_config;

use crate::common::run;

#[test]
fn tests_path_created() {
    run(|cfg| {
        println!("cfg: {:?}", &cfg);

        assert!(cfg.data_dir.is_dir());
        assert!(cfg.home.path().is_dir());
        assert!(cfg.exe_path.is_file());
    })
}
