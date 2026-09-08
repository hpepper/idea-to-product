#!/bin/bash

# This script builds the sad2md binary and copies it to the user's bin directory.

set -e

export PATH="$HOME/.cargo/bin:$PATH"

script_path="$(dirname -- "${BASH_SOURCE[0]}")"

echo "III script_path $script_path"

cd "${script_path}" || exit 1
cargo build --release

cp /tmp/sad2md/release/sad2md "/home/builder/bin"
