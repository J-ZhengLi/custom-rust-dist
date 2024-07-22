use crate::core::{Configuration, Preinstallation};
use anyhow::Result;

impl Preinstallation for Configuration {
    fn config_cargo() -> Result<()> {
        Ok(())
    }

    fn config_rustup_env_vars() -> Result<()> {
        // On linux, persistent env vars needs to be written in `.profile`, `.bash_profile`, etc.
        // Rustup already did all the dirty work by writting an entry in those files
        // to invoke `$CARGO_HOME/env.{sh|fish}`. Sadly we'll have to re-implement a similar procedure here,
        // because rustup will not write those file if a user has choose to pass `--no-modify-path`.
        // Which is not ideal for env vars such as `RUSTUP_DIST_SERVER`.
        Ok(())
    }
}

/// Unix shell module, contains methods that are dedicated in configuring rustup env vars.
// TODO?: Most code in this module are modified from rustup's `shell.rs`, this is not ideal for long term,
// as the file in rustup could change drasically in the future and somehow we'll need to update
// this as well. But as for now, this looks like the only feasible solution.
mod shell {
    use crate::utils;
    use anyhow::{bail, Result};
    use std::{env, path::PathBuf};

    pub(super) trait UnixShell {
        // Detects if a shell "exists". Users have multiple shells, so an "eager"
        // heuristic should be used, assuming shells exist if any traces do.
        fn does_exist(&self) -> bool;

        // Gives all rcfiles of a given shell that Rustup is concerned with.
        // Used primarily in checking rcfiles for cleanup.
        fn rcfiles(&self) -> Vec<PathBuf>;

        // Gives rcs that should be written to.
        fn update_rcs(&self) -> Vec<PathBuf>;

        // Writes the relevant env file.
        fn env_script(&self, raw_content: &str) -> ShellScript {
            let content = format!(
                "\n\
                # ===== rustup config section START =====\n\
                {raw_content}\n
                # ===== rustup config section END =====\n"
            );
            ShellScript {
                name: "env",
                content,
            }
        }
    }

    #[derive(Debug, PartialEq)]
    pub(super) struct ShellScript {
        content: String,
        name: &'static str,
    }

    struct Posix;
    struct Bash;
    struct Zsh;
    struct Fish;

    type Shell = Box<dyn UnixShell>;

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
