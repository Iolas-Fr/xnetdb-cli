# xnet-database-cli

`xnetdb` is a standalone Windows command-line tool that creates, updates, lists and removes NI-XNET database aliases (FIBEX, DBC, LDF, ...) without the NI-XNET Database Editor.
Intended for deployment scripts, e.g. to make a FIBEX database available to VeriStand on a new machine.

## Usage

```
xnetdb add <alias> <file> [--baud <rate>] [--no-replace]
xnetdb remove <alias>
xnetdb list
```

- `add` creates the alias, or replaces the file of an existing alias. Running it again with the same arguments is harmless, so it can be called on every deployment.
  The file path is made absolute before registration.
  `--no-replace` fails instead of replacing an existing alias.
  `--baud` sets the default baud rate; it only applies to DBC and NCD files (other formats carry their own baud rate) and is ignored with a warning otherwise.
- `remove` deletes the alias only; the database file is kept.
- `list` prints one `alias<TAB>path` line per alias.

Exit codes: `0` success, `1` operation failed (message on stderr), `2` invalid command line, `3` NI-XNET driver not found.

### NI-XNET behaviour

Validation is left to NI-XNET, which reports explicit errors. Observed behaviour:

- The file must exist and be a valid database.
- One database file can be assigned to one alias only.
- Alias names are case-insensitive: `add MyDb` replaces an existing `mydb` (and renames it).
- Commas are rejected in alias names and file paths.
- Paths with characters outside the system ANSI code page are not supported by NI-XNET.
- Removing an alias that does not exist is an error.

Aliases survive NI-XNET upgrades but are lost when the driver is uninstalled: run `xnetdb add` again after a clean reinstall.

## Runtime requirements

- Windows, 64-bit.
- NI-XNET driver installed (provides `nixnet.dll`, loaded at run time). Nothing else.

## Development

See [docs/development-environment.md](docs/development-environment.md) for the toolchain setup (Rust GNU + MSYS2, no Visual Studio).

```powershell
cargo test
cargo build --release
```

Branching follows gitflow: `main` (releases), `develop` (integration), `feature/*`, `release/*`, `hotfix/*`.
Commit messages and documentation are written in English.
