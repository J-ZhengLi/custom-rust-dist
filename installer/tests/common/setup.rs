use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::{env, fs, panic};

use tempfile::TempDir;

use super::utils::execute;

static EXE_PATH: OnceLock<PathBuf> = OnceLock::new();

#[cfg(windows)]
const EXE_SUFFIX: &str = ".exe";
#[cfg(not(windows))]
const EXE_SUFFIX: &str = "";

#[derive(Debug)]
pub struct TestConfig {
    pub root: PathBuf,
    pub data_dir: PathBuf,
    pub exe_path: PathBuf,
    /// Mocked user `HOME` path.
    pub home: TempDir,
}

impl TestConfig {
    pub fn init() -> Self {
        let tests_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");
        let home_dir = tests_dir.join("home");
        // need to make sure this root exist so that a temp dir could be created.
        fs::create_dir_all(&home_dir).expect(
            "unable to create `home` directory, \
            try manually create one under `tests`",
        );
        let home = tempfile::Builder::new()
            .prefix("home_")
            .tempdir_in(&home_dir)
            .expect("unable to create temp home");

        Self {
            data_dir: tests_dir.join("data"),
            exe_path: exe_path().to_path_buf(),
            root: tests_dir,
            home,
        }
    }

    pub fn execute(&self, args: &[&str]) {
        execute(&self.exe_path, args)
    }

    pub fn setup_env(&self) {
        #[cfg(unix)]
        let home_var = "HOME";
        #[cfg(windows)]
        let home_var = "USERPROFILE";

        env::set_var(home_var, &self.home.path());
    }
}

fn exe_path() -> &'static Path {
    EXE_PATH.get_or_init(|| {
        let test_exe_path = env::current_exe().expect("unable to locate current test exe");
        let mut built_exe_dir = test_exe_path.parent().unwrap();
        if built_exe_dir.ends_with("deps") {
            built_exe_dir = built_exe_dir.parent().unwrap();
        }
        built_exe_dir.join(format!("{}{EXE_SUFFIX}", env!("CARGO_PKG_NAME")))
    })
}

pub fn run<F>(test: F)
where
    F: FnOnce(&TestConfig) -> () + panic::UnwindSafe,
{
    let cfg = TestConfig::init();
    cfg.setup_env();
    panic::catch_unwind(|| test(&cfg)).unwrap();
}
