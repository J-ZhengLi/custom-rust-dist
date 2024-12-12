use std::{
    fs,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use url::Url;

pub mod installation;
pub mod manager;
pub mod server;

const TOOLKIT_NAME: &str = "Custom Rust Distribution";

// TODO: (?) we need a `rim_utils`,
// then we can expose `rim::core::directories::get_path_and_create` macro and use it instead

fn debug_dir() -> &'static Path {
    static DEBUG_DIR: OnceLock<PathBuf> = OnceLock::new();
    DEBUG_DIR.get_or_init(|| {
        // safe to unwrap, binary file always have a parent dir
        crate::current_exe().parent().unwrap().to_path_buf()
    })
}

fn mocked_dir() -> &'static Path {
    static MOCKED_DIR: OnceLock<PathBuf> = OnceLock::new();
    MOCKED_DIR.get_or_init(|| {
        let dir = debug_dir().join("mocked");
        fs::create_dir_all(&dir)
            .unwrap_or_else(|_| panic!("unable to create mocked dir at {}", dir.display()));
        dir
    })
}

fn install_dir() -> &'static Path {
    static INSTALL_DIR: OnceLock<PathBuf> = OnceLock::new();
    INSTALL_DIR.get_or_init(|| {
        let dir = mocked_dir().join("installation");
        fs::create_dir_all(&dir)
            .unwrap_or_else(|_| panic!("unable to create mocked install dir at {}", dir.display()));
        dir
    })
}

fn server_dir() -> &'static Path {
    static SERVER_DIR: OnceLock<PathBuf> = OnceLock::new();
    SERVER_DIR.get_or_init(|| {
        let dir = mocked_dir().join("server");
        fs::create_dir_all(&dir)
            .unwrap_or_else(|_| panic!("unable to create mocked server dir at {}", dir.display()));
        dir
    })
}

fn server_dir_url() -> Url {
    let mocked_dist_dir = server_dir();
    Url::from_directory_path(mocked_dist_dir).unwrap_or_else(|_| {
        panic!(
            "path {} cannot be converted to URL",
            mocked_dist_dir.display()
        )
    })
}

fn manager_dir() -> &'static Path {
    static MANAGER_DIR: OnceLock<PathBuf> = OnceLock::new();
    MANAGER_DIR.get_or_init(|| {
        let dir = server_dir().join("manager");
        fs::create_dir_all(&dir).unwrap_or_else(|_| {
            panic!(
                "unable to create mocked manager dist dir at {}",
                dir.display()
            )
        });
        dir
    })
}
