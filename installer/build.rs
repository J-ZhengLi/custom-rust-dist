// I stole this from rustup as well~

use std::env;

include!("rustup/src/dist/triple.rs");

fn from_build() -> Result<PartialTargetTriple, String> {
    let triple = env::var("TARGET").unwrap();
    PartialTargetTriple::new(&triple).ok_or(triple)
}

fn main() {
    println!("cargo:rerun-if-env-changed=TARGET");
    match from_build() {
        Ok(triple) => eprintln!("Computed build based partial target triple: {triple:#?}"),
        Err(s) => {
            eprintln!("Unable to parse target '{s}' as a PartialTargetTriple");
            eprintln!(
                "If you are attempting to bootstrap a new target you may need to adjust the\n\
               permitted values found in rustup/src/dist/triple.rs"
            );
            std::process::abort();
        }
    }
    let target = env::var("TARGET").unwrap();
    println!("cargo:rustc-env=TARGET={target}");
}
