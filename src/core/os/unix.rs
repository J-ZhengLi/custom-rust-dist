use crate::{
    core::{
        cargo_config::CargoConfig, InstallConfiguration, Installation, UninstallConfiguration,
        Uninstallation,
    },
    utils,
};
use anyhow::{anyhow, Context, Result};

impl Installation for InstallConfiguration {
    // On linux, persistent env vars needs to be written in `.profile`, `.bash_profile`, etc.
    // Rustup already did all the dirty work by writting an entry in those files
    // to invoke `$CARGO_HOME/env.{sh|fish}`. Sadly we'll have to re-implement a similar procedure here,
    // because rustup will not write those file if a user has choose to pass `--no-modify-path`.
    // Which is not ideal for env vars such as `RUSTUP_DIST_SERVER`.
    fn config_rustup_env_vars(&self) -> Result<()> {
        let vars_raw = self.env_vars()?;
        for sh in shell::get_available_shells() {
            // Shell commands to set env var, such as `export KEY='val'`
            let vars_shell_lines = vars_raw
                .iter()
                .map(|(k, v)| sh.env_var_string(k, v))
                .collect::<Vec<_>>()
                .join("\n");
            // This string will be wrapped in a certain identifier comments.
            let vars_shell_string = sh.script_content(&vars_shell_lines);
            for rc in sh.update_rcs() {
                let vars_to_write = match utils::read_to_string(&rc) {
                    // Assume env configuration exist if the section label presents.
                    Ok(content) if content.contains(shell::RC_FILE_SECTION_END) => continue,
                    Ok(content) if !content.ends_with('\n') => &format!("\n{}", &vars_shell_string),
                    _ => &vars_shell_string,
                };

                // Ok to append env config section now
                utils::write_file(&rc, vars_to_write, true).with_context(|| {
                    format!(
                        "failed to append environment vars to shell profile: '{}'",
                        rc.display()
                    )
                })?;
            }
        }
        Ok(())
    }

    fn config_cargo(&self) -> Result<()> {
        let mut config = CargoConfig::new();
        if let Some((name, url)) = &self.cargo_registry {
            config.add_source(name, url.to_owned(), true);
        }

        let config_toml = config.to_toml()?;
        if !config_toml.trim().is_empty() {
            let config_path = self.cargo_home().join("config.toml");
            utils::write_file(config_path, &config_toml, false)?;
        }

        Ok(())
    }
}

impl Uninstallation for UninstallConfiguration {
    // This is basically removing the section marked with `rustup config section` in shell profiles.
    fn remove_rustup_env_vars(&self) -> Result<()> {
        for sh in shell::get_available_shells() {
            for rc in sh.rcfiles().iter().filter(|rc| rc.is_file()) {
                let content = utils::read_to_string(rc)?;
                let new_content = remove_sub_string_between(
                    content,
                    shell::RC_FILE_SECTION_START,
                    &format!("{}\n", shell::RC_FILE_SECTION_END)
                ).ok_or_else(
                    || anyhow!(
                        "unable to remove rustup config section from shell profile: '{}'. \
                        This could mean that the section was already removed, or the section label is broken, \
                        please try manually removing any command wrapped within comments that saying \
                        'rustup config section' if there are any.",
                        rc.display()
                    )
                )?;
                utils::write_file(rc, &new_content, false)?;
            }
        }
        Ok(())
    }
}

fn remove_sub_string_between(mut input: String, start: &str, end: &str) -> Option<String> {
    let start_pos = input.find(start)?;
    let end_pos = input.find(end)? + end.len();
    input.drain(start_pos..=end_pos);
    Some(input)
}

/// Unix shell module, contains methods that are dedicated in configuring rustup env vars.
// TODO?: Most code in this module are modified from rustup's `shell.rs`, this is not ideal for long term,
// as the file in rustup could change drasically in the future and somehow we'll need to update
// this as well. But as for now, this looks like the only feasible solution.
mod shell {
    use crate::utils;
    use anyhow::{bail, Result};
    use std::{env, path::PathBuf};

    type Shell = Box<dyn UnixShell>;

    pub(super) const RC_FILE_SECTION_START: &str = "# ===== rustup config section START =====";
    pub(super) const RC_FILE_SECTION_END: &str = "# ===== rustup config section END =====";

    pub(super) trait UnixShell {
        // Detects if a shell "exists". Users have multiple shells, so an "eager"
        // heuristic should be used, assuming shells exist if any traces do.
        fn does_exist(&self) -> bool;

        // Gives all rcfiles of a given shell that Rustup is concerned with.
        // Used primarily in checking rcfiles for cleanup.
        fn rcfiles(&self) -> Vec<PathBuf>;

        // Gives rcs that should be written to.
        fn update_rcs(&self) -> Vec<PathBuf>;

        /// Format a shell command to set env var.
        fn env_var_string(&self, key: &'static str, val: &str) -> String {
            format!("export {key}='{val}'")
        }

        /// Wraps given content between a pair of identifiers.
        ///
        /// Such identifiers are comments defined as [`RC_FILE_SECTION_START`] and [`RC_FILE_SECTION_END`].
        fn script_content(&self, raw_content: &str) -> String {
            format!(
                "{RC_FILE_SECTION_START}\n\
                {raw_content}\n\
                {RC_FILE_SECTION_END}\n"
            )
        }
    }

    struct Posix;
    struct Bash;
    struct Zsh;
    struct Fish;

    impl UnixShell for Posix {
        fn does_exist(&self) -> bool {
            true
        }

