macro_rules! declare_instrcutions {
    ($($name:ident),+) => {
        $(pub(crate) mod $name;)*
        pub (crate) static SUPPORTED_TOOLS: &[&str] = &[$(stringify!($name)),+];
    };
}

declare_instrcutions!(buildtools, vscode);
