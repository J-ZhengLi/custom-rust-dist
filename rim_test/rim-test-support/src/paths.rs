use std::cell::RefCell;
use std::io::ErrorKind;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{env, fs};
use std::path::{Path, PathBuf}; 
use std::sync::{Mutex, OnceLock};

use crate::t;

static GLOBAL_ROOT_TEST_DIR: &str = "rim"; 

static GLOBAL_ROOT_DIR: OnceLock<Mutex<Option<PathBuf>>> = OnceLock::new();

thread_local! {
    static TEST_ID: RefCell<Option<usize>> = RefCell::new(None);
}

fn set_global_root_dir(tmp_dir: Option<&'static str>) {
    let mut root_dir = GLOBAL_ROOT_DIR
        .get_or_init(|| Default::default())
        .lock()
        .unwrap();
    
    if root_dir.is_none() {
        let mut dir = match tmp_dir {
            Some(td) => {
                PathBuf::from(td)
            },
            None => {
                let mut path = t!(env::current_exe());
                path.pop(); // chop off exe name
                path.pop(); // chop off "deps"
                path.push("tmp");
                path.mkdir_p();
                path
            },
        };

        dir.push(GLOBAL_ROOT_TEST_DIR);
        *root_dir = Some(dir);
    }
}

fn global_root_dir() -> PathBuf {
    let root_dir = GLOBAL_ROOT_DIR
        .get_or_init(|| Default::default())
        .lock()
        .unwrap();
    match root_dir.as_ref() {
        Some(p) => p.clone(),
        None => unreachable!("GLOBAL ROOT DIR not set yet"),
    }
}

fn test_root() -> PathBuf {
    let id = TEST_ID.with(|n| {
        n.borrow().expect("Failed to get test thread id")    
    });

    let mut test_root_dir = global_root_dir();
    test_root_dir.push(format!("t{}", id));
    test_root_dir
}

pub fn init_root(tmp_dir: Option<&'static str>) {
    static RUN_TEST_ID: AtomicUsize = AtomicUsize::new(0);

    let id = RUN_TEST_ID.fetch_add(1, Ordering::SeqCst);
    TEST_ID.with(|n| *n.borrow_mut() = Some(id));

    set_global_root_dir(tmp_dir);
    
    let test_root = test_root();
    test_root.rm_rf();
    test_root.mkdir_p();
}

/// Path to the current test home
///
/// example: $RIM_TARGET_TMPDIR/rim/t0/home
pub fn home() -> PathBuf {
    let mut path = test_root();
    path.push("home");
    path.mkdir_p();
    path
}

/// Path to the current test asset home
/// 
/// example: $CARGO_MANIFEST_DIR/asset
pub fn assets_home() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.pop();
    path.push("tests");
    path.push("assets");
    path
}

pub trait TestPathExt {
    fn mkdir_p(&self);
    fn rm_rf(&self);
}

impl TestPathExt for Path {
    fn mkdir_p(&self) {
        fs::create_dir_all(self)
            .unwrap_or_else(|e| panic!("failed to mkdir dir {}: \n cause: \n {}", self.display(), e))
    }
    
    fn rm_rf(&self) {
        let meta = match self.symlink_metadata() {
            Ok(meta) => meta,
            Err(e) => {
                if e.kind() == ErrorKind::NotFound {
                    return;
                }
                panic!("failed to remove {:?}, could not read: {:?}", self, e);
            }
        };

        if meta.is_dir() {
            if let Err(e) = fs::remove_dir_all(self) {
                panic!("failed to remove {:?}: {:?}", self, e)
            }
        } else if let Err(e) = fs::remove_file(self) {
            panic!("failed to remove {:?}: {:?}", self, e)
        }
    }
}

impl TestPathExt for PathBuf {
    fn mkdir_p(&self) {
        self.as_path().mkdir_p()
    }
    
    fn rm_rf(&self) {
        self.as_path().rm_rf()
    }
}