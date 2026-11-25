# Backlog

Open points after v0.1.0, none of them blocking.
Move an item to `CHANGELOG.md` when it is done, or to GitHub issues if the project moves its tracking there.

## Planned

### Linux support

NI-XNET is also available on desktop Linux. Provide the same command-line tool, with the same commands, for the distributions NI supports.

Supported distributions (NI Linux Device Drivers compatibility pages; check the latest one before starting):

- 2025 Q4: Ubuntu 22.04 and 24.04, RHEL 8.10 and 9.6, openSUSE Leap 15.5 and 15.6.
- 2026 Q2: Ubuntu 22.04 and 24.04, RHEL 9.6 and 10.0, openSUSE Leap 15.6 and 16.0.
- Fedora is not listed; RHEL is NI's Red Hat family target. 64-bit only.

Installation, per NI's readme: add NI's repository package for the distribution (ni.com/info, code `LinuxRepository`), install `ni-xnet` (optionally `libnixnet-devel`), then run `dkms autoinstall` before rebooting.

Steps:

1. **Explore on a virtual machine** (Ubuntu 24.04 first): install the driver and find the shared library and header the packages provide (`dpkg -L ni-xnet`, `ldconfig -p | grep -i nixnet`). Expected `libnixnet.so`, not verified.
   Check that the DKMS kernel modules load; with Secure Boot enabled they must be signed (MOK enrolment).
2. **Check the database functions without hardware**: whether `nxdbAddAlias64`, `nxdbRemoveAlias` and `nxdbGetDatabaseList` work in a VM with no XNET interface, and where Linux stores the aliases.
3. **Confirm the API details on Linux** from the installed `nixnet.h`: calling convention (`_NXFUNC` is empty outside Windows, so the C ABI), string encoding (presumably UTF-8, which the non-Windows branch of `src/nixnet.rs` already assumes), status codes.
4. **Adapt the code**: library name per platform in `src/nixnet.rs`; default test database path in `tests/nixnet_driver.rs` (Windows path today, `XNETDB_CLI_TEST_DB` as override). `build.rs` already skips non-Windows targets.
5. **Re-run the integration tests on the VM** to pin the Linux behaviour (case sensitivity, one alias per file, error codes may differ), and document differences in `docs/nixnet-api.md`.
6. **Release**: build `x86_64-unknown-linux-gnu` natively on the Ubuntu runner and attach a `.tar.gz` to the release. Decide the minimum glibc (build on the oldest supported Ubuntu) or a static musl build.

References:

- [NI Linux Device Drivers 2025 Q4 compatibility](https://www.ni.com/en/support/documentation/compatibility/25/ni-linux-device-drivers-2025-q4-compatibility.html)
- [NI Linux Device Drivers 2026 Q2 compatibility](https://www.ni.com/en/support/documentation/compatibility/26/ni-linux-device-drivers-2026-q2-compatibility.html)
- [NI-XNET requirements](https://www.ni.com/docs/en-US/bundle/ni-xnet/page/requirements.html)
- [NI-XNET 2022 Q3 for Linux readme](https://www.ni.com/pdf/manuals/ni-xnet-linux-2022-q3.html) (installation steps)

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
