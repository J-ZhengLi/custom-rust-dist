use std::{env, path::Path};

use super::install_dir_from_exe_path;
use crate::core::install::InstallConfiguration;
use crate::core::uninstall::{UninstallConfiguration, Uninstallation};
use crate::core::EnvConfig;
use crate::utils;
use anyhow::{anyhow, Context, Result};

impl EnvConfig for InstallConfiguration {
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
                .map(|(k, v)| sh.to_env_var_string(k, &format!("'{v}'")))
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

        // Update vars for current process
        for (key, val) in vars_raw {
            env::set_var(key, val);
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
                    shell::RC_FILE_SECTION_END
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

    fn remove_self(&self) -> Result<()> {
        // TODO: Run `rustup self uninstall` first
        // TODO: Remove possibly installed extensions for other software, such as `vs-code` plugins.

        // Remove the installer dir.
        let installed_dir = install_dir_from_exe_path()?;
        std::fs::remove_dir_all(installed_dir)?;
        Ok(())
    }
}

fn remove_sub_string_between(input: String, start: &str, end: &str) -> Option<String> {
    // TODO: this might not be an optimized solution.
    let start_pos = input.lines().position(|line| line == start)?;
    let end_pos = input.lines().position(|line| line == end)?;
    assert!(
        end_pos >= start_pos,
        "Interal Error: Failed deleting sub string, the start pos is larger than end pos"
    );
    let result = input
        .lines()
        .take(start_pos)
        .chain(input.lines().skip(end_pos + 1))
        .collect::<Vec<_>>()
        .join("\n");
    Some(result)
}

/// Get the enclosing string between two desired **lines**.
fn get_sub_string_between(input: &str, start: &str, end: &str) -> Option<String> {
    let start_pos = input.lines().position(|line| line == start)?;
    let end_pos = input.lines().position(|line| line == end)?;
    assert!(
        end_pos >= start_pos,
        "Interal Error: Failed extracting sub string, the start pos is larger than end pos"
    );
    let result = input
        .lines()
        .skip(start_pos + 1)
        .take(end_pos - start_pos - 1)
        .collect::<Vec<_>>()
        .join("\n");
    Some(result)
}

pub(super) fn add_to_path(path: &Path) -> Result<()> {
    let old_path = env::var_os("PATH").unwrap_or_default();
    let pathbuf = path.to_path_buf();
    let path_str = utils::path_to_str(path)?;

    let splited = env::split_paths(&old_path).collect::<Vec<_>>();
    let should_update_current_env = !splited.contains(&pathbuf);
    let mut new_path = splited;
    new_path.insert(0, pathbuf);

    // Add the new path to bash profiles
    for sh in shell::get_available_shells() {
        for rc in sh.update_rcs() {
            let rc_content = utils::read_to_string(&rc)?;
            let new_content = if let Some(existing_configs) = get_sub_string_between(
                &rc_content,
                shell::RC_FILE_SECTION_START,
                shell::RC_FILE_SECTION_END,
            ) {
                // Find the line that is setting path variable
                let maybe_setting_path =
                    existing_configs.lines().find(|line| line.contains("PATH"));
                // Safe to unwrap, the function could only return `None` when removing.
                let new_content = sh
                    .command_to_update_path(maybe_setting_path, path_str, false)
                    .unwrap();

                let mut new_configs = existing_configs.clone();
                if let Some(setting_path) = maybe_setting_path {
                    new_configs = existing_configs.replace(setting_path, &new_content);
                } else {
                    new_configs.push('\n');
                    new_configs.push_str(&new_content);
                }

                rc_content.replace(&existing_configs, &new_configs)
            } else {
                // No previous configuration (this might never happed tho)
                let path_configs = sh.command_to_update_path(None, path_str, false).unwrap();
                sh.script_content(&path_configs)
            };

            utils::write_file(&rc, &new_content, true).with_context(|| {
                format!(
                    "failed to append PATH variable to shell profile: '{}'",
                    rc.display()
                )
            })?;
        }
    }

    // Apply the new path to current process
    if should_update_current_env {
        env::set_var("PATH", env::join_paths(new_path)?);
    }

    Ok(())
}

/// Unix shell module, contains methods that are dedicated in configuring rustup env vars.
// TODO?: Most code in this module are modified from rustup's `shell.rs`, this is not ideal for long term,
// as the file in rustup could change drasically in the future and somehow we'll need to update
// this as well. But as for now, this looks like the only feasible solution.
mod shell {
    // Suggestion of this lint looks worse and doesn't have any improvement.
    #![allow(clippy::collapsible_else_if)]

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
        fn to_env_var_string(&self, key: &'static str, val: &str) -> String {
            format!("export {key}={val}")
        }

