// Shhhh, don't tell anyone that I stole these from rustup...
// Because I really need this to determine host triples during runtime,
// and I'm too lazy to write on my own.

use std::fmt;
use std::ops::Deref;

// Linux hosts don't indicate clib in uname, however binaries only
// run on boxes with the same clib, as expected.
#[cfg(all(not(windows), not(target_env = "musl")))]
const TRIPLE_X86_64_UNKNOWN_LINUX: &str = "x86_64-unknown-linux-gnu";
#[cfg(all(not(windows), target_env = "musl"))]
const TRIPLE_X86_64_UNKNOWN_LINUX: &str = "x86_64-unknown-linux-musl";
#[cfg(all(not(windows), not(target_env = "musl")))]
const TRIPLE_AARCH64_UNKNOWN_LINUX: &str = "aarch64-unknown-linux-gnu";
#[cfg(all(not(windows), target_env = "musl"))]
const TRIPLE_AARCH64_UNKNOWN_LINUX: &str = "aarch64-unknown-linux-musl";

// MIPS platforms don't indicate endianness in uname, however binaries only
// run on boxes with the same endianness, as expected.
// Hence we could distinguish between the variants with compile-time cfg()
// attributes alone.
#[cfg(all(not(windows), target_endian = "big"))]
static TRIPLE_MIPS_UNKNOWN_LINUX_GNU: &str = "mips-unknown-linux-gnu";
#[cfg(all(not(windows), target_endian = "little"))]
static TRIPLE_MIPS_UNKNOWN_LINUX_GNU: &str = "mipsel-unknown-linux-gnu";

#[cfg(all(not(windows), target_endian = "big"))]
static TRIPLE_MIPS64_UNKNOWN_LINUX_GNUABI64: &str = "mips64-unknown-linux-gnuabi64";
#[cfg(all(not(windows), target_endian = "little"))]
static TRIPLE_MIPS64_UNKNOWN_LINUX_GNUABI64: &str = "mips64el-unknown-linux-gnuabi64";

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TargetTriple(String);

impl Deref for TargetTriple {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TargetTriple {
    pub fn new(name: &str) -> Self {
        Self(name.to_string())
    }

    pub(crate) fn from_build() -> Self {
        if let Some(triple) = option_env!("RUSTUP_OVERRIDE_BUILD_TRIPLE") {
            Self::new(triple)
        } else {
            Self::new(env!("TARGET"))
        }
    }

    pub(crate) fn from_host() -> Option<Self> {
        #[cfg(windows)]
        fn inner() -> Option<TargetTriple> {
            use std::mem;

            /// Get the host architecture using `IsWow64Process2`. This function
            /// produces the most accurate results (supports detecting aarch64), but
            /// it is only available on Windows 10 1511+, so we use `GetProcAddress`
            /// to maintain backward compatibility with older Windows versions.
            fn arch_primary() -> Option<&'static str> {
                use winapi::shared::minwindef::BOOL;
                use winapi::um::libloaderapi::{GetModuleHandleA, GetProcAddress};
                use winapi::um::processthreadsapi::GetCurrentProcess;
                use winapi::um::winnt::HANDLE;

                const IMAGE_FILE_MACHINE_ARM64: u16 = 0xAA64;
                const IMAGE_FILE_MACHINE_AMD64: u16 = 0x8664;
                const IMAGE_FILE_MACHINE_I386: u16 = 0x014c;

                #[allow(non_snake_case)]
                let IsWow64Process2: unsafe extern "system" fn(
                    HANDLE,
                    *mut u16,
                    *mut u16,
                )
                    -> BOOL = unsafe {
                    let module = GetModuleHandleA(b"kernel32.dll\0" as *const u8 as *const i8);
                    if module.is_null() {
                        return None;
                    }
                    let process =
                        GetProcAddress(module, b"IsWow64Process2\0" as *const u8 as *const i8);
                    if process.is_null() {
                        return None;
                    }
                    mem::transmute(process)
                };

                let mut _machine = 0;
                let mut native_machine = 0;
                unsafe {
                    // cannot fail; handle does not need to be closed.
                    let process = GetCurrentProcess();
                    if IsWow64Process2(process, &mut _machine, &mut native_machine) == 0 {
                        return None;
                    }
                };
                match native_machine {
                    IMAGE_FILE_MACHINE_AMD64 => Some("x86_64"),
                    IMAGE_FILE_MACHINE_I386 => Some("i686"),
                    IMAGE_FILE_MACHINE_ARM64 => Some("aarch64"),
                    _ => None,
                }
            }

            /// Get the host architecture using `GetNativeSystemInfo`.
            /// Does not support detecting aarch64.
            fn arch_fallback() -> Option<&'static str> {
                use winapi::um::sysinfoapi::GetNativeSystemInfo;

                const PROCESSOR_ARCHITECTURE_AMD64: u16 = 9;
                const PROCESSOR_ARCHITECTURE_INTEL: u16 = 0;

                let mut sys_info;
                unsafe {
                    sys_info = mem::zeroed();
                    GetNativeSystemInfo(&mut sys_info);
                }

                match unsafe { sys_info.u.s() }.wProcessorArchitecture {
                    PROCESSOR_ARCHITECTURE_AMD64 => Some("x86_64"),
                    PROCESSOR_ARCHITECTURE_INTEL => Some("i686"),
                    _ => None,
                }
            }

