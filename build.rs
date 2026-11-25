//! Embeds the Windows version information (file properties > Details) into
//! the executable, using the MinGW-w64 `windres` and `ar`.

use std::env;
use std::path::PathBuf;

/// The linker set in .cargo/config.toml; `windres` and `ar` are taken from
/// the same directory.
const MINGW_GCC: &str = "x86_64-w64-mingw32-gcc";

fn main() {
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let mut res = winresource::WindowsResource::new();
    // ProductName, FileDescription (shown as the application name by
    // Windows), FileVersion and ProductVersion default to Cargo.toml values.
    res.set("CompanyName", "IOLAS")
        .set("LegalCopyright", "Copyright 2026 IOLAS. Apache License 2.0.")
        .set("OriginalFilename", concat!(env!("CARGO_PKG_NAME"), ".exe"));

    // On a Windows host, winresource calls unprefixed `windres` and `ar`,
    // which may resolve to another toolchain (e.g. NI Linux RT
    // cross-compilers on the system PATH). Use the tools that sit next to
    // the MinGW-w64 linker instead, unless WINDRES / AR are set explicitly.
    // Cross-compiling from Linux uses the x86_64-w64-mingw32- prefixed tools.
    if env::var("HOST").is_ok_and(|h| h.contains("windows"))
        && let Some(dir) = find_in_path(&format!("{MINGW_GCC}.exe"))
    {
        if env::var_os("WINDRES").is_none() {
            res.set_windres_path(&dir.join("windres.exe").to_string_lossy());
        }
        if env::var_os("AR").is_none() {
            res.set_ar_path(&dir.join("ar.exe").to_string_lossy());
        }
    }

    res.compile().expect("failed to embed Windows version information");
}

/// Directory of the first PATH entry that contains `file`.
fn find_in_path(file: &str) -> Option<PathBuf> {
    env::split_paths(&env::var_os("PATH")?).find(|dir| dir.join(file).is_file())
}
