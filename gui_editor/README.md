# GUI + Web editor

## Build

Run `./build.sh` for a release build, or `./build.sh --debug` for a debug build.
Any other argument fails.
The script needs podman or docker, and writes the binary to `target/release/gui_editor`.

- Pinned toolchain: the Rust image is pinned by version and digest.
  - `RUST_IMAGE` overrides the image only in the form `<image>:<tag>@sha256:<digest>`.
- Locked dependencies: `cargo build --locked` uses the committed `Cargo.lock`.
- Supply-chain checks: `cargo audit` and `cargo deny` fail on any unignored finding.
  - Both checks run before `cargo build`, so a flagged dependency never runs its build script.
- Advisory exceptions: record each exception in three mirrored lists.
  - Add `{ id = "RUSTSEC-...", reason = "..." }` to `ignore` in `deny.toml` and in `deny-gui_editor.toml`.
  - Copy the same ID to `ignore` in `.cargo/audit.toml`, with the reason as a comment above it.
  - `check_advisory_ignores.sh` fails the build when the lists differ.
- File ownership: the build output is owned by you with podman, including the podman-docker shim, and with rootful or rootless docker.
  - `build.sh` detects rootless docker from the `name=rootless` entry in `docker info`, and then runs without `--user`, because container root maps to you.
  - When the detection fails, `build.sh` keeps `--user` with your uid and gid.
- Registry cache: `.cargo-home/` holds the crate cache and is not committed.
- System headers: none needed, because the windowing libraries are loaded at run time.

## Run

Run `./target/release/gui_editor` from a desktop session on Fedora.
The window, titled `gui_editor`, shows an Open button and an empty component list.
Open shows the desktop file dialog and displays the chosen path.
The editor does not read the chosen file yet.

The windowing library picks the display server from the environment.

- Wayland: `WAYLAND_DISPLAY` is set, as in a GNOME Wayland session.
- X11: `WAYLAND_DISPLAY` is unset and `DISPLAY` is set, as in an X11 session.
  - To test X11 in a Wayland session, run `env -u WAYLAND_DISPLAY ./target/release/gui_editor`, which uses XWayland.
  - `xwininfo -root -tree | grep gui_editor` lists the window only when the editor runs on X11.

## Runtime dependencies on Fedora

The editor loads these libraries at run time, and Fedora Workstation installs them by default.

- Display: `libwayland-client`, `libxkbcommon`, `libX11`, `libxcb`, and `libxkbcommon-x11`.
- OpenGL: `libEGL` and `libGL` from Mesa.
- File dialog: `xdg-desktop-portal` with a desktop backend, such as `xdg-desktop-portal-gnome`, reached through `libdbus-1` on the session bus.
  - When the portal fails, rfd runs the `zenity` program found first on `PATH`, as a direct process, with no shell and no network.

The editor makes no network connections.

## Dependency policy

`cargo deny` reads one of two policy files, which differ only in the `gui_editor` entries.

- `deny-gui_editor.toml`: `gui_editor` only.
  - `licenses.exceptions` allows OFL-1.1 and Ubuntu-font-1.0 for the fonts in `epaint_default_fonts` only.
  - `bans.skip` lists the Linux duplicates of the eframe 0.37 windowing stack, each with a reason, until T-43 removes them.
- `deny.toml`: `sad2md`, `sad_xml_sql`, and `tui_editor`, with empty `licenses.exceptions` and `bans.skip`.
  - cargo-deny reports an exception or a skip entry that a crate does not need, so the `gui_editor` entries would fail these crates.
- Both files check the `x86_64-unknown-linux-gnu` graph only.
- `tests/test_build_pipeline_config.rs` fails when the files differ outside the `gui_editor` entries, so edit both files together.

## CI

CI in `.github/workflows/ci.yml` runs `tests/pipeline/test_build_pipeline.sh`, this build, a gitleaks secret scan, and the test suites of `gui_editor`, `sad_xml_sql`, `sad2md`, and `tui_editor`.
Before each test suite compiles, CI runs two checks for that crate: `cargo audit --deny warnings` and `cargo deny --locked check --deny warnings`.
`cargo deny` uses `deny-gui_editor.toml` for `gui_editor` and `deny.toml` for the other crates.
All four crates share `.cargo/audit.toml` of `gui_editor`.
