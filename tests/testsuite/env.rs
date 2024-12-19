use rim_test_support::prelude::*;

#[rim_test]
fn target_override() {
    let target = env!("TARGET");

    println!("target: {target}");

    if let Ok(override_target) = std::env::var("HOST_TRIPPLE") {
        assert_eq!(override_target, target);
        return;
    }

    #[cfg(all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"))]
    assert_eq!(target, "x86_64-unknown-linux-gnu");
    #[cfg(all(target_arch = "x86_64", target_os = "windows", target_env = "gnu"))]
    assert_eq!(target, "x86_64-pc-windows-gnu");
    #[cfg(all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"))]
    assert_eq!(target, "x86_64-pc-windows-msvc");
}