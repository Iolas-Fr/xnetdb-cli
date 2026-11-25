# xnetdb-cli

`xnetdb-cli` is a standalone Windows command-line tool that creates, updates, lists and removes NI-XNET database aliases (FIBEX, DBC, LDF, ...) without the NI-XNET Database Editor.
Intended for deployment scripts, e.g. to make a FIBEX database available to VeriStand on a new machine.

## Installation

Download `xnetdb-cli.exe` from the [latest release](https://github.com/Iolas-Fr/xnetdb-cli/releases/latest) and check it against the published checksum:

```powershell
(Get-FileHash .\xnetdb-cli.exe -Algorithm SHA256).Hash
Get-Content .\xnetdb-cli.exe.sha256
```

The executable needs no installation: place it next to the deployment script or in a directory on the `PATH`.
When redistributing it, keep `LICENSE`, `NOTICE` and `THIRD-PARTY-LICENSES.txt` with it.

Example in a PowerShell deployment script:

```powershell
.\xnetdb-cli.exe add LoopTest_8CAN "$PSScriptRoot\LoopTest_8CAN.xml"
if ($LASTEXITCODE -ne 0) { throw "NI-XNET alias registration failed ($LASTEXITCODE)" }
```

## Usage

```
xnetdb-cli add <alias> <file> [--baud <rate>] [--no-replace]
xnetdb-cli remove <alias>
xnetdb-cli list
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

Aliases survive NI-XNET upgrades but are lost when the driver is uninstalled: run `xnetdb-cli add` again after a clean reinstall.

API details, status codes and references: [docs/nixnet-api.md](docs/nixnet-api.md).

## Runtime requirements

- Windows, 64-bit.
- NI-XNET driver installed (provides `nixnet.dll`, loaded at run time). Nothing else.

## Development

See [docs/development-environment.md](docs/development-environment.md) for the toolchain setup (Rust GNU + MSYS2, no Visual Studio).

```powershell
cargo test
cargo build --release
```

### Tests

- **Unit tests** cover the tool's own logic. They run anywhere, with or without NI-XNET:

  ```powershell
  cargo test
  ```

- **NI-XNET integration tests** ([tests/nixnet_driver.rs](tests/nixnet_driver.rs)) run the built `xnetdb-cli` against the real driver.
  They **modify this machine's NI-XNET alias list**, which every NI-XNET application on the machine shares, VeriStand included.
  They are therefore only compiled when the `modify-system-aliases` feature is requested explicitly:

  ```powershell
  cargo test --features modify-system-aliases
  ```

  Without NI-XNET, do not enable the feature: the tests would fail, by design, rather than be skipped silently.
  Avoid running them on a production machine.

Why the integration tests exist: `xnetdb-cli` deliberately leaves validation to NI-XNET, so its correctness rests on the driver behaviour listed under [NI-XNET behaviour](#ni-xnet-behaviour), which was observed by hand rather than documented by NI.
The integration tests replay those observations, so they serve as evidence for the README and detect any behaviour change after a driver update.
Run them after changing the code that calls NI-XNET, and after upgrading NI-XNET.

They use temporary `xnetdb-cli-it-*` aliases pointing at copies of the FIBEX example installed with NI-XNET, and remove them afterwards.
Set `XNETDB_CLI_TEST_DB` to another valid database file if the example is not installed.
Assertions rely on exit codes and NI-XNET status codes, not on the localised messages.

These tests do not replace the acceptance test before a release: register a real database with `xnetdb-cli add`, check it under *File → Manage Aliases* in the NI-XNET Database Editor, and check that a VeriStand deployment finds it.

### Continuous integration and releases

[.github/workflows/release.yml](.github/workflows/release.yml) cross-compiles `xnetdb-cli.exe` on an `ubuntu-latest` runner (MinGW-w64, target `x86_64-pc-windows-gnu`), runs the unit tests and checks that the executable depends on no MinGW runtime DLL.

- **Manual run** (*Actions → Release → Run workflow*, any branch): build only. The executable, its SHA-256 checksum, `LICENSE`, `NOTICE` and `THIRD-PARTY-LICENSES.txt` are attached to the run as the `xnetdb-cli` artifact.
- **Version tag** `v*`: build, then publish a GitHub release with the same files.
  The release notes are the version's section of [CHANGELOG.md](CHANGELOG.md).
  The workflow fails before building if the tag does not match the version in `Cargo.toml`, or if `CHANGELOG.md` has no dated, non-empty section for it.

`THIRD-PARTY-LICENSES.txt` lists the licenses of the crates compiled into the executable, as their licenses require for binary redistribution.
It is generated by [cargo-about](https://github.com/EmbarkStudios/cargo-about) from [about.toml](about.toml) and [about.hbs](about.hbs), and fails on a dependency license not listed in `about.toml`.
To generate it locally (installation: see [docs/development-environment.md](docs/development-environment.md)):

```powershell
cargo about generate --locked --fail about.hbs -o THIRD-PARTY-LICENSES.txt
```

To release, following gitflow:

1. Create the release branch from `develop` (`release/x.y.z`; Fork names it `release/vx.y.z` with a `v` tag prefix, which is fine).
2. On that branch, set `version = "x.y.z"` in `Cargo.toml` if needed, and replace `Unreleased` with the release date (`YYYY-MM-DD`) in the version's heading of [CHANGELOG.md](CHANGELOG.md).
3. Push the branch and run the workflow manually on it; check the artifact (5 files).
4. Run the acceptance test with that executable (see [Tests](#tests)).
5. Finish the release: merge into `main` and back into `develop`, tag `main` with `vx.y.z`, and push the branches and the tag.
   The tag must start with `v`: in Fork, set the gitflow version tag prefix to `v`. Without Fork:

   ```powershell
   git tag -a vx.y.z -m "xnetdb-cli x.y.z"
   git push origin main develop vx.y.z
   ```

Branching follows gitflow: `main` (releases), `develop` (integration), `feature/*`, `release/*`, `hotfix/*`.
Commit messages and documentation are written in English.

Open points and deferred features: [docs/backlog.md](docs/backlog.md).

## License

Copyright 2026 IOLAS.

Licensed under the [Apache License, Version 2.0](LICENSE).
Redistributions, modified or not, must keep the [NOTICE](NOTICE) file and its attribution to IOLAS, as required by section 4 of the license.

NI-XNET, VeriStand and LabVIEW are trademarks of National Instruments / Emerson.
This project is not affiliated with or endorsed by National Instruments or Emerson.
