use rim_test_support::file;
use rim_test_support::prelude::*;

#[rim_test]
fn case() {
    snapbox::cmd::Command::rim_cli()
        .arg("--help")
        .assert()
        .success()
        .stdout_eq(file!["stdout.log"])
        .stderr_eq(file!["stderr.log"]);
}
