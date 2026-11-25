# Backlog

Open points after v0.1.0, none of them blocking.
Move an item to `CHANGELOG.md` when it is done, or to GitHub issues if the project moves its tracking there.

## To decide

- **CI on push and pull requests.** The workflow only runs manually or on `v*` tags, so unit tests and clippy never run automatically on `develop` or on pull requests. Candidate: a small workflow running `cargo test` and `cargo clippy` on Ubuntu.
- **Authenticode signing.** The executable is not signed, so SmartScreen or antivirus software may warn when it is downloaded. Needs a code-signing certificate and a key-handling procedure: an IOLAS decision.
- **Build provenance.** GitHub artifact attestations would let users verify that a binary comes from the workflow. Cheap, useful only if someone verifies.

## Improvements

- **cargo-about installation time.** The release workflow compiles cargo-about from source (about 5 minutes per run). Use a prebuilt binary or cache it.
- **MSRV in CI.** `rust-version = "1.88"` (set by libloading) was checked by hand. A CI job on 1.88 would catch regressions when dependencies change.
- **Line endings.** Only `*.sh` is pinned to LF in `.gitattributes`; other files follow each machine's `core.autocrlf`, hence the CRLF warnings on Windows. A repository-wide policy would remove them.
- **GitHub repository settings.** Branch protection on `main` and `develop`, repository description and topics: not reviewed.

## Not verified

- **Integration tests without the driver.** With `--features modify-system-aliases` and no NI-XNET, the tests are designed to fail with "NI-XNET driver required". Never run on a machine without NI-XNET.

## Deferred on purpose (YAGNI)

Add only when a real need appears.

- Deploy and undeploy to Real-Time targets (`nxdbDeploy`, `nxdbUndeploy`), e.g. a forced undeploy. VeriStand handles deployment today.
- JSON output for `list`. Text output (`alias<TAB>path`) is enough for scripts so far.
- Conversion of non-ASCII arguments to the ANSI code page. NI-XNET reports the error; not the tool's responsibility.
- Idempotent `remove` (success when the alias does not exist). Today it fails with NI-XNET status `0xBFF63150`.
