//! Common API for CLI interactions.
//!
//! You might think some of these look like they are from rustup.
//! You are god d*mn right!
//!                         --- Walter White

use std::{
    collections::HashMap,
    fmt::Display,
    io::{self, BufRead, Write},
};

use anyhow::{Context, Result};
use indexmap::IndexMap;
use log::warn;

use crate::components::Component;

/// A "convenient" helper macro to [`question_single_choice`].
///
/// This wraps all things one typically needs, such as storing user's input
/// then handling the action of each individual choice.
///
/// # Example
/// ```ignore
/// let s: &str;
///
/// handle_user_choice!(
///     "choose a word to greet",
///     // 1 is the default value
///     1,
///     s => {
///         1 "world" => { "hello world!" },
///         2 "rust" => { "hello rust!" },
///         3 "rim" => { "hello rim!" }
///     }
/// );
///
/// // if user enter 2, then this assertion succeed:
/// assert_eq!(s, "hello rust!");
/// ```
///
/// **The user will see a prompt as below:**
///
/// ```console
/// choose a word to greet
///
/// 1) world
/// 2) rust
/// 3) rim
///
/// please enter your choice below [default: 1]
/// >
/// ```
macro_rules! handle_user_choice {
    ($ques:expr, $default:expr, $assign:expr => { $($idx:literal $choice:expr => $action:block),+ }) => {
        #[allow(clippy::needless_late_init)]
        {
            let choices__ = &[ $($choice),* ];
            let choice__ = $crate::cli::common::question_single_choice($ques, choices__, $default)?;
            $assign = match choice__ {
                $($idx => $action),*
                _ => unreachable!("`question_single_choice` ensures choice's range")
            };
        }
    };
}
pub(crate) use handle_user_choice;

/// A map containing a component's version difference.
///
/// The keys of this map is the name of the component, the value is a pair of (maybe) strings
/// with the first one being the current version, second one being the target version.
pub(crate) type VersionDiffMap<'c> = HashMap<&'c str, (Option<&'c str>, Option<&'c str>)>;
/// A map contains the selected components with their indexes in the full component list.
///
/// Notice that this is an [`IndexMap`], which means the order will be preserved.
pub(crate) type ComponentChoices<'c> = IndexMap<&'c Component, usize>;

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
            write!(&mut stdout, "{prmt} ")?;
        }
        if show_default_above_input {
            writeln!(&mut stdout, "{default_badge}")?;
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
pub fn pause() -> Result<()> {
    let mut stdout = io::stdout();
    writeln!(&mut stdout, "\n{}", t!("pause_prompt"))?;
    _ = stdout.flush();

    readline()?;
    Ok(())
}

#[cfg(not(windows))]
pub fn pause() -> Result<()> {
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

/// Specify the string after each component's name, which is usually wrapped in parenthese.
#[derive(Debug, Default, Clone, Copy)]
pub(crate) enum ComponentDecoration<'c> {
    /// Version change info, i.e.:
    /// `hello-world (0.1.0 -> 0.2.0)`
    VersionDiff(&'c VersionDiffMap<'c>),
    /// Pre-installation labels, including `installed|required` to indicate whether a tool
    /// is installed or required but not installed, i.e.:
    /// `hello-world (required)`
    InstalledOrRequired,
    /// A label to show during confirmation page, indicates whether a tool is installed but will be re-install,
    /// i.e.: `hello-world (installed, reinstalling)`
    Confirmation,
    /// No decoration string, this is the default variant, i.e.:
    /// `hello-world`
    #[default]
    None,
}

impl ComponentDecoration<'_> {
    pub(crate) fn for_component(self, comp: &Component) -> String {
        match self {
            Self::None => String::new(),
            Self::InstalledOrRequired => {
                if comp.installed {
                    format!(" ({})", t!("installed"))
                } else if comp.required {
                    format!(" ({})", t!("required"))
                } else {
                    String::new()
                }
            }
            Self::Confirmation => {
                if comp.installed {
                    format!(" ({})", t!("reinstall"))
                } else {
                    String::new()
                }
            }
            Self::VersionDiff(diff_map) => diff_map
                .get(comp.name.as_str())
                .map(|(from, to)| {
                    format!(" ({} -> {})", from.unwrap_or("N/A"), to.unwrap_or("N/A"))
                })
                .unwrap_or_else(String::new),
        }
    }
}

/// A helper struct that takes a list of [`Component`], and convert it
/// to a list of component strings.
pub(crate) struct ComponentListBuilder<'c, I: IntoIterator<Item = &'c Component>> {
    components: I,
    show_desc: bool,
    decoration: ComponentDecoration<'c>,
}

impl<'c, I: IntoIterator<Item = &'c Component>> ComponentListBuilder<'c, I> {
    pub(crate) fn new(components: I) -> Self {
        Self {
            components,
            show_desc: false,
            decoration: ComponentDecoration::default(),
        }
    }

    pub(crate) fn show_desc(mut self, yes: bool) -> Self {
        self.show_desc = yes;
        self
    }

    pub(crate) fn decorate(mut self, deco: ComponentDecoration<'c>) -> Self {
        self.decoration = deco;
        self
    }

    pub(crate) fn build(self) -> Vec<String> {
        self.components
            .into_iter()
            .map(|c| {
                let deco = self.decoration.for_component(c);
                let desc = if self.show_desc {
                    format!("\n\t{}: {}", t!("description"), &c.desc)
                } else {
                    String::new()
                };
                format!("{}{deco}{desc}", &c.name)
            })
            .collect()
    }
}
