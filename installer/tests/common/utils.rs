//! Convient helper functions

use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn execute<P: AsRef<OsStr>>(program: P, args: &[&str]) {
    let _ = Command::new(&program)
        .args(args)
        .status()
        .unwrap_or_else(|e| {
            panic!(
                "unable to execute '{} {}': {}",
                program.as_ref().to_string_lossy().to_string(),
                args.join(" "),
                e.to_string(),
            )
        });
}

pub fn execute_with_env<'a, P, I>(program: P, args: &[&str], env: I)
where
    P: AsRef<OsStr>,
    I: IntoIterator<Item = (&'a str, &'a str)>,
{
    let _ = Command::new(program.as_ref())
        .args(args)
        .envs(env)
        .status()
        .unwrap_or_else(|e| {
            panic!(
                "unable to execute '{} {}': {}",
                program.as_ref().to_string_lossy().to_string(),
                args.join(" "),
                e.to_string(),
            )
        });
}

pub fn read_to_string<P: AsRef<Path>>(path: P) -> String {
    fs::read_to_string(path.as_ref()).unwrap_or_else(|e| {
        panic!(
            "unable to read '{}': {}",
            path.as_ref().display(),
            e.to_string()
        )
    })
}

pub fn path_to_str(path: &Path) -> &str {
    path.to_str()
        .unwrap_or_else(|| panic!("unable to convert path '{path:?}' to str"))
}
