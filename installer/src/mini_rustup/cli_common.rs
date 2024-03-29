use anyhow::{anyhow, Context, Result};
use std::io::{stdin, stdout, BufRead, Read, Write};

pub(crate) fn read_line() -> Result<String> {
    let stdin = stdin().lock();
    let mut lines = stdin.lines();
    let lines = lines.next().transpose()?;
    match lines {
        None => Err(anyhow!("no lines found from stdin")),
        Some(v) => Ok(v),
    }
    .context("unable to read from stdin for confirmation")
}

pub(crate) fn confirm(question: &str, default: bool) -> Result<bool> {
    let mut stdout = stdout();
    write!(stdout, "{question} ")?;
    let _ = std::io::stdout().flush();
    let input = read_line()?;

    let r = match &*input.to_lowercase() {
        "y" | "yes" => true,
        "n" | "no" => false,
        "" => default,
        _ => false,
    };

    writeln!(stdout)?;

    Ok(r)
}
