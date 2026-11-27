#!/usr/bin/env bash


# TODO what the actual fuck does this script actually do? should we have a script that checks this script......

# Builds, checks, and runs the gui_editor tool image (GEDT-AR-00034, T-54, T-60).
# The image comes from container/Containerfile:
# the pinned official Rust image plus cargo-audit, cargo-deny, clippy, rustfmt,
# and the Debian trixie shellcheck package.
# build.sh sources this file; CI and agents run it directly.
#
#   container/tool_image.sh build
#       build the image when its tag is missing, check it, and print its reference
#   container/tool_image.sh run <dir> <command> [<argument>...]
#       the same, then run the command in <dir>, a top-level directory of the repository,
#       or `.` for the repository root
#
#   CONTAINER_ENGINE=<podman or docker>   select the engine; default: podman, then docker
#   RUST_IMAGE=<image>:<tag>@sha256:<digest>   build from another pinned base
#
# Every use checks the image first, and a failed check stops the caller (fail-closed):
# the base digest in the image label must match the pinned digest,
# the four Rust tools must report the pinned versions,
# and shellcheck must start.

# Pinned by version and digest so that a retagged upstream image cannot change the build.
# Update the tag and the digest together with the BASE_IMAGE default in the Containerfile.
PINNED_RUST_IMAGE="docker.io/library/rust:1.100.0-trixie@sha256:223cc32ddca8f3ee90c436dbc94b6b5f052ea7f9653c5201e9612f08ad7d21b8"
# The whole value must be an image reference with a version tag and a digest.
# Anchoring both ends rejects values that the engine would parse as an option,
# and the tag keeps the toolchain version visible next to the digest.
# Registry: lowercase host, optional port. Repository: lowercase path components.
IMAGE_NAME_RE='[a-z0-9]+([._-][a-z0-9]+)*(:[0-9]+)?(/[a-z0-9]+([._-]+[a-z0-9]+)*)*'
IMAGE_TAG_RE='[A-Za-z0-9_][A-Za-z0-9._-]{0,127}'
PINNED_IMAGE_RE="^${IMAGE_NAME_RE}:${IMAGE_TAG_RE}@sha256:[0-9a-f]{64}\$"

# Tool versions pinned in the Containerfile; keep both files in sync.
CARGO_AUDIT_VERSION="0.23.0"
CARGO_DENY_VERSION="0.22.0"
# clippy and rustfmt follow the toolchain of the pinned base, rustc 1.100.0.
CLIPPY_VERSION_LINE="clippy 0.1.100 (0ef1bc3860 2026-11-09)"
RUSTFMT_VERSION_LINE="rustfmt 1.11.0-stable (0ef1bc3860 2026-11-09)"
EXPECTED_TOOL_VERSIONS="cargo-audit ${CARGO_AUDIT_VERSION}
cargo-deny ${CARGO_DENY_VERSION}
${RUSTFMT_VERSION_LINE}
${CLIPPY_VERSION_LINE}"
# One command per line, in the order of EXPECTED_TOOL_VERSIONS.
# build.sh and CI run cargo-audit and cargo-deny by the same absolute paths,
# so the check covers the binaries that run and no cargo [alias] can replace them.
# The shellcheck package has no version pin (GEDT-AR-00034),
# so the last command only proves that it starts, and prints nothing.
TOOL_VERSION_SCRIPT='set -eu
/usr/local/cargo/bin/cargo-audit --version
/usr/local/cargo/bin/cargo-deny --version
cargo fmt --version
cargo clippy --version
dpkg-query --search /usr/bin/shellcheck
dpkg --verify shellcheck
/usr/bin/shellcheck --version >/dev/null'

TOOL_IMAGE_REPOSITORY="localhost/gui_editor-tools"
BASE_LABEL="org.opencontainers.image.base.name"
TOOL_IMAGE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TOOL_IMAGE_GUI_DIR="$(dirname "$TOOL_IMAGE_DIR")"
TOOL_IMAGE_REPO_DIR="$(dirname "$TOOL_IMAGE_GUI_DIR")"

