//! Integration tests against the real NI-XNET driver.
//!
//! # Why these tests exist
//!
//! `xnetdb` deliberately leaves input validation to NI-XNET, which reports
//! explicit errors. The tool's correctness therefore depends on driver
//! behaviour that is documented nowhere but observed by hand (see the
//! "NI-XNET behaviour" section of the README):
//!
//! - `add` is idempotent and replaces the file of an existing alias;
//! - alias names are case-insensitive;
//! - one database file can be assigned to one alias only;
//! - missing files and commas are rejected;
//! - removing an unknown alias is an error.
//!
//! These tests turn those observations into executable evidence, and flag any
//! change after an NI-XNET driver update.
//!
//! # Running
//!
//! They need the NI-XNET driver and modify the machine's alias list, so they
//! are `#[ignore]`d and must be requested explicitly:
//!
//! ```text
//! cargo test -- --ignored
//! ```
//!
//! Each test uses its own `xnetdb-it-*` aliases pointing at temporary copies
//! of a valid database, and removes them afterwards, even on failure.
//! The source database defaults to the FIBEX example installed with NI-XNET;
//! set `XNETDB_TEST_DB` to use another valid database file.
//!
//! NI-XNET messages are localised, so assertions use the exit code and the
//! hexadecimal status code, never the message text.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::{Mutex, MutexGuard};
use std::{env, fs};

const DEFAULT_DB: &str =
    r"C:\Users\Public\Documents\National Instruments\NI-XNET\Examples\NIXNET_example.xml";

/// NI-XNET status codes asserted below.
const ERR_CANNOT_OPEN_DATABASE: &str = "0xBFF63081";
const ERR_FILE_ALREADY_ASSIGNED: &str = "0xBFF63103";
const ERR_ALIAS_NOT_DEFINED: &str = "0xBFF63150";
const ERR_INVALID_CHAR_IN_ALIAS: &str = "0xBFF6322C";
const ERR_INVALID_CHAR_IN_PATH: &str = "0xBFF6322D";

/// The alias list is global to the machine: run the tests one at a time.
static DRIVER: Mutex<()> = Mutex::new(());

/// Test environment: serialises access to the driver, provides database
/// copies, and removes the test aliases and files when dropped.
struct Fixture {
    dir: PathBuf,
    aliases: Vec<String>,
    _lock: MutexGuard<'static, ()>,
}

impl Fixture {
    fn new(name: &str) -> Self {
        let lock = DRIVER.lock().unwrap_or_else(|e| e.into_inner());
        let dir = env::temp_dir().join(format!("xnetdb-it-{}-{name}", std::process::id()));
        fs::create_dir_all(&dir).expect("create temp dir");
        Self {
            dir,
            aliases: Vec::new(),
            _lock: lock,
        }
    }

    /// Returns a fresh copy of the source database, i.e. a distinct file.
    fn database(&self, file_name: &str) -> PathBuf {
        let source = env::var_os("XNETDB_TEST_DB")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(DEFAULT_DB));
        let target = self.dir.join(file_name);
        fs::copy(&source, &target).unwrap_or_else(|e| {
            panic!(
                "cannot copy test database {}: {e} (set XNETDB_TEST_DB)",
                source.display()
            )
        });
        target
    }

    /// Returns a test alias name, registered for cleanup.
    fn alias(&mut self, suffix: &str) -> String {
        let alias = format!("xnetdb-it-{suffix}");
        self.aliases.push(alias.clone());
        // Leftover from an interrupted run.
        let _ = xnetdb(&["remove", &alias]);
        alias
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        for alias in &self.aliases {
            let _ = xnetdb(&["remove", alias]);
        }
        let _ = fs::remove_dir_all(&self.dir);
    }
}

fn xnetdb(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_xnetdb"))
        .args(args)
        .output()
        .expect("run xnetdb")
}

fn path_str(path: &Path) -> &str {
    path.to_str().expect("UTF-8 path")
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[track_caller]
fn assert_success(output: &Output) {
    assert!(output.status.success(), "xnetdb failed: {}", stderr(output));
}

#[track_caller]
fn assert_ni_error(output: &Output, status: &str) {
    assert_eq!(output.status.code(), Some(1), "stderr: {}", stderr(output));
    assert!(
        stderr(output).contains(status),
        "expected NI-XNET status {status}, got: {}",
        stderr(output)
    );
}

/// Path registered for `alias`, matched case-insensitively like NI-XNET.
fn registered_path(alias: &str) -> Option<String> {
    let output = xnetdb(&["list"]);
    assert_success(&output);
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| line.split_once('\t'))
        .find(|(name, _)| name.eq_ignore_ascii_case(alias))
        .map(|(_, path)| path.to_owned())
}

fn registered_name(alias: &str) -> Option<String> {
    let output = xnetdb(&["list"]);
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| line.split_once('\t'))
        .find(|(name, _)| name.eq_ignore_ascii_case(alias))
        .map(|(name, _)| name.to_owned())
}

#[test]
#[ignore = "needs the NI-XNET driver; modifies the alias list"]
fn add_registers_alias_and_remove_deletes_it() {
    let mut fx = Fixture::new("add");
    let alias = fx.alias("add");
    let db = fx.database("db.xml");

    assert_success(&xnetdb(&["add", &alias, path_str(&db)]));
    assert_eq!(registered_path(&alias).as_deref(), Some(path_str(&db)));

    assert_success(&xnetdb(&["remove", &alias]));
    assert_eq!(registered_path(&alias), None);
    assert!(db.exists(), "remove must not delete the database file");
}

