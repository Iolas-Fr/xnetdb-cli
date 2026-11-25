# CLAUDE.md

Context for Claude Code sessions on this repository. Human-oriented documentation lives in [README.md](README.md) and [docs/](docs/); this file adds working rules and the traps met so far.

## Project

`xnetdb-cli`: standalone Windows x64 command-line tool that adds, removes and lists NI-XNET database aliases (`add`, `remove`, `list`), for deployment scripts on VeriStand machines.
Rust, GNU toolchain, `nixnet.dll` loaded at run time. Owned by IOLAS, Apache-2.0, public at https://github.com/Iolas-Fr/xnetdb-cli.

Status: v0.1.0 released on 2026-11-25 (CI build, GitHub release, VeriStand acceptance test passed).
Open points: [docs/backlog.md](docs/backlog.md).

| Path | Content |
|---|---|
| `src/main.rs` | CLI (clap), exit codes, `--no-replace` / `--baud` logic |
| `src/nixnet.rs` | FFI wrapper over `nixnet.dll` (libloading), ANSI decoding of NI-XNET strings |
| `tests/nixnet_driver.rs` | Integration tests against the real driver, feature `modify-system-aliases` |
| `build.rs` | Windows version information (winresource) |
| `.cargo/config.toml` | MinGW-w64 linker, `+crt-static` |
| `.github/workflows/release.yml` | Cross-compilation on Ubuntu, release on `v*` tags |
| `.github/scripts/changelog-section.sh` | Extracts a CHANGELOG section as release notes |
| `about.toml`, `about.hbs` | cargo-about configuration for `THIRD-PARTY-LICENSES.txt` |
| `docs/development-environment.md` | Toolchain setup and pitfalls |
| `docs/nixnet-api.md` | NI-XNET API facts, observed behaviour, references |

## Working rules

- Everything in the repository is in English: code, comments, docs, commit messages. The user may write in French.
- Claude commits; the user pushes (with Fork). Never push, never create GitHub releases, issues or comments unless explicitly asked.
- Gitflow: `main` (releases), `develop`, `feature/*`, `release/*`, `hotfix/*`. Merge with `--no-ff`.
  Small documentation or configuration changes may be committed directly on `develop` when the user agrees; code goes through `feature/*`.
  Keep each branch's history about its own topic (rewrite unpushed history if needed, never pushed history).
- Commit messages end with the `Co-Authored-By` trailer.
- YAGNI: features are added when needed, not in advance (see the deferred list in the backlog).
- Concise answers; ask when a decision belongs to the user.

## Environment traps (this machine)

- A PowerShell session started before a tool was installed does not see the new `PATH`. Reload it first:
  `$env:Path = [Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [Environment]::GetEnvironmentVariable('Path','User')`
- The system `PATH` contains NI Linux RT cross-toolchains (`C:\build\...\nilrt...`) that shadow `gcc`, `ar` and `objdump`.
  The repository already works around it (linker in `.cargo/config.toml`, `windres`/`ar` lookup in `build.rs`). Use `C:\msys64\mingw64\bin\objdump.exe` explicitly.
- `cargo install` ignores `.cargo/config.toml`: set `$env:CC = 'x86_64-w64-mingw32-gcc'; $env:AR = 'x86_64-w64-mingw32-gcc-ar'` for tools that compile C.
- The integration tests **modify the machine's NI-XNET alias list** (temporary `xnetdb-cli-it-*` aliases, removed afterwards). Run them only when the user agrees, and compare `xnetdb-cli list` before and after.

## Commands

```powershell
cargo test                                           # unit tests
cargo test --features modify-system-aliases          # + driver integration tests (see above)
cargo clippy --all-targets --features modify-system-aliases
cargo build --release                                # target\release\xnetdb-cli.exe
cargo +1.88-x86_64-pc-windows-gnu build --target-dir target\msrv   # MSRV check
cargo about generate --locked --fail about.hbs -o THIRD-PARTY-LICENSES.txt
```

## Decisions (and why)

- GNU toolchain, MINGW64 (msvcrt) environment, no Visual Studio or Windows SDK: matches the Rust `windows-gnu` target; no Microsoft tooling on build machines.
- `nixnet.dll` loaded dynamically: no import library, clear exit code 3 when the driver is missing.
- Validation is left to NI-XNET, which returns explicit errors; no duplicate checks in the tool. Its behaviour is pinned by the integration tests.
- No conversion of non-ASCII input to the ANSI code page: not the tool's responsibility; NI-XNET reports the error.
- `Cargo.toml` is the single source of truth for the version. The tag only labels the release commit; CI checks they match. No client-side hook.
- `CHANGELOG.md` (Keep a Changelog, ISO dates `YYYY-MM-DD`) provides the release notes; CI refuses a tag without a dated section.
- Apache-2.0 with a `NOTICE` file to keep IOLAS attribution in redistributions; releases ship `LICENSE`, `NOTICE` and `THIRD-PARTY-LICENSES.txt`.
- Name `xnetdb-cli` (not `xnetdb`) to avoid targeting NI's product name directly.
- Integration tests gated by the feature `modify-system-aliases`, named after their side effect rather than their dependency on the driver.
