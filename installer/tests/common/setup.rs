use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::{env, fs, panic};

use tempfile::TempDir;

use super::utils;

static EXE_PATH: OnceLock<PathBuf> = OnceLock::new();
/// Global lock for tests using [`run`] wrapper,
/// because such test jobs would rely on modified env variables,
/// thus will not be safe to run concurrently.
static LOCK: Mutex<()> = Mutex::new(());

#[cfg(windows)]
const EXE_SUFFIX: &str = ".exe";
#[cfg(not(windows))]
const EXE_SUFFIX: &str = "";
#[cfg(windows)]
const HOME_VAR: &str = "USERPROFILE";
#[cfg(unix)]
const HOME_VAR: &str = "HOME";

#[derive(Debug)]
pub struct TestConfig {
    pub root: PathBuf,
    pub data_dir: PathBuf,
    pub exe_path: PathBuf,
    /// Mocked user `HOME` path.
    pub home: TempDir,
    pub installer_home: PathBuf,
    /// The default configuration path defined by the installer,
    /// which should be placed under `home` directory.
    pub conf_path: PathBuf,
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
        let installer_home = home.path().join(env!("CARGO_PKG_NAME"));
        let conf_path = installer_home.join("config");

        Self {
            data_dir: tests_dir.join("data"),
            exe_path: exe_path().to_path_buf(),
            root: tests_dir,
            home,
            installer_home,
            conf_path,
        }
    }

    pub fn setup_env<'a>(&self) -> Vec<(&'a str, Option<String>)> {
        let orig_home_var = env::var(HOME_VAR).ok();
        env::set_var(HOME_VAR, &self.home.path());

        vec![(HOME_VAR, orig_home_var)]
    }

    /// Convenient method to execute the built 'installer' executable with given args.
    pub fn execute(&self, args: &[&str]) {
        utils::execute(&self.exe_path, args)
    }

    /// Convenient method to read a single file under `data_dir`
    /// and returning its content as String.
    pub fn read_data<P: AsRef<Path>>(&self, filename: P) -> String {
        let path = self.data_dir.join(filename);
        utils::read_to_string(path)
    }

    pub fn read_config(&self) -> String {
        let raw = utils::read_to_string(&self.conf_path);
        // The output of configuration uses single quote instead of double quote
        // on some machine, which is bad for our tests,
        // so we need to unify them into double quotes.
        raw.replace('\'', "\"")
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

fn restore_env_vars(vars: &[(&str, Option<String>)]) {
    for (key, val) in vars {
        if let Some(v) = val {
            env::set_var(key, v);
        } else {
            env::remove_var(key);
        }
    }
}

pub fn run<F>(test: F)
where
    F: FnOnce(&TestConfig) -> () + panic::UnwindSafe,
{
    // NB: locking the current test thread to take care of environment variables,
    // the result of the lock should not matter since it does not hold any data.
    let _guard = LOCK.lock();

    let cfg = TestConfig::init();
    let backup = cfg.setup_env();
    panic::catch_unwind(|| test(&cfg)).unwrap();
    restore_env_vars(&backup);
}