# Fails unless $1 is pinned by version tag and digest.
require_pinned_base_image() {
    if [[ ! "$1" =~ $PINNED_IMAGE_RE ]]; then
        echo "error: RUST_IMAGE must be pinned by digest, with a version tag (<image>:<tag>@sha256:<digest>)" >&2
        return 1
    fi
}

# Sets ENGINE from CONTAINER_ENGINE, or to podman, then docker, from PATH.
select_engine() {
    ENGINE="${CONTAINER_ENGINE:-$(command -v podman || command -v docker || true)}"
    if [[ -z "$ENGINE" ]]; then
        echo "error: neither podman nor docker found" >&2
        return 1
    fi
}

# Sets USER_ARGS so that files written to a mount stay owned by the calling user.
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
select_user_args() {
    local engine_version
    engine_version="$("$ENGINE" --version 2>/dev/null || true)"
    if [[ "$(basename "$ENGINE")" == podman || "$engine_version" == *podman* ]]; then
        USER_ARGS=(--userns=keep-id)
    elif is_rootless_docker; then
        USER_ARGS=()
    else
        USER_ARGS=(--user "$(id -u):$(id -g)")
    fi
}

# Fails when a writable mount holds a file that would replace or weaken the checked tools.
# The content check runs without mounts, so it cannot see these files.
# $1 is the host directory mounted as CARGO_HOME;
# each further argument is a host directory that cargo and rustup search for configuration,
# from the container working directory up to the root of the mounts.
# In CARGO_HOME:
# bin holds subcommands that cargo runs before the ones on PATH,
# config and config.toml can define an [alias] for audit or deny,
# and audit.toml adds advisory exceptions outside the mirrored lists.
# In each searched directory:
# cargo prefers .cargo/config over the committed .cargo/config.toml,
# and rust-toolchain or rust-toolchain.toml can point rustup at a toolchain in a mount.
# The repository commits none of these files, so any of them was planted,
# for example by an older build.sh, a build script, or a test.
# -L also catches a symlink that dangles on the host but resolves in the container.
require_no_cargo_overrides() {
    local cargo_home="$1" dir path
    shift
    local paths=("$cargo_home/bin" "$cargo_home/config" "$cargo_home/config.toml" "$cargo_home/audit.toml")
    for dir in "$@"; do
        paths+=("$dir/.cargo/config" "$dir/rust-toolchain" "$dir/rust-toolchain.toml")
    done
    for path in "${paths[@]}"; do
        if [[ -e "$path" || -L "$path" ]]; then
            echo "error: remove $path; cargo or rustup would use it instead of the checked tools" >&2
            return 1
        fi
    done
}

# Fails when $1 is a symlink, and otherwise creates the directory $1 if it is missing.
# The engine resolves a symlinked mount source on the host,
# so a planted symlink would mount another host directory writable.
# Creating the directory on the host keeps it owned by you;
# the engine would create a missing one as root.
require_writable_mount_dir() {
    if [[ -L "$1" ]]; then
        echo "error: remove the symlink $1; a writable mount must be a directory" >&2
        return 1
    fi
    mkdir -p -- "$1"
}

