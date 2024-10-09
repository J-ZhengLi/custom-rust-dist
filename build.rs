use std::env;

const TARGET_OVERRIDE_ENV: &str = "TARGET_OVERRIDE";

fn main() {
    println!("cargo:rerun-if-env-changed={TARGET_OVERRIDE_ENV}");

    let target = env::var(TARGET_OVERRIDE_ENV).unwrap_or(env::var("TARGET").unwrap());
    println!("cargo:rustc-env=TARGET={target}");

    let profile = env::var("PROFILE").unwrap();
    println!("cargo:rustc-env=PROFILE={profile}");
}
