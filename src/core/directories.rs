use std::path::Path;

/// Declare a statically allocated `OnceLock` path, and create that directory if it does not exists.
macro_rules! get_path_and_create {
    ($path_ident:ident, $init:expr) => {{
        static $path_ident: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();
        let __path__ = $path_ident.get_or_init(|| $init);
        $crate::utils::ensure_dir(__path__)
            .expect("unable to create one of the directory under installation folder");
        __path__
    }};
}

pub(crate) trait RimDir {
    fn install_dir(&self) -> &Path;

    fn cargo_home(&self) -> &Path {
        get_path_and_create!(CARGO_HOME_DIR, self.install_dir().join(".cargo"))
    }

    fn cargo_bin(&self) -> &Path {
        get_path_and_create!(CARGO_BIN_DIR, self.cargo_home().join("bin"))
    }

    fn rustup_home(&self) -> &Path {
        get_path_and_create!(RUSTUP_HOME_DIR, self.install_dir().join(".rustup"))
    }

    fn temp_root(&self) -> &Path {
        get_path_and_create!(TEMP_DIR, self.install_dir().join("temp"))
    }

    fn tools_dir(&self) -> &Path {
        get_path_and_create!(TOOLS_DIR, self.install_dir().join("tools"))
    }
}
