use std::ops::Deref;

use crate::utils::Process;

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
#[cfg(all(not(windows), not(target_env = "musl")))]
const TRIPLE_LOONGARCH64_UNKNOWN_LINUX: &str = "loongarch64-unknown-linux-gnu";
#[cfg(all(not(windows), target_env = "musl"))]
const TRIPLE_LOONGARCH64_UNKNOWN_LINUX: &str = "loongarch64-unknown-linux-musl";

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

#[derive(Debug)]
pub struct TargetTriple(String);

impl TargetTriple {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub(crate) fn from_host(process: &Process) -> Option<Self> {
        #[cfg(windows)]
        fn inner() -> Option<TargetTriple> {
            use std::mem;

            /// Get the host architecture using `IsWow64Process2`. This function
            /// produces the most accurate results (supports detecting aarch64), but
            /// it is only available on Windows 10 1511+, so we use `GetProcAddress`
            /// to maintain backward compatibility with older Windows versions.
            fn arch_primary() -> Option<&'static str> {
                use windows_sys::core::s;
                use windows_sys::Win32::Foundation::{BOOL, HANDLE};
                use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};
                use windows_sys::Win32::System::Threading::GetCurrentProcess;

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
                    let module = GetModuleHandleA(s!("kernel32.dll"));
                    if module == 0 {
                        return None;
                    }
                    mem::transmute(GetProcAddress(module, s!("IsWow64Process2"))?)
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
                use windows_sys::Win32::System::SystemInformation::GetNativeSystemInfo;

                const PROCESSOR_ARCHITECTURE_AMD64: u16 = 9;
                const PROCESSOR_ARCHITECTURE_INTEL: u16 = 0;

                let mut sys_info;
                unsafe {
                    sys_info = mem::zeroed();
                    GetNativeSystemInfo(&mut sys_info);
                }

                match unsafe { sys_info.Anonymous.Anonymous }.wProcessorArchitecture {
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
                (b"Linux", b"x86_64") => Some(TRIPLE_X86_64_UNKNOWN_LINUX),
                (b"Linux", b"i686") => Some("i686-unknown-linux-gnu"),
                (b"Linux", b"mips") => Some(TRIPLE_MIPS_UNKNOWN_LINUX_GNU),
                (b"Linux", b"mips64") => Some(TRIPLE_MIPS64_UNKNOWN_LINUX_GNUABI64),
                (b"Linux", b"arm") => Some("arm-unknown-linux-gnueabi"),
                (b"Linux", b"armv7l") => Some("armv7-unknown-linux-gnueabihf"),
                (b"Linux", b"armv8l") => Some("armv7-unknown-linux-gnueabihf"),
                (b"Linux", b"aarch64") => Some(if is_32bit_userspace() {
                    "armv7-unknown-linux-gnueabihf"
                } else {
                    TRIPLE_AARCH64_UNKNOWN_LINUX
                }),
                (b"Linux", b"loongarch64") => Some(TRIPLE_LOONGARCH64_UNKNOWN_LINUX),
                (b"Darwin", b"x86_64") => Some("x86_64-apple-darwin"),
                (b"Darwin", b"i686") => Some("i686-apple-darwin"),
                _ => None,
            };

            host_triple.map(TargetTriple::new)
        }

        if let Ok(triple) = process.var("XUANWU_OVERRIDE_HOST_TRPLE") {
            Some(Self(triple))
        } else {
            inner()
        }
    }
}

impl Deref for TargetTriple {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Check if /bin/sh is a 32-bit binary. If it doesn't exist, fall back to
/// checking if _we_ are a 32-bit binary.
/// rustup-init.sh also relies on checking /bin/sh for bitness.
#[cfg(not(windows))]
fn is_32bit_userspace() -> bool {
    use std::fs;
    use std::io::{self, Read};

    // inner function is to simplify error handling.
    fn inner() -> io::Result<bool> {
        let mut f = fs::File::open("/bin/sh")?;
        let mut buf = [0; 5];
        f.read_exact(&mut buf)?;

        // ELF files start out "\x7fELF", and the following byte is
        //   0x01 for 32-bit and
        //   0x02 for 64-bit.
        Ok(&buf == b"\x7fELF\x01")
    }

    inner().unwrap_or(cfg!(target_pointer_width = "32"))
}