use rim_test_support::prelude::*;
use rim_test_support::file;

#[rim_test]
fn case() {
    snapbox::cmd::Command::rim_dev()
        .arg("--help")
        .assert()
        .success()
        .stdout_eq(file!["stdout.log"])
        .stderr_eq(file!["stderr.log"]);
}