        fn rcfiles(&self) -> Vec<PathBuf> {
            vec![utils::home_dir().join(".profile")]
        }

        fn update_rcs(&self) -> Vec<PathBuf> {
            // Write to .profile even if it doesn't exist. It's the only rc in the
            // POSIX spec so it should always be set up.
            self.rcfiles()
        }
    }

    impl UnixShell for Bash {
        fn does_exist(&self) -> bool {
            !self.update_rcs().is_empty()
        }

        fn rcfiles(&self) -> Vec<PathBuf> {
            // Bash also may read .profile, however Rustup already includes handling
            // .profile as part of POSIX and always does setup for POSIX shells.
            [".bash_profile", ".bash_login", ".bashrc"]
                .iter()
                .map(|rc| utils::home_dir().join(rc))
                .collect()
        }

        fn update_rcs(&self) -> Vec<PathBuf> {
            self.rcfiles()
                .into_iter()
                .filter(|rc| rc.is_file())
                .collect()
        }
    }

    impl Zsh {
        fn zdotdir() -> Result<PathBuf> {
            use std::ffi::OsStr;
            use std::os::unix::ffi::OsStrExt;

            if matches!(env::var("SHELL"), Ok(sh) if sh.contains("zsh")) {
                match env::var("ZDOTDIR") {
                    Ok(dir) if !dir.is_empty() => Ok(PathBuf::from(dir)),
                    _ => bail!("Zsh setup failed."),
                }
            } else {
                match std::process::Command::new("zsh")
                    .args(["-c", "echo -n $ZDOTDIR"])
                    .output()
                {
                    Ok(io) if !io.stdout.is_empty() => {
                        Ok(PathBuf::from(OsStr::from_bytes(&io.stdout)))
                    }
                    _ => bail!("Zsh setup failed."),
                }
            }
        }
    }

    impl UnixShell for Zsh {
        fn does_exist(&self) -> bool {
            // zsh has to either be the shell or be callable for zsh setup.
            matches!(env::var("SHELL"), Ok(sh) if sh.contains("zsh")) || utils::cmd_exist("zsh")
        }

        fn rcfiles(&self) -> Vec<PathBuf> {
            [Zsh::zdotdir().ok(), Some(utils::home_dir())]
                .iter()
                .filter_map(|dir| dir.as_ref().map(|p| p.join(".zshenv")))
                .collect()
        }

        fn update_rcs(&self) -> Vec<PathBuf> {
            // zsh can change $ZDOTDIR both _before_ AND _during_ reading .zshenv,
            // so we: write to $ZDOTDIR/.zshenv if-exists ($ZDOTDIR changes before)
            // OR write to $HOME/.zshenv if it exists (change-during)
            // if neither exist, we create it ourselves, but using the same logic,
            // because we must still respond to whether $ZDOTDIR is set or unset.
            // In any case we only write once.
            self.rcfiles()
                .into_iter()
                .filter(|env| env.is_file())
                .chain(self.rcfiles())
                .take(1)
                .collect()
        }
    }

    impl UnixShell for Fish {
        fn does_exist(&self) -> bool {
            // fish has to either be the shell or be callable for fish setup.
            matches!(env::var("SHELL"), Ok(sh) if sh.contains("fish")) || utils::cmd_exist("fish")
        }

        // > "$XDG_CONFIG_HOME/fish/conf.d" (or "~/.config/fish/conf.d" if that variable is unset) for the user
        // from <https://github.com/fish-shell/fish-shell/issues/3170#issuecomment-228311857>
        fn rcfiles(&self) -> Vec<PathBuf> {
            let mut res = env::var("XDG_CONFIG_HOME")
                .ok()
                .map(|p| vec![PathBuf::from(p).join("fish/conf.d/rustup.fish")])
                .unwrap_or_default();
            res.push(utils::home_dir().join(".config/fish/conf.d/rustup.fish"));

            res
        }

        fn env_var_string(&self, key: &'static str, val: &str) -> String {
            format!("set -Ux {key} {val}")
        }

        fn update_rcs(&self) -> Vec<PathBuf> {
            // The first rcfile takes precedence.
            match self.rcfiles().into_iter().next() {
                Some(path) => vec![path],
                None => vec![],
            }
        }
    }

    pub(super) fn get_available_shells() -> impl Iterator<Item = Shell> {
        let supported_shells: Vec<Shell> = vec![
            Box::new(Posix),
            Box::new(Bash),
            Box::new(Zsh),
            Box::new(Fish),
        ];

        supported_shells.into_iter().filter(|sh| sh.does_exist())
    }
}

#[cfg(test)]
mod tests {
    use super::shell;

    #[test]
    fn remove_labeled_section() {
        let mock_profile = "\
#
# ~/.bash_profile
#

[[ -f ~/.bashrc ]] && . ~/.bashrc

# ===== rustup config section START =====
export CARGO_HOME='/path/to/cargo'
export RUSTUP_HOME='/path/to/rustup'
export RUSTUP_DIST_SERVER='https://example.com'
export RUSTUP_UPDATE_ROOT='https://example.com/rustup'
# ===== rustup config section END =====
. \"$HOME/.cargo/env\"
";

        let new = super::remove_sub_string_between(
            mock_profile.to_string(),
            shell::RC_FILE_SECTION_START,
            shell::RC_FILE_SECTION_END,
        )
        .unwrap();
        assert_eq!(
            new,
            "\
#
# ~/.bash_profile
#

[[ -f ~/.bashrc ]] && . ~/.bashrc

. \"$HOME/.cargo/env\"
"
        );
    }
}
