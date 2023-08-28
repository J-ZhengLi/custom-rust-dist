mod setup;
pub mod utils;

pub use setup::{run, TestConfig};

pub const APPNAME: &str = env!("CARGO_PKG_NAME");
