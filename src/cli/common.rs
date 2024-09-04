use std::{
    fmt::Display,
    io::{self, BufRead, Write},
};

use anyhow::{Context, Result};

pub(crate) fn question_str<Q: Display, A: Display>(
    question: Q,
    extra: Option<String>,
    default: A,
) -> Result<String> {
    writeln!(io::stdout(), "{question} [{}: {default}]", t!("default"))?;
    if let Some(ex) = extra {
        writeln!(io::stdout())?;
        writeln!(io::stdout(), "{}", ex)?;
    }

    _ = io::stdout().flush();

    let input = readline()?;
    writeln!(io::stdout())?;

    if input.is_empty() {
        Ok(default.to_string())
    } else {
        Ok(input)
    }
}

fn readline() -> Result<String> {
    let mut input_buf = String::new();
    io::stdin()
        .lock()
        .read_line(&mut input_buf)
        .context("unable to read from standard input")?;
    Ok(input_buf)
}
