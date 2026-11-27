# GUI + Web editor

## Build

Run `./build.sh` for a release build, or `./build.sh --debug` for a debug build.
Any other argument fails.
The script needs podman or docker, and writes the binary to `target/release/gui_editor`.

# TODO all this shit looks like something that belongs in a dev doc, why the fuck would the user of this app need to know all these shit details.

- Pinned toolchain: the build runs in the tool image, built from the Rust image pinned by version and digest; see [Tool image](#tool-image).
  - `RUST_IMAGE` overrides the base image only in the form `<image>:<tag>@sha256:<digest>`.
- Locked dependencies: `cargo build --locked` uses the committed `Cargo.lock`.
- Shellcheck gate: `build.sh` stops before the supply-chain checks when a committed script fails the gate; see [Shellcheck gate](#shellcheck-gate).
- Supply-chain checks: `cargo audit` and `cargo deny` fail on any unignored finding.
  - Both checks run before `cargo build`, so a flagged dependency never runs its build script.
  - Both checks fetch a clean advisory database into the container `/tmp` on every run; see [Tool image](#tool-image).
- Advisory exceptions: record each exception in three mirrored lists.
  - Add `{ id = "RUSTSEC-...", reason = "..." }` to `ignore` in `deny.toml` and in `deny-gui_editor.toml`.
  - Copy the same ID to `ignore` in `.cargo/audit.toml`, with the reason as a comment above it.
  - `check_advisory_ignores.sh` fails the build when the lists differ.
- File ownership: the build output is owned by you with podman, including the podman-docker shim, and with rootful or rootless docker.
  - `build.sh` detects rootless docker from the `name=rootless` entry in `docker info`, and then runs without `--user`, because container root maps to you.
  - When the detection fails, `build.sh` keeps `--user` with your uid and gid.
- Registry cache: `.cargo-home/` holds the crate cache and is not committed.
- Writable paths: the container writes only to `target/` and `.cargo-home/`, and mounts the rest of `gui_editor` read-only.
  - A build script or proc macro cannot change `build.sh`, `container/tool_image.sh`, or a policy file, which run or apply on the host later.
  - `build.sh` stops when `target` or `.cargo-home` is a symlink, because the engine would mount the symlink target writable.
- System headers: none needed, because the windowing libraries are loaded at run time.

## Tool image

`build.sh` and CI run every cargo step in one local tool image, defined by `container/Containerfile`.
The tool image is a build and check environment, not a dev shell, and the GUI does not run in a container.

- Content: the official `docker.io/library/rust` image, pinned by version tag and digest, plus five tools.
  - `cargo-audit` 0.23.0 and `cargo-deny` 0.22.0, installed with `cargo install --locked --version`.
  - `clippy` and `rustfmt`, added with `rustup component add`, at the version of the pinned toolchain.
  - `shellcheck`, installed with `apt-get install` from the signature-checked Debian trixie archive of the base, at the version that the release provides, with no version pin.
- Tag: `localhost/gui_editor-tools:<hash>`, so a changed Containerfile or another base builds a new image.
  - `<hash>` is the sha256 over the Containerfile bytes, one NUL byte, and the checked base reference without a trailing newline.
  - The image is built locally when the tag is missing, and is never pushed to a registry.
- Content check before each use: the build stops when a check fails.
  - The digest in the `org.opencontainers.image.base.name` label must match the pinned base digest.
  - `cargo-audit --version`, `cargo-deny --version`, `cargo fmt --version`, and `cargo clippy --version` must report the pinned versions, without network.
  - `build.sh` and CI run `cargo-audit` and `cargo-deny` by the checked absolute paths in `/usr/local/cargo/bin`, so no cargo `[alias]` replaces them.
  - `dpkg-query --search /usr/bin/shellcheck` must name the `shellcheck` package, `dpkg --verify shellcheck` must print nothing, and `/usr/bin/shellcheck --version` must succeed.
  - The gate runs `shellcheck` by the checked absolute path `/usr/bin/shellcheck`.
- Base override: `RUST_IMAGE` with another pinned reference gets its own tag, so the first build with the override builds a new image.
  - An image whose label does not match the configured base digest still fails the label check.
  - Remove such an image with `docker image rm <tag>` or `podman image rm <tag>`, as the error message shows, and run the build again.
- Planted overrides: `build.sh` and `tool_image.sh run` stop when a mounted directory holds a file that would replace or weaken the checked tools.
  - In `.cargo-home/`: `bin`, `config`, `config.toml`, or `audit.toml`.
  - In the working directory, and for `run` also in the repository root: `.cargo/config`, `rust-toolchain`, or `rust-toolchain.toml`.
  - For `run`: a `.cargo/config.toml` in the repository root, which commits none.
  - For `run`: a `<dir>/.cargo/config.toml` that git does not track, or that is a symlink.
  - When git is missing, or the repository is not a git checkout, `run` refuses every `<dir>/.cargo/config.toml`.
  - The check also catches a symlink of one of these names, even one that dangles on the host.
  - Older versions of `build.sh` installed `cargo-audit` and `cargo-deny` in `.cargo-home/bin`; remove the directory once.

Run the checks through `build.sh`.
To run another cargo command in the tool image, use `container/tool_image.sh run <dir> <command>`, where `<dir>` is a top-level directory of the repository.
`build.sh` runs only the checks and `cargo build`, so run the tests through `tool_image.sh run`.
For example, `container/tool_image.sh run gui_editor cargo test --locked` runs the `gui_editor` tests.

- `container/tool_image.sh build` builds and checks the image, and prints its reference.
- `run` mounts the whole repository read-only, `<dir>` included, so a build script, proc macro, or test changes no file of the repository.
  - `<dir>` must not be a symlink, and `gui_editor/.cargo-home` must not be a symlink.
- `run` writes only to `gui_editor/.cargo-home/`, mounted as the crate cache, to `<dir>/test-output/`, and to the container `/tmp`.
  - `/tmp/target` holds the build output, and `/tmp` is lost with the container.
  - `<dir>/test-output/` is the git-ignored test output folder of the crate, kept on the host.
  - A test writes its files to `<crate>/test-output/`, `std::env::temp_dir()`, or `CARGO_TARGET_TMPDIR`.
  - A test that writes anywhere else in its crate directory fails with `Read-only file system`.
  - `run` creates `<dir>/test-output/` owned by you, and stops when `<dir>/test-output` is a symlink or a file.
  - `run .` mounts no test output folder, because the repository root is not a crate.
- Accepted risk: `.cargo-home/registry/src` stays writable, so a command in the container can change cached crate sources that later builds compile.
  - Remove `gui_editor/.cargo-home/` to download clean crate sources.
- Advisory databases: `cargo audit` and `cargo deny` fetch a clean copy into the container `/tmp/advisory-dbs` on every run, so no earlier container can edit the database that a later check reads.
  - `build.sh` sets `ADVISORY_DB_DIR=/tmp/advisory-dbs` in the container, and passes `--db "$ADVISORY_DB_DIR/cargo-audit"` to `cargo-audit`.
  - Both policy files take the `cargo-deny` database path from `ADVISORY_DB_DIR`, and `cargo-deny` fails when the variable is unset.
  - To run `cargo-deny` through `run`, put `env ADVISORY_DB_DIR=/tmp/advisory-dbs` before the command, as CI does.
  - To run `cargo-audit` through `run`, pass `--db /tmp/advisory-dbs/cargo-audit`, as CI does.
  - Older builds left `advisory-db` and `advisory-dbs` in `.cargo-home/`, which no check reads anymore, so remove them once.
- Accepted risk: the git check trusts your `PATH` and shell functions (O-1).
  - A fake `git` earlier on `PATH`, a `command` function loaded through `BASH_ENV`, or an exported `env` function could make an untracked `<dir>/.cargo/config.toml` pass.
  - Only you, the caller, control these, and code in the container cannot change them.
- Accepted risk: a later `cargo test` on the host follows symlinks that a container planted in `<crate>/test-output/` (O-2).
  - A `cargo test` on the host already runs the dependencies of the crate natively, so the symlinks give no access that the host run lacks.
- `run` checks whether git tracks `<dir>/.cargo/config.toml` with every `GIT_*` variable, the system configuration, and your user configuration ignored.
- `CONTAINER_ENGINE` selects podman or docker, and `RUST_IMAGE` selects another pinned base, as for `build.sh`.

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

## Shellcheck gate

`build.sh` runs `check_shellcheck.sh` after the tool image check and before any cargo step (GEDT-AR-00055, GEDT-AR-00056).
To run the gate alone, run `./check_shellcheck.sh`; `CONTAINER_ENGINE` and `RUST_IMAGE` work as for `build.sh`.

- Scope: every file that git tracks under `gui_editor`, `sad2md`, `sad_xml_sql`, and `tui_editor` with the extension `.sh` or `.bash`, in any letter case, or with a `#!` first line.
  - A new script needs no list change; the gate checks it once git tracks it.
  - The repository-root scripts and `aws2xml` stay unchecked.
  - A tracked file that is missing or a symlink in the working tree stops the gate, because the gate cannot classify it.
- Findings: the tool image runs `shellcheck --norc` over all scripts in one run, and any finding stops the build.
  - No option changes the checks or the severity, and `--norc` keeps any `.shellcheckrc` from changing them.
  - One run lets `shellcheck` follow `source container/tool_image.sh` without `-x`, so a sourced script must be tracked too.
  - The run mounts the repository read-only, with no network, and writes nothing to the host.
- Disable directives: a `# shellcheck disable=` directive needs a reason comment after it on the same line, such as `# shellcheck disable=SC2016 # Reason text.`.
  - A reason on the line above does not count.
  - Fix a finding where possible, and hide it only with a real reason.

## CI

CI in `.github/workflows/ci.yml` runs `tests/pipeline/test_build_pipeline.sh`, this build, a gitleaks secret scan, and the test suites of `gui_editor`, `sad_xml_sql`, `sad2md`, and `tui_editor`.
The build step runs the [Shellcheck gate](#shellcheck-gate) first, so CI fails on any script finding.
Before each test suite compiles, CI runs two checks for that crate: `cargo audit --deny warnings` and `cargo deny --locked check --deny warnings`.
`cargo deny` uses `deny-gui_editor.toml` for `gui_editor` and `deny.toml` for the other crates.
All four crates share `.cargo/audit.toml` of `gui_editor`.
CI builds the tool image from the pinned base in each job that needs the image, checks the image, and runs `cargo audit`, `cargo deny`, and `cargo test` in the image.
CI installs no tool with `cargo install`, pushes no image, and needs no permission beyond `contents: read`.

## Committed languages

# TODO what is all this giberish doing here? who will read it and why? an who is the audience and what intelligence level do they need to have to actual understand this word vomit?????????
The `gui_editor` test suite fails when a file that git tracks anywhere in the repository is program code or a script in a language other than Rust or Bash (GEDT-AR-00032, GEDT-AR-00033).
The check in `tests/test_build_pipeline_config.rs` reads the file list and contents from the git index, so untracked and ignored files are out of scope.

- The interpreter line of a script must equal `#!/bin/bash`, `#!/usr/bin/bash`, or `#!/usr/bin/env bash`, with no further character before the line end (GEDT-AR-00054).
  - Any other line fails, such as `#!./bash`, `#!/opt/x/bash`, `#!/usr/bin/env /bin/bash`, `#! /bin/bash`, a trailing space, or a CRLF line end.
  - Flags, `env` options, `-S`, and assignments fail, such as `#!/bin/bash -e` or `#!/usr/bin/env -S bash`.
  - Set bash options in the script body, such as `set -euo pipefail`.
- An executable file needs one of the three interpreter lines.
- A `.sh` or `.bash` file needs one of the three interpreter lines.
- Every other file needs the `.rs` extension or a listed data or configuration format, such as TOML, YAML, XML, Markdown, Mermaid, `Containerfile`, `Dockerfile`, `.gitattributes`, `.gitmodules`, or `.editorconfig`.
- A `.mk`, `.tmpl`, or `.pp` file fails, and so does a `Makefile` without one of the three interpreter lines, because make recipes, templates, and manifests hold code or scripts.
- A submodule fails even with a committed `.gitmodules`, because the check cannot read the language of another repository.
- Code generated by the Rust toolchain, such as `wasm-bindgen` JavaScript glue, is never committed (GEDT-AR-00053).
  - The build generates the code, and `.gitignore` keeps the output out of git, so the check has no exception for generated code.
- To commit a new data or configuration format, add its extension or file name to the lists in the test, and justify the entry in review.
