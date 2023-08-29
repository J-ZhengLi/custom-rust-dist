use std::path::{Path, PathBuf};
use std::sync::{Mutex, Once, OnceLock};
use std::{env, fs, panic};

use anyhow::Result;
use tempfile::TempDir;

use super::utils;
use rupe::mini_rustup::target_triple;

static EXE_PATH: OnceLock<PathBuf> = OnceLock::new();
/// Global lock for tests using [`run`] wrapper,
/// because such test jobs would rely on modified env variables,
/// thus will not be safe to run concurrently.
static LOCK: Mutex<()> = Mutex::new(());
static CREATE_MOCKED_RUSTUP_ROOT: Once = Once::new();

type EnvPairs = Vec<(String, Option<String>)>;

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
    /// The default configuration path defined by the installer,
    /// which should be placed under `home` directory.
    pub conf_path: PathBuf,
    pub mocked_server_root: PathBuf,
}

impl TestConfig {
    pub fn init() -> Self {
        let tests_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");
        let home_dir = tests_dir.join("home");
        // need to make sure this root exist so that a temp dir could be created.
        fs::create_dir_all(&home_dir).expect(
            "unable to create test home, \
            try manually create a `./tests/home` directory",
        );

        let home = tempfile::Builder::new()
            .prefix("home_")
            .tempdir_in(&home_dir)
            .expect("unable to create temp home");
        let conf_path = home.path().join(format!(".{}-config", super::APPNAME));
        let data_dir = tests_dir.join("data");

        // make a mock rustup dist server
        let mocked_server_root = home_dir.join("server");
        CREATE_MOCKED_RUSTUP_ROOT.call_once(|| {
            create_mocked_server(&mocked_server_root).expect("unable to create mocked server")
        });

        Self {
            data_dir,
            exe_path: exe_path().to_path_buf(),
            root: tests_dir,
            home,
            conf_path,
            mocked_server_root,
        }
    }

    pub fn setup_env(&self) -> EnvPairs {
        let orig_home_var = env::var(HOME_VAR).ok();
        let orig_cargo_home = env::var("CARGO_HOME").ok();
        let orig_rustup_home = env::var("RUSTUP_HOME").ok();

        env::set_var(HOME_VAR, &self.home.path());
        env::remove_var("CARGO_HOME");
        env::remove_var("RUSTUP_HOME");

        vec![
            (HOME_VAR.into(), orig_home_var),
            ("CARGO_HOME".into(), orig_cargo_home),
            ("RUSTUP_HOME".into(), orig_rustup_home),
        ]
    }

    /// Convenient method to execute the built 'installer' executable with given args.
    pub fn execute(&self, args: &[&str]) {
        utils::execute(&self.exe_path, args)
    }

    pub fn execute_with_env<'a, I: IntoIterator<Item = (&'a str, &'a str)>>(
        &self,
        args: &[&str],
        env: I,
    ) {
        utils::execute_with_env(&self.exe_path, args, env)
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

/// This will build `rustup-init` from `rustup` submodule, and then copy
/// the result binary into a several locations under `tests/home` to produce a
/// mocked rustup update root
// FIXME: DRY
fn create_mocked_server(server_path: &Path) -> Result<()> {
    // build rustup-init from `rusup` submodule, this is preventing tests from downloading
    // rustup-init binary from
    let rustup_source_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("rustup");
    if !rustup_source_dir.is_dir() {
        panic!(
            "unable to find rustup source directory, \n\
            make sure `rustup` submodule was up to date by running \n\
            `git submodule --init && git submodule --update`"
        );
    }
    let target_triple = target_triple();
    let target = target_triple.as_ref();

    // build rustup-init
    let _ = std::process::Command::new("cargo")
        .current_dir(&rustup_source_dir)
        .args(["build", "--locked", "--release", "--target", &target])
        .status()
        .expect("unable to build rustup");

    let bin_name = format!("rustup-init{EXE_SUFFIX}");
    let source_bin = rustup_source_dir
        .join("target")
        .join(target)
        .join("release")
        .join(&bin_name);

    let rustup_root = server_path.join("rustup");
    let dist_dir = rustup_root.join("dist").join(target);
    fs::create_dir_all(&dist_dir)?;
    let dist_path = dist_dir.join(&bin_name);
    fs::copy(&source_bin, dist_path)?;

    let mut version = String::new();
    let rustup_manifest = fs::read_to_string(rustup_source_dir.join("Cargo.toml"))
        .expect("unable to read cargo manifest from `rustup`");
    for line in rustup_manifest.lines() {
        if let Some(ver) = line.strip_prefix("version =") {
            let trim_pat: &[_] = &['\"', ' ', '\n'];
            version = ver.trim_matches(trim_pat).to_string();
            break;
        }
    }
    let archive_dir = rustup_root.join("archive").join(version).join(target);
    fs::create_dir_all(&archive_dir)?;
    let archive_path = archive_dir.join(&bin_name);
    fs::copy(&source_bin, archive_path)?;

    Ok(())
}

fn exe_path() -> &'static Path {
    EXE_PATH.get_or_init(|| {
        let test_exe_path = env::current_exe().expect("unable to locate current test exe");
        let mut built_exe_dir = test_exe_path.parent().unwrap();
        if built_exe_dir.ends_with("deps") {
            built_exe_dir = built_exe_dir.parent().unwrap();
        }
        built_exe_dir.join(format!("{}{EXE_SUFFIX}", super::APPNAME))
    })
}

fn restore_env_vars(vars: &EnvPairs) {
    for (key, val) in vars {
        if let Some(v) = val {
            env::set_var(key, v);
        } else {
            env::remove_var(key);
        }
    }
}

pub enum Profile {
    PreInit,
    InitDefault,
}

impl Profile {
    pub fn run(&self, cfg: &TestConfig) {
        match self {
            Profile::PreInit => (),
            Profile::InitDefault => {
                let rustup_update_root =
                    url::Url::from_directory_path(cfg.mocked_server_root.join("rustup")).unwrap();

                cfg.execute_with_env(
                    &[
                        "-y",
                        "init",
                        "--no-modify-path",
                        "--rustup-update-root",
                        rustup_update_root.as_str(),
                    ],
                    [("PATH", "")],
                );
            }
        }
    }
}

pub fn run<F>(test: F)
where
    F: FnOnce(&TestConfig) -> () + panic::UnwindSafe,
{
    run_with_profile(Profile::PreInit, test)
}

pub fn run_with_profile<F>(profile: Profile, test: F)
where
    F: FnOnce(&TestConfig) -> () + panic::UnwindSafe,
{
    // NB: locking the current test thread to take care of environment variables,
    // the result of the lock should not matter since it does not hold any data.
    let _guard = LOCK.lock();

    let cfg = TestConfig::init();
    let backup = cfg.setup_env();

    profile.run(&cfg);

    panic::catch_unwind(|| test(&cfg)).unwrap();
    restore_env_vars(&backup);
}
