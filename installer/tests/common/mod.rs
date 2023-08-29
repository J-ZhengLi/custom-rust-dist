mod setup;
pub mod utils;

pub use setup::{run, run_with_profile, Profile, TestConfig};

pub const APPNAME: &str = env!("CARGO_PKG_NAME");
