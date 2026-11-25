# Development environment

The tool is built with the Rust **GNU** toolchain (`x86_64-pc-windows-gnu`) and MinGW-w64 from MSYS2.
Visual Studio, the MSVC toolchain and the Windows SDK are **not** used and must not be required by any dependency.

## Requirements

| Component | Version / package | Purpose |
|---|---|---|
| Git for Windows | latest | Version control (gitflow: `main`, `develop`, `feature/*`) |
| MSYS2 | latest, installed in `C:\msys64` | Provides MinGW-w64 |
| MinGW-w64 (MINGW64 env) | `mingw-w64-x86_64-toolchain` | Linker (`x86_64-w64-mingw32-gcc`), `objdump`, `dlltool` |
| rustup / Rust stable | host `x86_64-pc-windows-gnu` | Compiler and Cargo |
| NI-XNET driver | any recent | Provides `C:\Windows\System32\nixnet.dll` (64-bit), needed at run time only |

Stay 64-bit throughout: the `nixnet.dll` in `System32` is 64-bit.

The MINGW64 environment (msvcrt) is used rather than UCRT64, because the Rust `windows-gnu` target links against msvcrt.

## Installation

Run in PowerShell (not as administrator unless winget asks for elevation).

1. Git and MSYS2:

   ```powershell
   winget install --id Git.Git -e
   winget install --id MSYS2.MSYS2 -e
   ```

2. MinGW-w64 toolchain (run the update twice if the first run closes the shell after updating the core packages):

   ```powershell
   C:\msys64\usr\bin\bash.exe -lc "pacman -Syu --noconfirm"
   C:\msys64\usr\bin\bash.exe -lc "pacman -S --needed --noconfirm mingw-w64-x86_64-toolchain"
   ```

   Optional, only for the import-library fallback (`gendef`):

   ```powershell
   C:\msys64\usr\bin\bash.exe -lc "pacman -S --needed --noconfirm mingw-w64-x86_64-tools-git"
   ```

3. Add `C:\msys64\mingw64\bin` to the user `PATH`:

   ```powershell
   [Environment]::SetEnvironmentVariable('Path', 'C:\msys64\mingw64\bin;' + [Environment]::GetEnvironmentVariable('Path', 'User'), 'User')
   ```

4. rustup with the GNU host as default (never the MSVC variant):

   ```powershell
   Invoke-WebRequest https://win.rustup.rs/x86_64 -OutFile "$env:TEMP\rustup-init.exe"
   & "$env:TEMP\rustup-init.exe" -y --default-host x86_64-pc-windows-gnu --default-toolchain stable
   ```

   On an existing rustup installation:

   ```powershell
   rustup toolchain install stable-x86_64-pc-windows-gnu
   rustup default stable-x86_64-pc-windows-gnu
   ```

5. Open a **new** terminal so the `PATH` changes apply.

## Verification

```powershell
git --version
rustup show
x86_64-w64-mingw32-gcc --version
```

`rustup show` must report `stable-x86_64-pc-windows-gnu` as the default toolchain.

## Known pitfall: other `gcc` on the PATH

Machines with the NI Linux Real-Time cross-compilers have their `gcc` directories (`C:\build\...\nilrt...`) in the system `PATH`.
The system `PATH` comes before the user `PATH`, so a plain `gcc` resolves to an ARM or Linux cross-compiler and linking fails.

The repository avoids this: [`.cargo/config.toml`](../.cargo/config.toml) sets the linker explicitly to `x86_64-w64-mingw32-gcc`, which only MinGW-w64 provides.
Do not remove that setting.

## Standalone binary

The release binary must depend on no MinGW DLL (`libgcc_s_seh-1.dll`, `libwinpthread-1.dll`).
[`.cargo/config.toml`](../.cargo/config.toml) enables `+crt-static` for that purpose.
Check the actual imports after a build:

```powershell
C:\msys64\mingw64\bin\objdump.exe -p target\release\<binary>.exe | Select-String 'DLL Name'
```

Use the full path: the NI cross-toolchains also put an `objdump` on the system `PATH`.

Only Windows system DLLs are acceptable (`KERNEL32.dll`, `msvcrt.dll`, `ntdll.dll`, `USERENV.dll`, `WS2_32.dll`, `bcryptprimitives.dll`, `api-ms-win-*` for a Rust std binary).
`nixnet.dll` must **not** appear: it is loaded at run time with `libloading`, so a missing driver yields a clear error message instead of a loader failure.
