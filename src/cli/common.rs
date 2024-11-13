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
use log::warn;

pub(crate) fn question_str<Q: Display, A: Display>(
    question: Q,
    extra: Option<&str>,
    default: A,
) -> Result<String> {
    question_str_with_retry(question, extra, None, default, |_| true)
}

pub(crate) fn question_str_with_retry<Q, A, F>(
    question: Q,
    extra: Option<&str>,
    // a prompt shown on top of user input, such as "please enter your choice below".
    prompt: Option<&str>,
    default: A,
    cond: F,
) -> Result<String>
where
    Q: Display,
    A: Display,
    F: Fn(&str) -> bool,
{
    let mut stdout = io::stdout();
    let default_badge = format!("[{}: {default}]", t!("default"));
    // if there's a specified prompt or if the extra lines are too long,
    // we will display the default label above the actual input, making it more visible to users.
    let show_default_above_input =
        prompt.is_some() || extra.map(|e| e.lines().count() > 2).unwrap_or_default();

    // print question, with or without default label.
    if show_default_above_input {
        writeln!(&mut stdout, "{question}")?;
    } else {
        writeln!(&mut stdout, "{question} {default_badge}")?;
    }
    // print extra info, such as a list of selectable options.
    if let Some(ex) = extra {
        writeln!(&mut stdout, "\n{}", ex)?;
    }

    loop {
        if let Some(prmt) = prompt {
            writeln!(&mut stdout, "{prmt} {default_badge}")?;
        }
        write!(&mut stdout, "> ")?;
        _ = stdout.flush();

        let input_raw = readline()?;
        let input = input_raw.trim();
        writeln!(&mut stdout)?;

        if input.is_empty() {
            return Ok(default.to_string());
        } else if !cond(input) {
            continue;
        } else {
            return Ok(input.to_string());
        }
    }
}

/// Display a list of given `choices` and ask for user input that related to choice's index.
///
///
/// # Example
/// Each choice will be labelled with a number that equal to their **index + 1**, for example,
/// when given a choices list as:
///
/// ```ignore
/// let choices = &["do something", "do other things", "abort"];
/// let default = 1;
/// ```
///
/// It will be shown as:
///
/// ```console
/// 1) do something
/// 2) do other things
/// 3) abort
///
/// enter your choice below [default: 1]
/// >
/// ```
///
/// Therefore, if user enter "3", meaning they choose to "abort".
pub(crate) fn question_single_choice<Q, C, D>(
    question: Q,
    choices: &[C],
    default: D,
) -> Result<usize>
where
    Q: Display,
    C: Display,
    D: Display,
{
    let mut choices_prompt = String::new();

    for (idx, choice) in choices.iter().enumerate() {
        let choice_item = format!("{}) {choice}\n", idx + 1);
        choices_prompt.push_str(&choice_item);
    }

    let response = question_str_with_retry(
        question,
        Some(&choices_prompt),
        Some(t!("enter_choice_below").as_ref()),
        default,
        |s| match s.parse::<usize>() {
            Ok(u) if (1..=choices.len()).contains(&u) => true,
            _ => {
                let expected = t!(
                    "ranged_integer",
                    lower_bound = 1,
                    upper_bound = choices.len()
                );
                warn!("{}", t!("invalid_input", actual = s, expect = expected));
                false
            }
        },
    )?;
    Ok(response.parse()?)
}

/// Similar to [`question_single_choice`], but instead of asking user to type one integer,
/// this will ask for a list of integers that are separated by spaces.
pub(crate) fn question_multi_choices<Q, C, D>(
    question: Q,
    choices: &[C],
    default: D,
) -> Result<Vec<usize>>
where
    Q: Display,
    C: Display,
    D: Display,
{
    let mut choices_prompt = String::new();

    for (idx, choice) in choices.iter().enumerate() {
        let choice_item = format!("{}) {choice}\n", idx + 1);
        choices_prompt.push_str(&choice_item);
    }

    let response = question_str_with_retry(
        question,
        Some(&choices_prompt),
        Some(t!("enter_choice_below").as_ref()),
        default,
        |s| {
            if s.split_whitespace().all(
                |s| matches!(s.parse::<usize>(), Ok(idx) if (1..=choices.len()).contains(&idx)),
            ) {
                true
            } else {
                let expected = format!(
                    "{}{}",
                    t!("space_separated_and"),
                    t!(
                        "ranged_integer",
                        lower_bound = 1,
                        upper_bound = choices.len()
                    )
                );
                warn!("{}", t!("invalid_input", actual = s, expect = expected));
                false
            }
        },
    )?;

    Ok(response
        .split_whitespace()
        // The choices should already be valid at this point, but use filter_map just in case.
        .filter_map(|s| s.parse::<usize>().ok())
        .collect())
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
