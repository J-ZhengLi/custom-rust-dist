//! Default values used across the whole crate

macro_rules! crate_const {
    ($($name:ident: $type:ty = $val:literal;)*) => {
        $(pub(crate) const $name: $type = $val;)*
    };
}

crate_const! {
    RUSTUP_DIST_SERVER: &str = "https://static.rust-lang.org/";
    RUSTUP_UPDATE_ROOT: &str = "https://static.rust-lang.org/rustup/";
}