#[test]
#[ignore = "needs the NI-XNET driver; modifies the alias list"]
fn add_is_idempotent() {
    let mut fx = Fixture::new("idempotent");
    let alias = fx.alias("idempotent");
    let db = fx.database("db.xml");

    assert_success(&xnetdb(&["add", &alias, path_str(&db)]));
    assert_success(&xnetdb(&["add", &alias, path_str(&db)]));
    assert_eq!(registered_path(&alias).as_deref(), Some(path_str(&db)));
}

#[test]
#[ignore = "needs the NI-XNET driver; modifies the alias list"]
fn add_replaces_file_of_existing_alias() {
    let mut fx = Fixture::new("replace");
    let alias = fx.alias("replace");
    let one = fx.database("one.xml");
    let two = fx.database("two.xml");

    assert_success(&xnetdb(&["add", &alias, path_str(&one)]));
    assert_success(&xnetdb(&["add", &alias, path_str(&two)]));
    assert_eq!(registered_path(&alias).as_deref(), Some(path_str(&two)));
}

#[test]
#[ignore = "needs the NI-XNET driver; modifies the alias list"]
fn no_replace_refuses_existing_alias() {
    let mut fx = Fixture::new("no-replace");
    let alias = fx.alias("no-replace");
    let one = fx.database("one.xml");
    let two = fx.database("two.xml");

    assert_success(&xnetdb(&["add", &alias, path_str(&one)]));
    let output = xnetdb(&["add", &alias, path_str(&two), "--no-replace"]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(registered_path(&alias).as_deref(), Some(path_str(&one)));
}

#[test]
#[ignore = "needs the NI-XNET driver; modifies the alias list"]
fn no_replace_is_case_insensitive() {
    let mut fx = Fixture::new("no-replace-case");
    let alias = fx.alias("no-replace-case");
    let one = fx.database("one.xml");
    let two = fx.database("two.xml");

    assert_success(&xnetdb(&["add", &alias, path_str(&one)]));
    let upper = alias.to_uppercase();
    let output = xnetdb(&["add", &upper, path_str(&two), "--no-replace"]);
    assert_eq!(output.status.code(), Some(1));
}

#[test]
#[ignore = "needs the NI-XNET driver; modifies the alias list"]
fn alias_names_are_case_insensitive() {
    let mut fx = Fixture::new("case");
    let alias = fx.alias("case");
    let one = fx.database("one.xml");
    let two = fx.database("two.xml");
    let upper = alias.to_uppercase();

    assert_success(&xnetdb(&["add", &alias, path_str(&one)]));
    assert_success(&xnetdb(&["add", &upper, path_str(&two)]));

    // One alias only, renamed to the new case and pointing at the new file.
    assert_eq!(registered_name(&alias).as_deref(), Some(upper.as_str()));
    assert_eq!(registered_path(&alias).as_deref(), Some(path_str(&two)));
}

#[test]
#[ignore = "needs the NI-XNET driver; modifies the alias list"]
fn file_can_be_assigned_to_one_alias_only() {
    let mut fx = Fixture::new("one-file");
    let first = fx.alias("one-file-a");
    let second = fx.alias("one-file-b");
    let db = fx.database("db.xml");

    assert_success(&xnetdb(&["add", &first, path_str(&db)]));
    assert_ni_error(
        &xnetdb(&["add", &second, path_str(&db)]),
        ERR_FILE_ALREADY_ASSIGNED,
    );
    assert_eq!(registered_path(&second), None);
}

#[test]
#[ignore = "needs the NI-XNET driver; modifies the alias list"]
fn missing_file_is_rejected() {
    let mut fx = Fixture::new("missing");
    let alias = fx.alias("missing");
    let missing = fx.dir.join("missing.xml");

    assert_ni_error(
        &xnetdb(&["add", &alias, path_str(&missing)]),
        ERR_CANNOT_OPEN_DATABASE,
    );
    assert_eq!(registered_path(&alias), None);
}

#[test]
#[ignore = "needs the NI-XNET driver; modifies the alias list"]
fn non_database_file_is_rejected() {
    let mut fx = Fixture::new("invalid");
    let alias = fx.alias("invalid");
    let file = fx.dir.join("not-a-database.xml");
    fs::write(&file, "not a database").expect("write file");

    assert_ni_error(
        &xnetdb(&["add", &alias, path_str(&file)]),
        ERR_CANNOT_OPEN_DATABASE,
    );
}

#[test]
#[ignore = "needs the NI-XNET driver; modifies the alias list"]
fn comma_in_alias_is_rejected() {
    let mut fx = Fixture::new("comma-alias");
    let db = fx.database("db.xml");
    let alias = fx.alias("comma,alias");

    assert_ni_error(
        &xnetdb(&["add", &alias, path_str(&db)]),
        ERR_INVALID_CHAR_IN_ALIAS,
    );
}

#[test]
#[ignore = "needs the NI-XNET driver; modifies the alias list"]
fn comma_in_path_is_rejected() {
    let mut fx = Fixture::new("comma-path");
    let alias = fx.alias("comma-path");
    let db = fx.database("a,b.xml");

    assert_ni_error(
        &xnetdb(&["add", &alias, path_str(&db)]),
        ERR_INVALID_CHAR_IN_PATH,
    );
}

#[test]
#[ignore = "needs the NI-XNET driver; modifies the alias list"]
fn removing_unknown_alias_is_an_error() {
    let mut fx = Fixture::new("unknown");
    let alias = fx.alias("unknown");

    assert_ni_error(&xnetdb(&["remove", &alias]), ERR_ALIAS_NOT_DEFINED);
}
