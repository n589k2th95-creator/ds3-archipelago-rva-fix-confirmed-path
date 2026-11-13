# Experimental DS3 Archipelago RVA initialization fix

## Trace from the supplied source

The popup with the **Show full error** checkbox is rendered by
`crates/shared/src/error_display.rs` (`ErrorDisplay::render_error`). It is not
the Windows `DllMain` panic message box.

The error reaches that UI through this path:

1. `shared::initialize()` in `crates/shared/src/lib.rs` creates the DS3 core.
2. It calls `G::run_recurring_task(...)`.
3. If that method returns an error, `shared::initialize()` passes the error to
   `ErrorDisplay::new(core, blocker)`.
4. The original DS3 implementation of `run_recurring_task` in
   `crates/ds3-archipelago/src/game.rs` used
   `SprjTaskImp::wait_for_instance(Duration::MAX)?`.

That is the failing initialization path. The `SprjTaskImp::wait_for_instance`
API is documented as able to return `SystemInitError::InvalidRva` when the
`HINSTANCE` or `SprjTaskImp` RVA does not resolve inside the executable. This
matches the popup text, `Could not translate RVA to VA`.

## Candidate fix

This patch replaces the combined `SprjTaskImp::wait_for_instance` call with
three explicit steps:

1. Wait for DS3's system initialization / DLRF reflection data with
   `wait_for_system_init(&Program::current(), Duration::MAX)`.
2. Resolve `SprjTaskImp::instance()` only after system initialization.
3. Register the Archipelago frame task.

This is not an invented startup sequence: the later upstream DS3 client
(`ds3-v4.1.0-alpha.3`) uses the same explicit `wait_for_system_init` followed
by `SprjTaskImp::instance()` sequence instead of `wait_for_instance()`.

The patch adds log messages and `anyhow::Context` at the two resolution stages,
so a remaining failure will say which step did not complete.

## Scope

- Keeps the supplied 4.0.2 source and its existing `Cargo.lock` / dependency
  versions. No unrelated dependency upgrade is included.
- Changes only the recurring-task initialization code plus a build workflow and
  these notes.
- Does not modify the user's game files, save, ME3 installation, Bridgeplay
  settings, or Steam launch options.
- Builds a Windows x64 PE DLL, not a macOS dylib: DS3 is a Windows x64 process
  running inside WineCX.

## Build

The workflow `.github/workflows/build-ds3-rva-fix.yml` uses Windows CI to build
the locked dependency set for `x86_64-pc-windows-msvc`.

1. Put this source tree in a new GitHub repository or a branch of your fork.
2. Open **Actions**.
3. Select **Build DS3 task-runner initialization fix**.
4. Select **Run workflow**.
5. Download artifact `ds3_archipelago-task-runner-init-fix`.

## Test

First test the resulting DLL with the user's DLL-only profile. Keep the
original DLL backed up. If the dialog disappears, test the full Archipelago
profile and verify the in-game overlay and multiworld connection.

## Validation status

The source change is grounded in the supplied call path and later upstream
initialization sequence. This execution environment does not have `cargo` or
`rustc`, so I could not compile or run the patched DLL here. The included GitHub
Actions workflow performs the Windows x64 build; the resulting DLL remains
experimental until it is tested on DS3 1.15.2.0 Worldwide under Bridgeplay
WineCX.
