//! This module loads packages from `resources` to pack an offline installer binary.

use cfg_if::cfg_if;
use std::collections::HashMap;

macro_rules! packages {
    ($(($name:literal, $target:literal, $filename:literal)),+) => {{
        let mut __map__ = std::collections::HashMap::new();
        $(
            let __value__ = include_bytes!(concat!("../../resources/packages/", $target, "/", $filename)).as_slice();
            __map__.insert(
                $name,
                $crate::core::offline_packages::PackageSource::new($filename, __value__)
            );
        )*
        $crate::core::offline_packages::OfflinePackages::from(__map__)
    }};
}

/// Represent a set of offline packages' name and package binary in the `resources/packages` pairs.
#[derive(Debug, Default)]
pub(crate) struct OfflinePackages<'p>(pub(crate) HashMap<&'p str, PackageSource<'p>>);

#[derive(Debug, Clone, Copy)]
pub(crate) struct PackageSource<'p> {
    pub(crate) filename: &'p str,
    pub(crate) value: &'p [u8],
}

impl<'p> PackageSource<'p> {
    pub(crate) fn new(filename: &'p str, value: &'p [u8]) -> Self {
        PackageSource { filename, value }
    }
}

impl<'p> From<HashMap<&'p str, PackageSource<'p>>> for OfflinePackages<'p> {
    fn from(value: HashMap<&'p str, PackageSource<'p>>) -> Self {
        OfflinePackages(value)
    }
}

impl<'p> OfflinePackages<'p> {
    pub(crate) fn load() -> Self {
        cfg_if! {
            if #[cfg(all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"))] {
                packages! {
                    ("cargo-llvm-cov", "x86_64-unknown-linux-gnu", "cargo-llvm-cov-x86_64-unknown-linux-gnu.tar.gz")
                }
            } else {
                Self::default()
            }
        }
    }
}
