macro_rules! declare_instrcutions {
    ($($name:ident),+) => {
        $(pub(crate) mod $name;)*
        pub(crate) static SUPPORTED_TOOLS: &[&str] = &[$(stringify!($name)),+];

        pub(crate) fn install(tool: &str, path: &std::path::Path, config: &super::install::InstallConfiguration) -> anyhow::Result<()> {
            match tool {
                $(
                    stringify!($name) => $name::install(path, config),
                )*
                _ => anyhow::bail!("no custom install instruction for '{tool}'")
            }
        }

        pub(crate) fn uninstall(tool: &str) -> anyhow::Result<()> {
            match tool {
                $(
                    stringify!($name) => $name::uninstall(),
                )*
                _ => anyhow::bail!("no custom uninstall instruction for '{tool}'")
            }
        }
    };
}

declare_instrcutions!(buildtools, vscode);
