use std::path::PathBuf;
use std::sync::OnceLock;
use std::{env, fs, panic};

use anyhow::Result;

static CFG: OnceLock<TestConfig> = OnceLock::new();

#[cfg(windows)]
const EXE_SUFFIX: &str = ".exe";
#[cfg(not(windows))]
const EXE_SUFFIX: &str = "";

#[derive(Debug)]
pub struct TestConfig {
    pub root: PathBuf,
    pub data_dir: PathBuf,
    pub home_dir: PathBuf,
    pub exe_path: PathBuf,
}

impl TestConfig {
    pub fn new() -> Self {
        let tests_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");
        let test_exe_path = env::current_exe().expect("unable to locate current test exe");

        // copy-pasta from rustup tests/mock/clitools
        let mut built_exe_dir = test_exe_path.parent().unwrap();
        if built_exe_dir.ends_with("deps") {
            built_exe_dir = built_exe_dir.parent().unwrap();
        }

        let exe_name = format!("{}{EXE_SUFFIX}", env!("CARGO_PKG_NAME"));

        Self {
            data_dir: tests_dir.join("data"),
            home_dir: tests_dir.join("home"),
            exe_path: built_exe_dir.join(exe_name),
            root: tests_dir,
        }
    }
}

fn setup() -> Result<&'static TestConfig> {
    let cfg = CFG.get_or_init(|| TestConfig::new());
    if !cfg.home_dir.is_dir() {
        fs::create_dir_all(&cfg.home_dir)?;
    }
    Ok(cfg)
}

pub fn run<F>(test: F)
where
    F: FnOnce(&TestConfig) -> () + panic::UnwindSafe,
{
    let cfg = setup().expect("unable to create test environment");
    panic::catch_unwind(|| {
        test(cfg);
    })
    .unwrap();
}
