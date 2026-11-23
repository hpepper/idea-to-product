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
- Advisory exceptions: record each exception in two mirrored lists.
  - Add `{ id = "RUSTSEC-...", reason = "..." }` to `ignore` in `deny.toml`.
  - Copy the same ID to `ignore` in `.cargo/audit.toml`, with the reason as a comment above it.
  - `check_advisory_ignores.sh` fails the build when the two lists differ.
- File ownership: the build output is owned by you with podman, including the podman-docker shim, and with rootful or rootless docker.
  - `build.sh` detects rootless docker from the `name=rootless` entry in `docker info`, and then runs without `--user`, because container root maps to you.
  - When the detection fails, `build.sh` keeps `--user` with your uid and gid.
- Registry cache: `.cargo-home/` holds the crate cache and is not committed.

CI in `.github/workflows/ci.yml` runs `tests/pipeline/test_build_pipeline.sh`, this build, a gitleaks secret scan, and the test suites of `gui_editor`, `sad_xml_sql`, `sad2md`, and `tui_editor`.
Before each test suite compiles, CI runs two checks for that crate: `cargo audit --deny warnings` and `cargo deny --locked check --deny warnings`.
All four crates share `deny.toml` and `.cargo/audit.toml` of `gui_editor`, so one pair of mirrored exception lists covers them.
