use std::env;

const TARGET_OVERRIDE_ENV: &str = "HOST_TRIPPLE";
const FILES_TO_TRIGGER_REBUILD: &[&str] = &["../locales/en.json", "../locales/zh-CN.json"];

fn main() {
    println!("cargo:rerun-if-env-changed={TARGET_OVERRIDE_ENV}");
    for file in FILES_TO_TRIGGER_REBUILD {
        println!("cargo:rerun-if-changed={file}");
    }

    let target = env::var(TARGET_OVERRIDE_ENV).unwrap_or(env::var("TARGET").unwrap());
    println!("cargo:rustc-env=TARGET={target}");
}
