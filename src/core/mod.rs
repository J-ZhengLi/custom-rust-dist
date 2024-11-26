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
pub(crate) mod rustup;
pub mod toolkit;
pub(crate) mod tools;
pub mod try_it;
pub(crate) mod uninstall;
pub mod update;

pub use locales::Language;

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
