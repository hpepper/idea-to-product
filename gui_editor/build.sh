#!/usr/bin/env bash
# Build gui_editor inside the pinned tool image and run the supply-chain checks.
# The binary ends up in ./target/release/gui_editor, owned by you.
# The build fails on any unignored `cargo audit` or `cargo deny` finding (GEDT-AR-00025).
# The shellcheck gate of check_shellcheck.sh runs first (GEDT-AR-00055, GEDT-AR-00056).
# The tool image comes from container/Containerfile (GEDT-AR-00034);
# container/tool_image.sh builds it when its tag is missing and checks it before each use.
#
#   ./build.sh            release build
#   ./build.sh --debug    debug build
#   RUST_IMAGE=<image>:<tag>@sha256:<digest> ./build.sh   use another pinned base
set -euo pipefail

cd "$(dirname "$0")"

# shellcheck source=SCRIPTDIR/container/tool_image.sh
source container/tool_image.sh
# shellcheck source=SCRIPTDIR/check_shellcheck.sh
source check_shellcheck.sh

BASE_IMAGE="${RUST_IMAGE:-$PINNED_RUST_IMAGE}"
require_pinned_base_image "$BASE_IMAGE" || exit 1
select_engine || exit 1

PROFILE_FLAG="--release"
PROFILE_DIR="release"
# Unknown arguments fail, so a typo such as --relase does not silently build release.
case "$#:${1:-}" in
    0:) ;;
    1:--debug)
        PROFILE_FLAG=""
        PROFILE_DIR="debug"
        ;;
    *)
        echo "usage: $0 [--debug]" >&2
        exit 1
        ;;
esac

# Keep the crate registry inside the project so repeat builds don't re-download.
# .gitignore excludes the cache and target/ from commits.
# The container writes only to these two directories, so neither may be a symlink.
require_writable_mount_dir .cargo-home || exit 1
require_writable_mount_dir target || exit 1
require_no_cargo_overrides .cargo-home . || exit 1

select_user_args
ensure_tool_image || exit 1

# The shellcheck gate (GEDT-AR-00055, GEDT-AR-00056) runs before the build container,
# so a script finding stops the build before any dependency code runs.
run_shellcheck_gate || exit 1

# The container script reads its inputs from the environment,
# never from interpolated text.
# CARGO_TARGET_DIR overrides .cargo/config.toml,
# whose /tmp target dir is lost with the container.
# `--deny warnings` turns unmaintained, unsound, and yanked advisories into failures.
# check_advisory_ignores.sh first proves that both tools share one exception list.
# cargo deny reads deny-gui_editor.toml, the gui_editor policy;
# deny.toml is the shared policy of the other crates.
# Audit and deny run before the build,
# so a flagged dependency never runs its build script or proc macros on the /app mount.
# Neither check needs a build: audit reads Cargo.lock, and deny reads `cargo metadata`.
# Both tools keep their advisory databases in the container /tmp (R-T54-01):
# cargo-audit through --db, and cargo-deny through the db-path of the policy file.
# A database that an earlier container edited in .cargo-home therefore never applies.
# Both tools run by the absolute paths that check_tool_image checked,
# because `cargo audit` and `cargo deny` would follow an [alias] in a cargo config file.
# shellcheck disable=SC2016 # Expanded by bash inside the container, not here.
CONTAINER_SCRIPT='
set -euo pipefail
export ADVISORY_DB_DIR=/tmp/advisory-dbs
bash ./check_advisory_ignores.sh
/usr/local/cargo/bin/cargo-audit audit --db "$ADVISORY_DB_DIR/cargo-audit" --deny warnings
/usr/local/cargo/bin/cargo-deny --locked --config deny-gui_editor.toml check --deny warnings
cargo build --locked $PROFILE_FLAG
'

# The gui_editor source is mounted read-only,
# so no build script or proc macro can change a file that later runs on the host,
# such as this script, or a policy file such as deny-gui_editor.toml (D-T54-04).
# Only target/ and .cargo-home/ are mounted writable, on top of the read-only /app.
# The read-only parent keeps the container from replacing either mount point,
# or from creating any other path in /app.
# The binary lands in target/, owned by you.
# The container /tmp stays writable, holds the advisory databases,
# and is lost with the container.
# :Z relabels the mounts for SELinux (Fedora/RHEL); harmless elsewhere.
# "--" ends the engine options, so the image is never parsed as an option.
"$ENGINE" run --rm \
    "${USER_ARGS[@]}" \
    -v "$PWD":/app:ro,Z \
    -v "$PWD/target":/app/target:Z \
    -v "$PWD/.cargo-home":/app/.cargo-home:Z \
    -w /app \
    -e CARGO_HOME=/app/.cargo-home \
    -e CARGO_TARGET_DIR=/app/target \
    -e PROFILE_FLAG="$PROFILE_FLAG" \
    -- \
    "$TOOL_IMAGE" \
    bash -c "$CONTAINER_SCRIPT"

BIN="target/$PROFILE_DIR/gui_editor"
echo
echo "Built $BIN, run it with: ./$BIN"