            // Default to msvc
            let arch = arch_primary().or_else(arch_fallback)?;
            let msvc_triple = format!("{arch}-pc-windows-msvc");
            Some(TargetTriple(msvc_triple))
        }

        #[cfg(not(windows))]
        fn inner() -> Option<TargetTriple> {
            use std::ffi::CStr;
            use std::mem;

            let mut sys_info;
            let (sysname, machine) = unsafe {
                sys_info = mem::zeroed();
                if libc::uname(&mut sys_info) != 0 {
                    return None;
                }

                (
                    CStr::from_ptr(sys_info.sysname.as_ptr()).to_bytes(),
                    CStr::from_ptr(sys_info.machine.as_ptr()).to_bytes(),
                )
            };

            let host_triple = match (sysname, machine) {
                (_, b"arm") if cfg!(target_os = "android") => Some("arm-linux-androideabi"),
                (_, b"armv7l") if cfg!(target_os = "android") => Some("armv7-linux-androideabi"),
                (_, b"armv8l") if cfg!(target_os = "android") => Some("armv7-linux-androideabi"),
                (_, b"aarch64") if cfg!(target_os = "android") => Some("aarch64-linux-android"),
                (_, b"i686") if cfg!(target_os = "android") => Some("i686-linux-android"),
                (_, b"x86_64") if cfg!(target_os = "android") => Some("x86_64-linux-android"),
                (b"Linux", b"x86_64") => Some(TRIPLE_X86_64_UNKNOWN_LINUX),
                (b"Linux", b"i686") => Some("i686-unknown-linux-gnu"),
                (b"Linux", b"mips") => Some(TRIPLE_MIPS_UNKNOWN_LINUX_GNU),
                (b"Linux", b"mips64") => Some(TRIPLE_MIPS64_UNKNOWN_LINUX_GNUABI64),
                (b"Linux", b"arm") => Some("arm-unknown-linux-gnueabi"),
                (b"Linux", b"armv7l") => Some("armv7-unknown-linux-gnueabihf"),
                (b"Linux", b"armv8l") => Some("armv7-unknown-linux-gnueabihf"),
                (b"Linux", b"aarch64") => Some(TRIPLE_AARCH64_UNKNOWN_LINUX),
                (b"Darwin", b"x86_64") => Some("x86_64-apple-darwin"),
                (b"Darwin", b"i686") => Some("i686-apple-darwin"),
                (b"FreeBSD", b"x86_64") => Some("x86_64-unknown-freebsd"),
                (b"FreeBSD", b"i686") => Some("i686-unknown-freebsd"),
                (b"OpenBSD", b"x86_64") => Some("x86_64-unknown-openbsd"),
                (b"OpenBSD", b"i686") => Some("i686-unknown-openbsd"),
                (b"NetBSD", b"x86_64") => Some("x86_64-unknown-netbsd"),
                (b"NetBSD", b"i686") => Some("i686-unknown-netbsd"),
                (b"DragonFly", b"x86_64") => Some("x86_64-unknown-dragonfly"),
                (b"SunOS", b"i86pc") => Some("x86_64-unknown-illumos"),
                _ => None,
            };

            host_triple.map(TargetTriple::new)
        }

        inner()
    }
}

impl fmt::Display for TargetTriple {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
