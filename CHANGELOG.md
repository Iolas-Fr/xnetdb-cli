# Changelog

All notable changes to this project are documented in this file.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project uses [Semantic Versioning](https://semver.org/).

## [0.1.0] - 2026-11-25

First release.

### Added

- `add` command: creates an NI-XNET database alias, or replaces the file of an existing one. Options `--no-replace` and `--baud` (DBC and NCD only).
- `remove` command: removes an alias, keeping the database file.
- `list` command: prints the aliases registered on the machine.
- Standalone Windows x64 executable; `nixnet.dll` is loaded at run time, with a dedicated exit code when the driver is missing.
- Windows file properties: company, product, version and copyright.
- Releases ship `LICENSE`, `NOTICE`, `THIRD-PARTY-LICENSES.txt` and a SHA-256 checksum with the executable.

[0.1.0]: https://github.com/Iolas-Fr/xnetdb-cli/releases/tag/v0.1.0
