//! The mini (stolen) version of rustup libraries.
//!
//! There are many many amazing api(s) that I wanna take advantage of,
//! but I'm too lazy to write them myself, so... why not just... copy & paste.
//!
//! Sorry, rustup.

#[cfg(feature = "cli")]
pub(crate) mod cli_common;
pub(crate) mod dist;
pub(crate) mod utils;

use dist::TargetTriple;

pub(crate) fn target_triple() -> TargetTriple {
    // Get build triple
    let triple = dist::TargetTriple::from_build();
    // For windows x86 builds seem slow when used with windows defender.
    // The website defaulted to i686-windows-gnu builds for a long time.
    // This ensures that we update to a version thats appropriate for users
    // and also works around if the website messed up the detection.
    // If someone really wants to use another version, they still can enforce
    // that using the environment variable RUSTUP_OVERRIDE_HOST_TRIPLE.
    #[cfg(windows)]
    let triple = dist::TargetTriple::from_host().unwrap_or(triple);
    triple
}