        /// Wraps given content between a pair of identifiers.
        ///
        /// Such identifiers are comments defined as [`RC_FILE_SECTION_START`] and [`RC_FILE_SECTION_END`].
        fn script_content(&self, raw_content: &str) -> String {
            format!(
                "{RC_FILE_SECTION_START}\n\
                {raw_content}\n\
                {RC_FILE_SECTION_END}"
            )
        }

        /// Update the PATH export command, which should be `export PATH="..."` on bash like shells,
        /// and `set -Ux PATH ...` on fish shell.
        ///
        /// If the remove flag is set to `true`, this will attempt to return the `old_command` but without `path_str`.
        fn command_to_update_path(
            &self,
            old_command: Option<&str>,
            path_str: &str,
            remove: bool,
        ) -> Option<String> {
            if let Some(cmd) = old_command {
                let path_str_with_spliter = format!("{path_str}:");
                if remove {
                    Some(cmd.replace(&path_str_with_spliter, ""))
                } else {
                    let where_to_insert = cmd.find('\"')? + 1;
                    let mut new_cmd = cmd.to_string();
                    new_cmd.insert_str(where_to_insert, &path_str_with_spliter);
                    Some(new_cmd)
                }
            } else {
                if remove {
                    None
                } else {
                    Some(self.to_env_var_string("PATH", &format!("\"{path_str}:$PATH\"")))
                }
            }
        }
    }

    pub(super) struct Posix;
    pub(super) struct Bash;
    pub(super) struct Zsh;
    pub(super) struct Fish;

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

        fn to_env_var_string(&self, key: &'static str, val: &str) -> String {
            format!("set -Ux {key} {val}")
        }

        fn update_rcs(&self) -> Vec<PathBuf> {
            // The first rcfile takes precedence.
            match self.rcfiles().into_iter().next() {
                Some(path) => vec![path],
                None => vec![],
            }
        }

        fn command_to_update_path(
            &self,
            old_command: Option<&str>,
            path_str: &str,
            remove: bool,
        ) -> Option<String> {
            if let Some(cmd) = old_command {
                let path_str_with_spliter = format!("{path_str} ");
                if remove {
                    Some(cmd.replace(&path_str_with_spliter, ""))
                } else {
                    let (before_path, after_path) = cmd.split_once("PATH")?;
                    Some(format!("{before_path}PATH {path_str}{after_path}"))
                }
            } else {
                if remove {
                    None
                } else {
                    Some(self.to_env_var_string("PATH", &format!("{path_str} $PATH")))
                }
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
    use crate::utils;
    use std::path::PathBuf;

    use super::shell::{self, UnixShell};

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

. \"$HOME/.cargo/env\""
        );
    }

    #[test]
    fn labeled_section_at_the_end() {
        let mocked_profile = r#"
alias autoremove='sudo pacman -Rcns $(pacman -Qdtq)'
. "$HOME/.cargo/env"

# ===== rustup config section START =====
export CARGO_HOME='/home/.cargo'
export RUSTUP_HOME='/home/.rustup'
# ===== rustup config section END ====="#;
        let new = super::remove_sub_string_between(
            mocked_profile.to_string(),
            shell::RC_FILE_SECTION_START,
            shell::RC_FILE_SECTION_END,
        )
        .unwrap();
        assert_eq!(
            new,
            r#"
alias autoremove='sudo pacman -Rcns $(pacman -Qdtq)'
. "$HOME/.cargo/env"
"#
        );
    }

    // TODO: Move this test to `utils`
    #[test]
    fn path_ambiguity() {
        let with_dots = PathBuf::from("/path/to/home/./my_app/../my_app");
        let without_dots = PathBuf::from("/path/to/home/my_app");
        assert_ne!(with_dots, without_dots);

        let with_dots_comps: PathBuf = with_dots.components().collect();
        let without_dots_comps: PathBuf = without_dots.components().collect();
        // Components take `..` accountable in case of symlink.
        assert_ne!(with_dots_comps, without_dots_comps);

        let with_dots_normalized = utils::to_nomalized_abspath(&with_dots, None).unwrap();
        let without_dots_normalized = utils::to_nomalized_abspath(&without_dots, None).unwrap();
        assert_eq!(with_dots_normalized, without_dots_normalized);
    }

