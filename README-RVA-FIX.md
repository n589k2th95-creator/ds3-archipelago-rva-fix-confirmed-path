# DS3 Archipelago 4.0.2 — task-runner RVA initialization patch

This is the supplied source tree with a targeted change in
`crates/ds3-archipelago/src/game.rs`.

The failing UI is `shared::ErrorDisplay`, which receives the error returned by
`Game::run_recurring_task`. The old DS3 implementation used
`SprjTaskImp::wait_for_instance()`. The patch changes that to the explicit
system-initialization followed by `SprjTaskImp::instance()` sequence used by
the later upstream DS3 client. See `RVA-FIX-NOTES.md` for the source trace and
validation details.

A manual GitHub Actions workflow builds the Windows x64 `ds3_archipelago.dll`
for the Windows game process running under WineCX. It does not build a macOS
`.dylib`.

The patch has not been compiled or tested in this environment. Run the included
workflow before installing the resulting DLL.
