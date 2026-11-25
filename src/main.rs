//! xnetdb: manage NI-XNET database aliases from the command line.

mod nixnet;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};

use nixnet::NiXnet;

/// Exit code when an NI-XNET operation fails.
const EXIT_FAILURE: u8 = 1;
/// Exit code when nixnet.dll cannot be loaded.
const EXIT_NO_DRIVER: u8 = 3;

/// Manage NI-XNET database aliases (FIBEX, DBC, LDF, ...).
#[derive(Parser)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create an alias, or replace the file of an existing one.
    Add {
        /// Alias name (no commas).
        alias: String,
        /// Database file; resolved to an absolute path.
        file: PathBuf,
        /// Default baud rate, used only for DBC and NCD files.
        #[arg(long)]
        baud: Option<u64>,
        /// Fail if the alias already exists instead of replacing it.
        #[arg(long)]
        no_replace: bool,
    },
    /// Remove an alias. The database file is not deleted.
    Remove {
        /// Alias name.
        alias: String,
    },
    /// List the aliases registered on this machine.
    List,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let nx = match NiXnet::load() {
        Ok(nx) => nx,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::from(EXIT_NO_DRIVER);
        }
    };

    match run(&nx, cli.command) {
        Ok(()) => ExitCode::SUCCESS,
        Err(msg) => {
            eprintln!("error: {msg}");
            ExitCode::from(EXIT_FAILURE)
        }
    }
}

fn run(nx: &NiXnet, command: Command) -> Result<(), String> {
    match command {
        Command::Add {
            alias,
            file,
            baud,
            no_replace,
        } => {
            if no_replace && alias_exists(nx, &alias)? {
                return Err(format!("alias '{alias}' already exists"));
            }
            if baud.is_some() && !uses_baud_rate(&file) {
                eprintln!("warning: --baud is ignored for this file type (DBC and NCD only)");
            }
            let path = std::path::absolute(&file)
                .map_err(|e| format!("cannot resolve '{}': {e}", file.display()))?;
            let path = path.to_string_lossy();
            nx.add_alias(&alias, &path, baud.unwrap_or(0))
                .map_err(|e| e.to_string())?;
            println!("{alias} -> {path}");
        }
        Command::Remove { alias } => {
            nx.remove_alias(&alias).map_err(|e| e.to_string())?;
            println!("removed {alias}");
        }
        Command::List => {
            for a in nx.list_aliases().map_err(|e| e.to_string())? {
                println!("{}\t{}", a.name, a.path);
            }
        }
    }
    Ok(())
}

fn alias_exists(nx: &NiXnet, alias: &str) -> Result<bool, String> {
    let list = nx.list_aliases().map_err(|e| e.to_string())?;
    Ok(list.iter().any(|a| a.name.eq_ignore_ascii_case(alias)))
}

/// DBC and NCD files carry no baud rate; the default baud rate applies to them only.
fn uses_baud_rate(file: &Path) -> bool {
    file.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("dbc") || e.eq_ignore_ascii_case("ncd"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baud_rate_only_for_dbc_and_ncd() {
        assert!(uses_baud_rate(Path::new("a.dbc")));
        assert!(uses_baud_rate(Path::new(r"C:\x\a.NCD")));
        assert!(!uses_baud_rate(Path::new("a.xml")));
        assert!(!uses_baud_rate(Path::new("a.ldf")));
        assert!(!uses_baud_rate(Path::new("a")));
    }

    #[test]
    fn cli_is_consistent() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}