    #[test]
    fn estimated_install_dir() {
        let mocked_exe_path = PathBuf::from("/path/to/home/my_app/.cargo/bin/my_app");
        let anc_count = mocked_exe_path.components().count();
        // Components count root dir (/) as well, so there should be 8 components.
        assert_eq!(anc_count, 8);
        let maybe_install_dir: PathBuf = mocked_exe_path.components().take(anc_count - 3).collect();
        assert_eq!(maybe_install_dir, PathBuf::from("/path/to/home/my_app"));
    }

    #[test]
    fn extract_labeled_section() {
        let mock_profile = r#"\
#
# ~/.bash_profile
#

[[ -f ~/.bashrc ]] && . ~/.bashrc

# ===== rustup config section START =====
export CARGO_HOME='/path/to/cargo'
export RUSTUP_HOME='/path/to/rustup'
export PATH="/path/to/bin:$PATH"
# ===== rustup config section END =====
. \"$HOME/.cargo/env\"
"#;

        let wanted = super::get_sub_string_between(
            mock_profile,
            shell::RC_FILE_SECTION_START,
            shell::RC_FILE_SECTION_END,
        )
        .unwrap();
        assert_eq!(
            wanted,
            r#"export CARGO_HOME='/path/to/cargo'
export RUSTUP_HOME='/path/to/rustup'
export PATH="/path/to/bin:$PATH""#
        );
    }

    #[test]
    fn insert_path_default() {
        let shell = shell::Bash;
        let path_str = "/path/to/bin";
        let cmd = shell.command_to_update_path(None, path_str, false);

        assert_eq!(cmd, Some("export PATH=\"/path/to/bin:$PATH\"".to_string()));
    }

    #[test]
    fn insert_path_with_old_cmd_default() {
        let shell = shell::Bash;
        let path_str = "/path/to/bin";
        let old_cmd = r#"export PATH="/path/to/tool/bin:$PATH""#;
        let cmd = shell.command_to_update_path(Some(old_cmd), path_str, false);

        assert_eq!(
            cmd,
            Some("export PATH=\"/path/to/bin:/path/to/tool/bin:$PATH\"".to_string())
        );
    }

    #[test]
    fn remove_path_with_no_old_cmd_default() {
        let shell = shell::Bash;
        let path_str = "/path/to/bin";
        let cmd = shell.command_to_update_path(None, path_str, true);

        assert!(cmd.is_none());
    }

    #[test]
    fn remove_path_with_old_cmd_default() {
        let shell = shell::Bash;
        let path_str = "/path/to/bin";
        let old_cmd = r#"export PATH="/path/to/tool/bin:/path/to/bin:$PATH""#;
        let cmd = shell.command_to_update_path(Some(old_cmd), path_str, true);

        assert_eq!(
            cmd,
            Some("export PATH=\"/path/to/tool/bin:$PATH\"".to_string())
        );
    }

    #[test]
    fn insert_path_fish() {
        let shell = shell::Fish;
        let path_str = "/path/to/bin";
        let cmd = shell.command_to_update_path(None, path_str, false);

        assert_eq!(cmd, Some("set -Ux PATH /path/to/bin $PATH".to_string()));
    }

    #[test]
    fn insert_path_with_old_cmd_fish() {
        let shell = shell::Fish;
        let path_str = "/path/to/bin";
        let old_cmd = "set -Ux PATH /path/to/tool/bin $PATH";
        let cmd = shell.command_to_update_path(Some(old_cmd), path_str, false);

        assert_eq!(
            cmd,
            Some("set -Ux PATH /path/to/bin /path/to/tool/bin $PATH".to_string())
        );
    }

    #[test]
    fn remove_path_with_no_old_cmd_fish() {
        let shell = shell::Fish;
        let path_str = "/path/to/bin";
        let cmd = shell.command_to_update_path(None, path_str, true);

        assert!(cmd.is_none());
    }

    #[test]
    fn remove_path_with_old_cmd_fish() {
        let shell = shell::Fish;
        let path_str = "/path/to/bin";
        let old_cmd = "set -Ux PATH /path/to/tool/bin /path/to/bin $PATH";
        let cmd = shell.command_to_update_path(Some(old_cmd), path_str, true);

        assert_eq!(
            cmd,
            Some("set -Ux PATH /path/to/tool/bin $PATH".to_string())
        );
    }
}
