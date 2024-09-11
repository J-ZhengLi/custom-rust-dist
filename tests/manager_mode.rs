use rust_i18n::{i18n, t};
use std::process::Command;
use std::{env, fs};

i18n!("locales", fallback = "en");

fn main() {
    let test_bin = env::current_exe().expect("failed to get the path of current test binary");
    let installer_bin = test_bin
        .parent()
        .unwrap()
        .with_file_name(format!("installer{}", env::consts::EXE_SUFFIX));
    if !installer_bin.is_file() {
        panic!("debug binary has not been built, run `cargo build` first");
    }
    // make a copy of the `installer`
    let manager_bin = installer_bin.with_file_name(format!(
        "{}-manager{}",
        t!("vendor_en"),
        env::consts::EXE_SUFFIX
    ));
    fs::copy(installer_bin, &manager_bin).unwrap();

    let args_for_manager = env::args_os().skip(1).collect::<Vec<_>>();
    let status = Command::new(&manager_bin)
        .args(args_for_manager)
        .status()
        .unwrap();
    println!(
        "\nmanager exited with status code: {}",
        status.code().unwrap_or(-1)
    );
}
