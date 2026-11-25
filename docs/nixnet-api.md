# NI-XNET API notes

Background on the NI-XNET functions used by `xnetdb-cli`, what was verified, and where it comes from.

## Context

The NI-XNET Database Editor stores, on each machine, a mapping between a name (alias) and a database file (FIBEX, DBC, LDF, ...).
VeriStand deploys databases by alias, so a database copied to a new machine is unusable until its alias is created.
NI provides no command-line tool for this, only APIs: LabVIEW (File Mgt palette, *Managing Local* example), C (`nxdb...` functions) and Python (`nixnet` package, `add_alias`).
`xnetdb-cli` fills that gap with a standalone executable.

A LabVIEW tool built around *XNET Database Add Alias* was considered and rejected: it needs a built executable plus the LabVIEW Run-Time Engine on every machine.

## Functions used

Declared in `nixnet.h`, installed with the driver in `C:\Program Files (x86)\National Instruments\Shared\ExternalCompilerSupport\C\include\`.
Exported by `C:\Windows\System32\nixnet.dll` (64-bit), loaded at run time.

| Function | Signature | Use |
|---|---|---|
| `nxdbAddAlias64` | `nxStatus_t (const char *DatabaseAlias, const char *DatabaseFilepath, u64 DefaultBaudRate)` | `add` |
| `nxdbRemoveAlias` | `nxStatus_t (const char *DatabaseAlias)` | `remove` |
| `nxdbGetDatabaseListSizes` | `nxStatus_t (const char *IPAddress, u32 *SizeofAliasBuffer, u32 *SizeofFilepathBuffer)` | `list` |
| `nxdbGetDatabaseList` | `nxStatus_t (const char *IPAddress, u32 SizeofAliasBuffer, char *AliasBuffer, u32 SizeofFilepathBuffer, char *FilepathBuffer, u32 *NumberOfDatabases)` | `list` |
| `nxStatusToString` | `void (nxStatus_t Status, u32 SizeofString, char *StatusDescription)` | error messages |

- Calling convention: `_NXFUNC` is `__cdecl`, i.e. `extern "C"` on x86_64.
- `nxStatus_t` is `i32`: `0` success, negative values errors, positive values warnings.
- `IPAddress` empty (`""`) means the local machine.
- `nxdbGetDatabaseList` returns comma-separated lists, which is why NI-XNET forbids commas in aliases and paths.
- `DefaultBaudRate` only matters for formats without a baud rate (DBC, NCD); `0` is passed otherwise.
- Strings are `char *` in the system ANSI code page. Output strings (messages, paths) are decoded with `MultiByteToWideChar(CP_ACP)`; input strings are passed as is (no conversion, by decision).
- Messages from `nxStatusToString` are localised (French on a French Windows).

Other database functions exist (`nxdbDeploy`, `nxdbUndeploy` for Real-Time targets) but are deliberately not exposed: VeriStand handles deployment.

## Observed behaviour

Verified on NI-XNET with a French Windows 10, and pinned by [tests/nixnet_driver.rs](../tests/nixnet_driver.rs):

| Case | Result | Status code |
|---|---|---|
| Add an existing alias | Replaces its file path (idempotent with the same path) | success |
| Alias names | Case-insensitive; `add` with another case renames the alias | success |
| File already assigned to another alias | Rejected | `0xBFF63103` |
| Missing file, relative path to a missing file, non-database file | Rejected | `0xBFF63081` |
| Comma in the alias | Rejected | `0xBFF6322C` |
| Comma in the file path | Rejected | `0xBFF6322D` |
| Remove an unknown alias | Rejected | `0xBFF63150` |

Persistence (NI knowledge base): an alias lasts until it is removed, survives LabVIEW restarts and NI-XNET upgrades, and is lost when NI-XNET is uninstalled.
Removing an alias never deletes the database file.

## References

- NI knowledge base, programmatic import and export of NI-XNET database aliases (updated 2023-11-01): https://knowledge.ni.com/KnowledgeArticleDetails?id=kA03q000000YOUXCA4&l=en-US
- NI-XNET C reference: `nixnet.h` and the NI-XNET manual, database functions chapter.
- Reference implementation in Python: https://github.com/ni/nixnet-python, modules `nixnet/_cfuncs.py` and `nixnet/system/_databases.py`.
