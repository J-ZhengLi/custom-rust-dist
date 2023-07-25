

pub(crate) enum FrameContent {
    /// Contains essential configurations and maybe installing for `rustup`.
    Setup,
    /// Just a loading animation
    Loading,
    /// Main interface of toolchain management, such as add/remove toolchains,
    /// or add/remove components to existing toolchain etc.
    /// 
    /// This should shows up only after the app has been configured.
    ToolchainManager,
    /// Software settings panel, contains multiple tabs.
    Settings,
}
