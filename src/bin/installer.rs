use anyhow::Result;
use clap::Parser;
use custom_rust::{cli, utils};

fn main() -> Result<()> {
    match utils::lowercase_program_name() {
        Some(s) if s.starts_with("manager") => cli::Manager::parse().execute(),
        // Every thing else will fallback to installer mode
        _ => cli::Installer::parse().execute(),
    }
}
