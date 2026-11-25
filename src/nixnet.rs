//! Thin wrapper around the NI-XNET database functions of `nixnet.dll`.
//!
//! The DLL is loaded at run time so that a missing driver produces a clear
//! error instead of a loader failure. Signatures come from `nixnet.h`
//! (`_NXFUNC` is `__cdecl`, i.e. the C ABI on x86_64).

use std::ffi::{CString, c_char};
use std::fmt;

use libloading::Library;

type NxStatus = i32;

type AddAlias64Fn = unsafe extern "C" fn(*const c_char, *const c_char, u64) -> NxStatus;
type RemoveAliasFn = unsafe extern "C" fn(*const c_char) -> NxStatus;
type GetDatabaseListSizesFn = unsafe extern "C" fn(*const c_char, *mut u32, *mut u32) -> NxStatus;
type GetDatabaseListFn = unsafe extern "C" fn(
    *const c_char,
    u32,
    *mut c_char,
    u32,
    *mut c_char,
    *mut u32,
) -> NxStatus;
type StatusToStringFn = unsafe extern "C" fn(NxStatus, u32, *mut c_char);

const DLL_NAME: &str = "nixnet.dll";

/// Errors returned by this module.
#[derive(Debug)]
pub enum Error {
    /// `nixnet.dll` or one of its functions could not be loaded.
    Driver(String),
    /// An NI-XNET function returned an error status.
    Status { code: i32, message: String },
    /// A string passed to NI-XNET contains an interior NUL byte.
    InvalidString(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Driver(msg) => write!(f, "NI-XNET driver not available: {msg}"),
            Error::Status { code, message } => {
                write!(f, "NI-XNET error {code:#010X}: {message}", code = *code as u32)
            }
            Error::InvalidString(s) => write!(f, "invalid string (contains NUL): {s:?}"),
        }
    }
}

impl std::error::Error for Error {}

/// A database alias and the file it points to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Alias {
    pub name: String,
    pub path: String,
}

/// Loaded NI-XNET driver.
pub struct NiXnet {
    add_alias64: AddAlias64Fn,
    remove_alias: RemoveAliasFn,
    get_database_list_sizes: GetDatabaseListSizesFn,
    get_database_list: GetDatabaseListFn,
    status_to_string: StatusToStringFn,
    // Keeps the function pointers above valid.
    _lib: Library,
}

impl NiXnet {
    /// Loads `nixnet.dll` and resolves the database functions.
    pub fn load() -> Result<Self, Error> {
        // SAFETY: nixnet.dll is a regular NI driver DLL without unusual
        // initialisation requirements.
        let lib = unsafe { Library::new(DLL_NAME) }
            .map_err(|e| Error::Driver(format!("cannot load {DLL_NAME}: {e}")))?;

        // SAFETY: the function types match the declarations in nixnet.h.
        unsafe {
            Ok(Self {
                add_alias64: symbol(&lib, "nxdbAddAlias64")?,
                remove_alias: symbol(&lib, "nxdbRemoveAlias")?,
                get_database_list_sizes: symbol(&lib, "nxdbGetDatabaseListSizes")?,
                get_database_list: symbol(&lib, "nxdbGetDatabaseList")?,
                status_to_string: symbol(&lib, "nxStatusToString")?,
                _lib: lib,
            })
        }
    }

    /// Creates the alias, or replaces its file path if it already exists.
    pub fn add_alias(&self, name: &str, path: &str, default_baud_rate: u64) -> Result<(), Error> {
        let name = c_string(name)?;
        let path = c_string(path)?;
        // SAFETY: both pointers are valid NUL-terminated strings for the call.
        let status = unsafe { (self.add_alias64)(name.as_ptr(), path.as_ptr(), default_baud_rate) };
        self.check(status)
    }

    /// Removes the alias. The database file itself is left untouched.
    pub fn remove_alias(&self, name: &str) -> Result<(), Error> {
        let name = c_string(name)?;
        // SAFETY: valid NUL-terminated string.
        let status = unsafe { (self.remove_alias)(name.as_ptr()) };
        self.check(status)
    }

