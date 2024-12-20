use rim_test_support::file;
use rim_test_support::prelude::*;

#[rim_test]
fn case() {
    snapbox::cmd::Command::rim_dev()
        .arg("run-manager")
        .arg("--help")
        .assert()
        .success()
        .stdout_eq(file!["stdout.log"])
        .stderr_eq(file!["stderr.log"]);
}
