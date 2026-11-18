#!/usr/bin/env bash
# Build amber-dos inside the official Rust container.
# The binary ends up in ./target/release/amber-dos, owned by you.
#
#   ./build.sh            release build
#   ./build.sh --debug    debug build
#   RUST_IMAGE=rust:1.85 ./build.sh   pin a toolchain version
set -euo pipefail

#cd "$(dirname "$0")"
#pwd

IMAGE="${RUST_IMAGE:-docker.io/library/rust:latest}"
ENGINE="${CONTAINER_ENGINE:-$(command -v podman || command -v docker || true)}"
[[ -n "$ENGINE" ]] || { echo "error: neither podman nor docker found" >&2; exit 1; }

PROFILE_FLAG="--release"
[[ "${1:-}" == "--debug" ]] && PROFILE_FLAG=""

# Keep the crate registry inside the project so repeat builds don't re-download.
mkdir -p .cargo-home

# Run as the calling user so build output isn't root-owned.
# Rootless podman needs keep-id for that; docker takes the uid directly.
if [[ "$(basename "$ENGINE")" == podman ]]; then
    USER_ARGS=(--userns=keep-id)
else
    USER_ARGS=(--user "$(id -u):$(id -g)")
fi

# :Z relabels the mount for SELinux (Fedora/RHEL); harmless elsewhere.
"$ENGINE" run --rm \
    "${USER_ARGS[@]}" \
    -v "$PWD":/app:Z \
    -w /app \
    -e CARGO_HOME=/app/.cargo-home \
    "$IMAGE" \
    cargo build $PROFILE_FLAG

BIN="target/${PROFILE_FLAG:+release}${PROFILE_FLAG:-debug}/amber-dos"
echo
echo "Built $BIN — run it with: ./$BIN"
