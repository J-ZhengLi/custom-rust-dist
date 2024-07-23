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
            None => panic!("Failed to get local host triple."),
        };
        Self {
            triple: host_triple,
        }
    }

    pub fn download(&self, dest: &Path) {
        let download_url = url::Url::parse(&format!(
            "{}/{}/{}/{}",
            RUSTUP_UPDATE_ROOT, "dist", self.triple, RUSTUP_INIT
        ))
        .expect("Failed to init rustup download url");
        match download_from_start(RUSTUP_INIT, &download_url, dest) {
            Ok(_) => (),
            Err(e) => panic!("Failed to download rustup, cause: {:?}", e),
        }
    }
}