    /// Lists the aliases registered on the local machine.
    pub fn list_aliases(&self) -> Result<Vec<Alias>, Error> {
        let local = c_string("")?;
        let mut alias_size = 0u32;
        let mut path_size = 0u32;
        // SAFETY: valid string and out-pointers.
        let status = unsafe {
            (self.get_database_list_sizes)(local.as_ptr(), &mut alias_size, &mut path_size)
        };
        self.check(status)?;

        let mut aliases = vec![0u8; alias_size as usize + 1];
        let mut paths = vec![0u8; path_size as usize + 1];
        let mut count = 0u32;
        // SAFETY: buffers are at least as large as the sizes passed.
        let status = unsafe {
            (self.get_database_list)(
                local.as_ptr(),
                aliases.len() as u32,
                aliases.as_mut_ptr().cast(),
                paths.len() as u32,
                paths.as_mut_ptr().cast(),
                &mut count,
            )
        };
        self.check(status)?;

        Ok(parse_database_list(
            &buffer_to_string(&aliases),
            &buffer_to_string(&paths),
            count as usize,
        ))
    }

    fn check(&self, status: NxStatus) -> Result<(), Error> {
        // Negative values are errors; positive values are warnings.
        if status >= 0 {
            return Ok(());
        }
        let mut buf = vec![0u8; 2048];
        // SAFETY: buffer size passed matches the allocation.
        unsafe { (self.status_to_string)(status, buf.len() as u32, buf.as_mut_ptr().cast()) };
        Err(Error::Status {
            code: status,
            message: buffer_to_string(&buf).trim().to_owned(),
        })
    }
}

/// Splits the comma-separated lists returned by `nxdbGetDatabaseList`.
fn parse_database_list(aliases: &str, paths: &str, count: usize) -> Vec<Alias> {
    if count == 0 || aliases.is_empty() {
        return Vec::new();
    }
    aliases
        .split(',')
        .zip(paths.split(','))
        .take(count)
        .map(|(name, path)| Alias {
            name: name.to_owned(),
            path: path.to_owned(),
        })
        .collect()
}

unsafe fn symbol<T: Copy>(lib: &Library, name: &str) -> Result<T, Error> {
    // SAFETY: the caller guarantees that T matches the exported function.
    unsafe { lib.get::<T>(name.as_bytes()) }
        .map(|s| *s)
        .map_err(|e| Error::Driver(format!("{name} not found in {DLL_NAME}: {e}")))
}

fn c_string(s: &str) -> Result<CString, Error> {
    CString::new(s).map_err(|_| Error::InvalidString(s.to_owned()))
}

/// Decodes a NUL-terminated buffer filled by NI-XNET.
///
/// NI-XNET returns strings (localised messages, file paths) in the system
/// ANSI code page, so they are decoded with `MultiByteToWideChar(CP_ACP)`.
fn buffer_to_string(buf: &[u8]) -> String {
    let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    ansi_to_string(&buf[..end])
}

#[cfg(windows)]
fn ansi_to_string(bytes: &[u8]) -> String {
    const CP_ACP: u32 = 0;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn MultiByteToWideChar(
            code_page: u32,
            flags: u32,
            multi_byte: *const u8,
            multi_byte_len: i32,
            wide: *mut u16,
            wide_len: i32,
        ) -> i32;
    }

    if bytes.is_empty() {
        return String::new();
    }
    let len = bytes.len() as i32;
    // SAFETY: input pointer/length describe `bytes`; first call only sizes.
    let wide_len =
        unsafe { MultiByteToWideChar(CP_ACP, 0, bytes.as_ptr(), len, std::ptr::null_mut(), 0) };
    if wide_len <= 0 {
        return String::from_utf8_lossy(bytes).into_owned();
    }
    let mut wide = vec![0u16; wide_len as usize];
    // SAFETY: output buffer holds `wide_len` elements.
    unsafe { MultiByteToWideChar(CP_ACP, 0, bytes.as_ptr(), len, wide.as_mut_ptr(), wide_len) };
    String::from_utf16_lossy(&wide)
}

#[cfg(not(windows))]
fn ansi_to_string(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_list() {
        assert!(parse_database_list("", "", 0).is_empty());
    }

    #[test]
    fn parse_list_pairs_aliases_and_paths() {
        let list = parse_database_list("A,B c", r"C:\a.xml,C:\b c.dbc", 2);
        assert_eq!(
            list,
            vec![
                Alias { name: "A".into(), path: r"C:\a.xml".into() },
                Alias { name: "B c".into(), path: r"C:\b c.dbc".into() },
            ]
        );
    }

    #[test]
    fn buffer_stops_at_nul() {
        assert_eq!(buffer_to_string(b"abc\0def"), "abc");
    }
}
