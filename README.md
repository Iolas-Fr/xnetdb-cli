# xnet-database-cli

Standalone Windows command-line tool that creates, updates, lists and removes NI-XNET database aliases (FIBEX, DBC, LDF) without the NI-XNET Database Editor.
Intended for deployment scripts, e.g. to make a FIBEX database available to VeriStand on a new machine.

> Status: under development.

## Runtime requirements

- Windows, 64-bit.
- NI-XNET driver installed (provides `nixnet.dll`). Nothing else.

## Development

See [docs/development-environment.md](docs/development-environment.md) for the toolchain setup (Rust GNU + MSYS2, no Visual Studio).

Branching follows gitflow: `main` (releases), `develop` (integration), `feature/*`, `release/*`, `hotfix/*`.
Commit messages and documentation are written in English.
