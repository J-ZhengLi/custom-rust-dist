//! Core functionalities of this program
//!
//! Including configuration, toolchain, toolset management.

pub mod components;
mod custom_instructions;
pub(crate) mod directories;
pub mod install;
mod locales;
pub(crate) mod os;
pub(crate) mod parser;
mod path_ext;
pub(crate) mod rustup;
pub mod toolkit;
pub(crate) mod tools;
pub mod try_it;
pub(crate) mod uninstall;
pub mod update;

use std::sync::OnceLock;

pub use locales::Language;
pub(crate) use path_ext::PathExt;

macro_rules! declare_env_vars {
    ($($key:ident),+) => {
        $(pub(crate) const $key: &str = stringify!($key);)*
        #[cfg(windows)]
        pub(crate) static ALL_VARS: &[&str] = &[$($key),+];
    };
}

declare_env_vars!(
    CARGO_HOME,
    RUSTUP_HOME,
    RUSTUP_DIST_SERVER,
    RUSTUP_UPDATE_ROOT
);

pub(crate) const RIM_DIST_SERVER: &str = "https://rust-mirror.obs.cn-north-4.myhuaweicloud.com";

/// Representing the options that user pass to the program, such as
/// `--yes`, `--no-modify-path`, etc.
///
/// This struct will be stored globally for easy access, also make
/// sure the [`set`](GlobalOpts::set) function is called exactly once
/// to initialize the global singleton.
// TODO: add verbose and quiest options
#[derive(Debug, Default)]
pub(crate) struct GlobalOpts {
    pub(crate) verbose: bool,
    pub(crate) quiet: bool,
    pub(crate) yes_to_all: bool,
    pub(crate) no_modify_env: bool,
    pub(crate) no_modify_path: bool,
}

/// Globally stored user options
static GLOBAL_OPTS: OnceLock<GlobalOpts> = OnceLock::new();

impl GlobalOpts {
    /// Initialize a new object and store it globally, will also return a
    /// static reference to the global stored value.
    ///
    /// Note that the value cannot be updated once initialized.
    pub(crate) fn set(
        verbose: bool,
        quiet: bool,
        yes: bool,
        no_modify_env: bool,
        no_modify_path: bool,
    ) -> &'static Self {
        GLOBAL_OPTS.get_or_init(|| Self {
            verbose,
            quiet,
            yes_to_all: yes,
            no_modify_env,
            no_modify_path,
        })
    }

    /// Get the stored global options.
    ///
    /// # Panic
    /// Will panic if `Self` has not been initialized, make sure [`GlobalOpts::new`] is called
    /// prior to this call.
    pub(crate) fn get() -> &'static Self {
        GLOBAL_OPTS
            .get()
            .expect("internal error: global options has not been initialized yet")
    }
}

#[cfg(test)]
mod tests {
    use super::GlobalOpts;

    #[test]
    fn global_opts_set_and_get() {
        GlobalOpts::set(true, false, true, true, false);

        let opts = GlobalOpts::get();
        assert_eq!(opts.verbose, true);
        assert_eq!(opts.quiet, false);
        assert_eq!(opts.yes_to_all, true);
        assert_eq!(opts.no_modify_env, true);
        assert_eq!(opts.no_modify_path, false);
    }
}
