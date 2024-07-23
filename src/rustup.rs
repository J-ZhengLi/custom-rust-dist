use std::path::Path;

use crate::utils::cli::download_from_start;
use crate::utils::{HostTriple, Process};

const RUSTUP_DIST_SERVER: &str = "https://mirrors.tuna.tsinghua.edu.cn/rustup";
const RUSTUP_UPDATE_ROOT: &str = "https://mirrors.tuna.tsinghua.edu.cn/rustup/rustup";

#[cfg(windows)]
const RUSTUP_INIT: &str = "rustup-init.exe";
#[cfg(not(windows))]
const RUSTUP_INIT: &str = "rustup-init";

pub struct Rustup {
    triple: HostTriple,
}

impl Rustup {
    pub fn new(process: &Process) -> Self {
        let host_triple = match HostTriple::from_host(process) {
            Some(host_triple) => host_triple,
            None => todo!("Do warning processing"),
        };
        Self {
            triple: host_triple,
        }
    }

    pub fn download(&self, dest: &Path) {
        let download_url = match url::Url::parse(&format!(
            "{}/{}/{}/{}",
            RUSTUP_UPDATE_ROOT, "dist", self.triple, RUSTUP_INIT
        )) {
            Ok(url) => url,
            Err(_) => todo!("Do warning processing"),
        };
        match download_from_start(RUSTUP_INIT, &download_url, dest) {
            Ok(_) => (),
            Err(_) => todo!("Do warning processing"),
        }
    }
}
