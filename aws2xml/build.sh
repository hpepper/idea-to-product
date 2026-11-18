#!/usr/bin/env bash
set -euo pipefail



PROFILE_FLAG="--release"
[[ "${1:-}" == "--debug" ]] && PROFILE_FLAG=""


export PATH="$HOME/.cargo/bin:$PATH"

script_path="$(dirname -- "${BASH_SOURCE[0]}")"

echo "III script_path $script_path"

cd "${script_path}" || exit 1
cargo build $PROFILE_FLAG

