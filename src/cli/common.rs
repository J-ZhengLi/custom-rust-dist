//! Common API for CLI interactions.
//!
//! You might think these look like they are from rustup.
//! You are god d*mn right!
//!                         --- Walter White

use std::{
    fmt::Display,
    io::{self, BufRead, Write},
};

use anyhow::{Context, Result};

pub(crate) fn question_str<Q: Display, A: Display>(
    question: Q,
    extra: Option<&str>,
    default: A,
) -> Result<String> {
    question_str_with_retry(question, extra, default, |_| true)
}

pub(crate) fn question_str_with_retry<Q, A, F>(
    question: Q,
    extra: Option<&str>,
    default: A,
    cond: F,
) -> Result<String>
where
    Q: Display,
    A: Display,
    F: Fn(&str) -> bool,
{
    let mut stdout = io::stdout();

    writeln!(&mut stdout, "{question} [{}: \"{default}\"]", t!("default"))?;
    if let Some(ex) = extra {
        writeln!(&mut stdout, "\n{}", ex)?;
    }

    loop {
        write!(&mut stdout, "> ")?;
        _ = stdout.flush();

        let input = readline()?;
        writeln!(&mut stdout)?;

        if input.is_empty() {
            return Ok(default.to_string());
        } else if !cond(&input) {
            continue;
        } else {
            return Ok(input);
        }
    }
}

pub(crate) fn confirm<Q: Display>(question: Q, default: bool) -> Result<bool> {
    let mut stdout = io::stdout();
    writeln!(
        &mut stdout,
        "{} ({})",
        question,
        if default { "Y/n" } else { "y/N" }
    )?;
    write!(&mut stdout, "> ")?;
    _ = stdout.flush();

    let input = readline()?;
    let choice = match input.to_lowercase().as_str() {
        "y" | "yes" => true,
        "n" | "no" => false,
        "" => default,
        _ => false,
    };

    writeln!(&mut stdout)?;
    Ok(choice)
}

pub(crate) enum Confirm {
    Yes,
    No,
    Abort,
}

pub(crate) fn confirm_install() -> Result<Confirm> {
    let mut stdout = io::stdout();

    writeln!(&mut stdout, "\n{}\n", t!("question_install_options"))?;
    writeln!(&mut stdout, "1) {} ({})", t!("confirm"), t!("default"))?;
    writeln!(&mut stdout, "2) {}", t!("reenter"))?;
    writeln!(&mut stdout, "3) {}", t!("cancel"))?;
    write!(&mut stdout, "> ")?;
    _ = stdout.flush();

    let input = readline()?;
    let choice = match input.as_str() {
        "1" | "" => Confirm::Yes,
        "2" => Confirm::No,
        _ => Confirm::Abort,
    };

    writeln!(&mut stdout)?;
    Ok(choice)
}

#[cfg(windows)]
pub(crate) fn pause() -> Result<()> {
    let mut stdout = io::stdout();
    writeln!(&mut stdout, "\n{}", t!("pause_prompt"))?;
    _ = stdout.flush();

    readline()?;
    Ok(())
}

fn readline() -> Result<String> {
    let mut input_buf = String::new();
    io::stdin()
        .lock()
        .read_line(&mut input_buf)
        .context("unable to read from standard input")?;
    Ok(input_buf.trim().to_string())
}
