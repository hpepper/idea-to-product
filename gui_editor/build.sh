#!/usr/bin/env bash
# Build gui_editor inside a pinned Rust container and run the supply-chain checks.
# The binary ends up in ./target/release/gui_editor, owned by you.
# The build fails on any unignored `cargo audit` or `cargo deny` finding (GEDT-AR-00025).
#
#   ./build.sh            release build
#   ./build.sh --debug    debug build
#   RUST_IMAGE=<image>:<tag>@sha256:<digest> ./build.sh   use another pinned toolchain
set -euo pipefail

cd "$(dirname "$0")"

# Pinned by version and digest so that a retagged upstream image cannot change the build.
# Update the tag and the digest together, and keep .github/workflows/ci.yml in sync.
PINNED_RUST_IMAGE="docker.io/library/rust:1.100.0-trixie@sha256:223cc32ddca8f3ee90c436dbc94b6b5f052ea7f9653c5201e9612f08ad7d21b8"
IMAGE="${RUST_IMAGE:-$PINNED_RUST_IMAGE}"
# The whole value must be an image reference with a version tag and a digest.
# Anchoring both ends rejects values that the engine would parse as an option,
# and the tag keeps the toolchain version visible next to the digest.
# Registry: lowercase host, optional port. Repository: lowercase path components.
IMAGE_NAME_RE='[a-z0-9]+([._-][a-z0-9]+)*(:[0-9]+)?(/[a-z0-9]+([._-]+[a-z0-9]+)*)*'
IMAGE_TAG_RE='[A-Za-z0-9_][A-Za-z0-9._-]{0,127}'
PINNED_IMAGE_RE="^${IMAGE_NAME_RE}:${IMAGE_TAG_RE}@sha256:[0-9a-f]{64}\$"
if [[ ! "$IMAGE" =~ $PINNED_IMAGE_RE ]]; then
    echo "error: RUST_IMAGE must be pinned by digest, with a version tag (<image>:<tag>@sha256:<digest>)" >&2
    exit 1
fi

# Tool versions are pinned and installed with --locked for reproducible checks.
CARGO_AUDIT_VERSION="0.23.0"
CARGO_DENY_VERSION="0.22.0"

ENGINE="${CONTAINER_ENGINE:-$(command -v podman || command -v docker || true)}"
[[ -n "$ENGINE" ]] || { echo "error: neither podman nor docker found" >&2; exit 1; }

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
# .gitignore excludes the cache from commits.
mkdir -p .cargo-home

# Run as the calling user so build output isn't root-owned.
# Rootless podman needs keep-id for that; rootful docker takes the uid directly.
# `--version` also detects podman behind a `docker` name, such as the podman-docker shim.
# Rootless docker maps container root to you and --user to a subordinate uid,
# so it gets no user flag.
# The engine reports rootless mode as the "name=rootless" entry of its security options.
# Detection is fail-safe: a failing `info` or any other output keeps the --user flag.
# Engine output is only compared as text, never evaluated.
ROOTLESS_DOCKER_RE='^\[(.*,)?"name=rootless(,[^"]*)?"(,.*)?\]$'
is_rootless_docker() {
    local security_options
    security_options="$("$ENGINE" info --format '{{json .SecurityOptions}}' 2>/dev/null)" \
        || return 1
    [[ "$security_options" =~ $ROOTLESS_DOCKER_RE ]]
}
ENGINE_VERSION="$("$ENGINE" --version 2>/dev/null || true)"
if [[ "$(basename "$ENGINE")" == podman || "$ENGINE_VERSION" == *podman* ]]; then
    USER_ARGS=(--userns=keep-id)
elif is_rootless_docker; then
    USER_ARGS=()
else
    USER_ARGS=(--user "$(id -u):$(id -g)")
fi

# The container script reads its inputs from the environment,
# never from interpolated text.
# CARGO_TARGET_DIR overrides .cargo/config.toml,
# whose /tmp target dir is lost with the container.
# `cargo install` is a no-op when the pinned version is already in .cargo-home.
# `--deny warnings` turns unmaintained, unsound, and yanked advisories into failures.
# check_advisory_ignores.sh first proves that both tools share one exception list.
# Audit and deny run before the build,
# so a flagged dependency never runs its build script or proc macros on the /app mount.
# Neither check needs a build: audit reads Cargo.lock, and deny reads `cargo metadata`.
# shellcheck disable=SC2016 # Expanded by bash inside the container, not here.
CONTAINER_SCRIPT='
set -euo pipefail
bash ./check_advisory_ignores.sh
cargo install --locked cargo-audit --version "$CARGO_AUDIT_VERSION"
cargo install --locked cargo-deny --version "$CARGO_DENY_VERSION"
cargo audit --deny warnings
cargo deny --locked check --deny warnings
cargo build --locked $PROFILE_FLAG
'

# :Z relabels the mount for SELinux (Fedora/RHEL); harmless elsewhere.
# "--" ends the engine options, so the image is never parsed as an option.
"$ENGINE" run --rm \
    "${USER_ARGS[@]}" \
    -v "$PWD":/app:Z \
    -w /app \
    -e CARGO_HOME=/app/.cargo-home \
    -e CARGO_TARGET_DIR=/app/target \
    -e CARGO_AUDIT_VERSION="$CARGO_AUDIT_VERSION" \
    -e CARGO_DENY_VERSION="$CARGO_DENY_VERSION" \
    -e PROFILE_FLAG="$PROFILE_FLAG" \
    -- \
    "$IMAGE" \
    bash -c "$CONTAINER_SCRIPT"

BIN="target/$PROFILE_DIR/gui_editor"
echo
echo "Built $BIN, run it with: ./$BIN"