# Succeeds when git tracks the file $2, a path relative to the repository root $1.
# Fails when git is missing, when $1 is not the top level of a git checkout,
# or when git cannot answer, so every unclear case refuses the file (fail-closed).
# The paths are arguments with literal pathspecs, never evaluated or used as patterns.
# git runs in an empty environment (D-T54-06):
# a GIT_* variable such as GIT_DIR, GIT_WORK_TREE, or GIT_INDEX_FILE from the caller
# would otherwise make git answer for another checkout or index.
# The empty environment and GIT_CONFIG_NOSYSTEM and GIT_CONFIG_GLOBAL also keep system,
# user, and command line configuration out,
# so only the configuration of the repository itself applies.
# core.fsmonitor=false keeps git from starting a monitor command configured there.
# git is the executable found on the caller PATH, never a shell function;
# the fixed PATH applies to its subprocesses.
GIT_SAFE_PATH="/usr/bin:/bin"
is_tracked_by_git() {
    local repo_dir="$1" path="$2" git_bin top_level
    git_bin="$(command -v git)" && [[ "$git_bin" == /* ]] || return 1
    local git_cmd=(env -i "PATH=$GIT_SAFE_PATH" GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null
        "$git_bin" -c core.fsmonitor=false)
    top_level="$("${git_cmd[@]}" -C "$repo_dir" rev-parse --show-toplevel 2>/dev/null)" \
        || return 1
    [[ "$top_level" == "$(cd -- "$repo_dir" && pwd -P)" ]] || return 1
    "${git_cmd[@]}" --literal-pathspecs -C "$repo_dir" \
        ls-files --error-unmatch -- "$path" >/dev/null 2>&1
}

# Fails when cargo would read a .cargo/config.toml that the repository does not commit.
# $1 is the repository root and $2 the <dir> of `run`.
# The repository root commits no .cargo/config.toml, so one there is always refused.
# In <dir>, the file must be a regular file tracked by git;
# an untracked file or a symlink was planted,
# for example by an older build script or test.
require_committed_cargo_config() {
    local repo_dir="$1" dir="$2"
    local path="$repo_dir/.cargo/config.toml"
    if [[ -e "$path" || -L "$path" ]]; then
        echo "error: remove $path; the repository commits no .cargo/config.toml at its root" >&2
        return 1
    fi
    [[ "$dir" != . ]] || return 0
    path="$repo_dir/$dir/.cargo/config.toml"
    if [[ ! -e "$path" && ! -L "$path" ]]; then
        return 0
    fi
    if [[ -L "$path" ]] || ! is_tracked_by_git "$repo_dir" "$dir/.cargo/config.toml"; then
        echo "error: remove $path; git does not track it, or git cannot tell" >&2
        return 1
    fi
}

# Sets TOOL_IMAGE to the local tag, the sha256 over the Containerfile and the base,
# so a change of either builds a new image instead of reusing a stale one.
# The hash input is the Containerfile bytes, one NUL byte,
# and BASE_IMAGE without a newline.
# BASE_IMAGE has passed require_pinned_base_image and holds no NUL byte,
# so the last NUL byte separates the two parts unambiguously.
# Needs BASE_IMAGE.
select_tool_image() {
    local tool_image_hash
    tool_image_hash="$(set -o pipefail
        { cat -- "$TOOL_IMAGE_DIR/Containerfile" && printf '\0%s' "$BASE_IMAGE"; } | sha256sum)" \
        || return 1
    TOOL_IMAGE="${TOOL_IMAGE_REPOSITORY}:${tool_image_hash%% *}"
}

# Fails unless the image records the base digest of BASE_IMAGE and holds the pinned tools.
# The engine output is compared as text, never evaluated.
# The version check runs without network and without mounts.
check_tool_image() {
    local base_label tool_versions
    base_label="$("$ENGINE" image inspect --format "{{ index .Config.Labels \"$BASE_LABEL\" }}" \
        -- "$TOOL_IMAGE")" || {
        echo "error: cannot read the $BASE_LABEL label of $TOOL_IMAGE" >&2
        return 1
    }
    if [[ ! "$base_label" =~ $PINNED_IMAGE_RE || "${base_label##*@}" != "${BASE_IMAGE##*@}" ]]; then
        echo "error: $TOOL_IMAGE was not built from ${BASE_IMAGE##*@}; its label is: $base_label" >&2
        echo "Remove the image with: $ENGINE image rm $TOOL_IMAGE" >&2
        return 1
    fi
    tool_versions="$("$ENGINE" run --rm --network none -- "$TOOL_IMAGE" \
        bash -c "$TOOL_VERSION_SCRIPT")" || {
        echo "error: cannot read the tool versions of $TOOL_IMAGE" >&2
        return 1
    }
    if [[ "$tool_versions" != "$EXPECTED_TOOL_VERSIONS" ]]; then
        echo "error: $TOOL_IMAGE holds other tool versions than pinned." >&2
        echo "Expected:" >&2
        echo "$EXPECTED_TOOL_VERSIONS" >&2
        echo "Found:" >&2
        echo "$tool_versions" >&2
        return 1
    fi
}

# Builds the image when its tag is missing, then checks it.
# Needs ENGINE and BASE_IMAGE; sets TOOL_IMAGE.
# Engine output goes to stderr, so `build` prints only the reference on stdout.
# "--" ends the engine options, so no argument is ever parsed as an option.
ensure_tool_image() {
    select_tool_image || return 1
    if ! "$ENGINE" image inspect -- "$TOOL_IMAGE" >/dev/null 2>&1; then
        echo "Building $TOOL_IMAGE from $BASE_IMAGE" >&2
        "$ENGINE" build \
            --tag "$TOOL_IMAGE" \
            --build-arg "BASE_IMAGE=$BASE_IMAGE" \
            --file "$TOOL_IMAGE_DIR/Containerfile" \
            -- \
            "$TOOL_IMAGE_DIR" >&2 || return 1
    fi
    check_tool_image
}

# Runs a command in the tool image, in a top-level directory of the repository.
# The whole repository, <dir> included, is mounted read-only, so no build script,
# proc macro, or test can change a file that later runs on the host, such as build.sh,
# this script, or a policy file such as deny-gui_editor.toml (D-T54-04).
# The command can write only to:
# - gui_editor/.cargo-home, the crate registry cache shared with build.sh, as /cargo-home;
# - <dir>/test-output, the git-ignored test output folder of the crate, at the same path;
# - the container /tmp.
# /tmp holds the build output in /tmp/target, std::env::temp_dir, CARGO_TARGET_TMPDIR,
# and the advisory databases of the CI audit and deny steps.
# /tmp is lost with the container; test-output stays on the host.
# Nothing on the host runs files from test-output.
# `run .` gets no test-output folder, because the repository root is not a crate.
# A symlink is not a directory of the repository:
# its target lies outside the checked paths.
# :z relabels the mounts for SELinux (Fedora/RHEL), shared between containers;
# harmless elsewhere.
run_in_tool_image() {
    local dir="$1"
    shift
    if [[ ! "$dir" =~ ^(\.|[A-Za-z0-9_][A-Za-z0-9_-]*)$ || -L "$TOOL_IMAGE_REPO_DIR/$dir" \
        || ! -d "$TOOL_IMAGE_REPO_DIR/$dir" ]]; then
        echo "error: <dir> must be . or a top-level directory of the repository" >&2
        return 1
    fi
    local cargo_home="$TOOL_IMAGE_GUI_DIR/.cargo-home"
    require_writable_mount_dir "$cargo_home" || return 1
    require_no_cargo_overrides "$cargo_home" "$TOOL_IMAGE_REPO_DIR/$dir" "$TOOL_IMAGE_REPO_DIR" \
        || return 1
    require_committed_cargo_config "$TOOL_IMAGE_REPO_DIR" "$dir" || return 1
    local test_output_args=()
    if [[ "$dir" != . ]]; then
        local test_output="$TOOL_IMAGE_REPO_DIR/$dir/test-output"
        require_writable_mount_dir "$test_output" || return 1
        test_output_args=(-v "$test_output:/repo/$dir/test-output:z")
    fi
    "$ENGINE" run --rm \
        "${USER_ARGS[@]}" \
        -v "$TOOL_IMAGE_REPO_DIR":/repo:ro,z \
        -v "$cargo_home":/cargo-home:z \
        "${test_output_args[@]}" \
        -w "/repo/$dir" \
        -e CARGO_HOME=/cargo-home \
        -e CARGO_TARGET_DIR=/tmp/target \
        -- \
        "$TOOL_IMAGE" \
        "$@"
}

tool_image_main() {
    set -euo pipefail
    if [[ ! ( $# -eq 1 && "$1" == build ) && ! ( $# -ge 3 && "$1" == run ) ]]; then
        echo "usage: $0 build | run <dir> <command> [<argument>...]" >&2
        return 1
    fi
    BASE_IMAGE="${RUST_IMAGE:-$PINNED_RUST_IMAGE}"
    require_pinned_base_image "$BASE_IMAGE"
    select_engine
    select_user_args
    ensure_tool_image
    if [[ "$1" == build ]]; then
        echo "$TOOL_IMAGE"
    else
        run_in_tool_image "${@:2}"
    fi
}

# Runs only when executed; build.sh sources the functions above.
if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
    tool_image_main "$@"
fi
