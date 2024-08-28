//! Core functionalities of this program
//!
//! Including configuration, toolchain, toolset management.

mod custom_instructions;
pub mod install;
mod os;
pub(crate) mod parser;
pub(crate) mod rustup;
pub(crate) mod tools;
pub mod try_it;
pub(crate) mod uninstall;

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
