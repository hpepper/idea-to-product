#!/usr/bin/env bash
# Checks for the build pipeline of GEDT-STORY-00019: T-01, and the T-54 tool image.
# Covers GEDT-AR-00024, GEDT-AR-00025, GEDT-AR-00030, and GEDT-AR-00034
# without a container engine:
# static inspection of build.sh, the Containerfile, and the policy files,
# plus build.sh and container/tool_image.sh runs against a fake engine,
# which records its arguments instead of starting a container.
# The deny.toml, ci.yml, and Containerfile structure checks are in
# tests/test_build_pipeline_config.rs.
# Needs bash, git, and sha256sum.
#
#   gui_editor/tests/pipeline/test_build_pipeline.sh

# Expected container script lines are literal text, so single-quoted '$VAR' is intended.
# shellcheck disable=SC2016
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
GUI_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
REPO_DIR="$(cd "$GUI_DIR/.." && pwd)"
OFFICIAL_RUST_IMAGE_RE='^docker\.io/library/rust:[0-9]+\.[0-9]+\.[0-9]+-[a-z]+@sha256:[0-9a-f]{64}$'
VALID_DIGEST="sha256:$(printf 'a%.0s' {1..64})"
OTHER_DIGEST="sha256:$(printf 'b%.0s' {1..64})"

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

passed=0
failed=0
pass() { passed=$((passed + 1)); echo "PASS $1"; }
fail() { failed=$((failed + 1)); echo "FAIL $1: $2"; }

# Fake runs use a scratch copy of build.sh and container/,
# so .cargo-home never lands in the repository.
# $WORK plays the repository root for container/tool_image.sh.
SANDBOX="$WORK/gui_editor"
mkdir -p "$SANDBOX/container"
cp "$GUI_DIR/build.sh" "$SANDBOX/build.sh"
cp "$GUI_DIR/container/Containerfile" "$GUI_DIR/container/tool_image.sh" "$SANDBOX/container/"
TOOL_IMAGE_SH="$SANDBOX/container/tool_image.sh"
# shellcheck source=SCRIPTDIR/../../container/tool_image.sh
PINNED_DEFAULT="$(source "$TOOL_IMAGE_SH" && echo "$PINNED_RUST_IMAGE")"
# Prints the reference that tool_image.sh must use for the base $1:
# the tag is the sha256 over the Containerfile bytes, one NUL byte,
# and the base reference.
expected_tool_image() {
    local hash
    hash="$({ cat "$SANDBOX/container/Containerfile"; printf '\0%s' "$1"; } | sha256sum)"
    echo "localhost/gui_editor-tools:${hash%% *}"
}
EXPECTED_TOOL_IMAGE="$(expected_tool_image "$PINNED_DEFAULT")"
# shellcheck source=SCRIPTDIR/../../container/tool_image.sh
PINNED_TOOL_VERSIONS="$(source "$TOOL_IMAGE_SH" && echo "$EXPECTED_TOOL_VERSIONS")"

# The fake engine keeps the image state in $FAKE_STATE, shared by all fake engines:
# image.present lists the built tags, image.label holds the base label of the last build,
# and tool.versions holds what the version check prints.
# Each fake reads its own replies from files next to it, named <engine>.<reply>.
# Only the main `run` writes $WORK/argv, one argument per line,
# so tests can assert on exact argv entries;
# `build` writes $FAKE_STATE/build.argv,
# and every request appends its first word to calls.
# Engine output is compared as text by the scripts under test, never evaluated.
export FAKE_STATE="$WORK/state"
mkdir -p "$FAKE_STATE"
# make_engine <path> <run exit code> [<--version output> <info exit code> <info output>]
make_engine() {
    local path="$1"
    mkdir -p "$(dirname "$path")"
    printf '%s\n' "$2" >"$path.run-rc"
    printf '%s\n' "${3:-}" >"$path.version"
    printf '%s\n' "${4:-0}" >"$path.info-rc"
    printf '%s\n' "${5:-}" >"$path.info"
    cat >"$path" <<'FAKE'
#!/usr/bin/env bash
printf '%s\n' "${1:-}" >>"$FAKE_STATE/calls"
case "${1:-}" in
    --version) cat "$0.version" ;;
    info)
        printf '%s\n' "$@" >"$0.info-argv"
        cat "$0.info"
        exit "$(cat "$0.info-rc")"
        ;;
    image)
        grep -qxF -- "${!#}" "$FAKE_STATE/image.present" 2>/dev/null || exit 1
        if [[ " $* " == *" --format "* ]]; then
            printf '%s\n' "$@" >"$FAKE_STATE/inspect.argv"
            cat "$FAKE_STATE/image.label"
        fi
        ;;
    build)
        printf '%s\n' "$@" >"$FAKE_STATE/build.argv"
        [[ ! -e "$FAKE_STATE/build-fails" ]] || exit 3
        previous=""
        for arg in "$@"; do
            if [[ "$arg" == BASE_IMAGE=* ]]; then
                printf '%s\n' "${arg#BASE_IMAGE=}" >"$FAKE_STATE/image.label"
            fi
            if [[ "$previous" == --tag ]]; then
                printf '%s\n' "$arg" >>"$FAKE_STATE/image.present"
            fi
            previous="$arg"
        done
        ;;
    run)
        if [[ " $* " == *" --network none "* ]]; then
            printf '%s\n' "$@" >"$FAKE_STATE/check.argv"
            cat "$FAKE_STATE/tool.versions"
            exit 0
        fi
        printf '%s\n' "$@" >"$WORK_ARGV"
        exit "$(cat "$0.run-rc")"
        ;;
    *) exit 64 ;;
esac
FAKE
    chmod +x "$path"
}
export WORK_ARGV="$WORK/argv"
make_engine "$WORK/bin/docker" 0
make_engine "$WORK/bin/podman" 0
make_engine "$WORK/bin/failing-engine" 4
make_engine "$WORK/dir with space/docker" 0

# Removes the fake image, so the next run builds it again.
remove_fake_image() {
    rm -f "$FAKE_STATE/image.present" "$FAKE_STATE/image.label" "$FAKE_STATE/build-fails"
    printf '%s\n' "$PINNED_TOOL_VERSIONS" >"$FAKE_STATE/tool.versions"
}
# Runs the scratch build.sh with the image state left as it is; sets $rc and $out.
run_build_cached() {
    rm -f "$WORK/argv" "$FAKE_STATE/build.argv" "$FAKE_STATE/check.argv" \
        "$FAKE_STATE/inspect.argv" "$FAKE_STATE/calls"
    set +e
    out="$(env "$@" "$SANDBOX/build.sh" "${BUILD_ARGS[@]}" 2>&1)"
    rc=$?
    set -e
}
# Runs the scratch build.sh without a built image; sets $rc and $out.
run_build() {
    remove_fake_image
    run_build_cached "$@"
}
argv_has_line() { grep -qxF -- "$1" "$WORK/argv" 2>/dev/null; }
build_argv_has_line() { grep -qxF -- "$1" "$FAKE_STATE/build.argv" 2>/dev/null; }
image_arg() {
    # The image is the argument right before "bash".
    grep -B1 -xF bash "$WORK/argv" | head -1
}
base_build_arg() {
    sed -n 's/^BASE_IMAGE=//p' "$FAKE_STATE/build.argv" 2>/dev/null
}
engine_called() { [[ -s "$FAKE_STATE/calls" ]]; }

# --- GEDT-AR-00025 / T-01 AC1: build.sh builds gui_editor, not amber-dos ---
if grep -q 'amber-dos' "$GUI_DIR/build.sh"; then
    fail "AR-00025_AC1_no_amber_dos" "build.sh still names amber-dos"
else
    pass "AR-00025_AC1_no_amber_dos"
fi
BUILD_ARGS=()
run_build CONTAINER_ENGINE="$WORK/bin/docker"
if [[ $rc -eq 0 && "$out" == *"target/release/gui_editor"* ]]; then
    pass "AR-00025_AC1_release_reports_gui_editor_binary"
else
    fail "AR-00025_AC1_release_reports_gui_editor_binary" "rc=$rc out=$out"
fi

# --- T-01 AC2 and T-54: base pinned by version and digest, build in the tool image ---
default_base="$(base_build_arg)"
if [[ "$default_base" =~ $OFFICIAL_RUST_IMAGE_RE && "$default_base" == "$PINNED_DEFAULT" ]]; then
    pass "AR-00034_T54_default_base_pinned_version_and_digest"
else
    fail "AR-00034_T54_default_base_pinned_version_and_digest" "base=$default_base"
fi
containerfile_base="$(sed -n 's/^ARG BASE_IMAGE=//p' "$GUI_DIR/container/Containerfile")"
if [[ "$containerfile_base" == "$default_base" ]] \
    && grep -qxF 'FROM ${BASE_IMAGE}' "$GUI_DIR/container/Containerfile" \
    && [[ "$(grep -c '^FROM ' "$GUI_DIR/container/Containerfile")" -eq 1 ]]; then
    pass "AR-00034_T54_containerfile_base_matches_tool_image_sh"
else
    fail "AR-00034_T54_containerfile_base_matches_tool_image_sh" "containerfile=$containerfile_base"
fi
if [[ "$(image_arg)" == "$EXPECTED_TOOL_IMAGE" ]]; then
    pass "AR-00034_T54_build_runs_in_tool_image_tagged_by_containerfile_and_base_hash"
else
    fail "AR-00034_T54_build_runs_in_tool_image_tagged_by_containerfile_and_base_hash" "image=$(image_arg)"
fi
# The tag differs from the hash of the Containerfile alone,
# so the base is part of the input.
containerfile_only_hash="$(sha256sum "$SANDBOX/container/Containerfile")"
if [[ "$EXPECTED_TOOL_IMAGE" != *":${containerfile_only_hash%% *}" ]]; then
    pass "AR-00034_T54_tag_hash_includes_base"
else
    fail "AR-00034_T54_tag_hash_includes_base" "tag=$EXPECTED_TOOL_IMAGE"
fi
# The Containerfile pins both cargo tools with --locked,
# and adds clippy and rustfmt through rustup.
for expected in \
    'RUN cargo install --locked --version 0.23.0 cargo-audit && rm -rf "$CARGO_HOME/registry"' \
    'RUN cargo install --locked --version 0.22.0 cargo-deny && rm -rf "$CARGO_HOME/registry"' \
    'RUN rustup component add clippy' \
    'RUN rustup component add rustfmt'; do
    if grep -qxF -- "$expected" "$GUI_DIR/container/Containerfile"; then
        pass "AR-00034_T54_containerfile_pins_tool[$expected]"
    else
        fail "AR-00034_T54_containerfile_pins_tool[$expected]" "missing from the Containerfile"
    fi
done
if [[ "$(grep -c '^RUN ' "$GUI_DIR/container/Containerfile")" -eq 4 ]]; then
    pass "AR-00034_T54_containerfile_adds_only_four_tools"
else
    fail "AR-00034_T54_containerfile_adds_only_four_tools" "other RUN lines"
fi
if ! grep -q 'cargo install' "$GUI_DIR/build.sh" "$GUI_DIR/container/tool_image.sh" \
    "$REPO_DIR/.github/workflows/ci.yml"; then
    pass "AR-00034_T54_no_cargo_install_per_run"
else
    fail "AR-00034_T54_no_cargo_install_per_run" "build.sh, tool_image.sh, or ci.yml runs cargo install"
fi

# Unpinned or malformed overrides must stop before the engine runs.
rejected_images=(
    "docker.io/library/rust:latest"
    "rust:1.100"
    "rust@sha256:abc"
    "rust@sha256:$(printf 'A%.0s' {1..64})"
    "rust@sha256:$(printf 'a%.0s' {1..63})"
    "rust@sha256:$(printf 'a%.0s' {1..65})"
    "rust@md5:$(printf 'a%.0s' {1..64})"
    "rust@${VALID_DIGEST} extra"
    "rust:1.0@${VALID_DIGEST}"$'\n'
    $'rust@'"${VALID_DIGEST}"$'\nmalicious'
    ""
)
for image in "${rejected_images[@]}"; do
    run_build CONTAINER_ENGINE="$WORK/bin/docker" RUST_IMAGE="$image"
    # An empty RUST_IMAGE falls back to the pinned default by design (${VAR:-default}).
    if [[ -z "$image" ]]; then
        if [[ $rc -eq 0 && "$(base_build_arg)" == "$default_base" ]]; then
            pass "AR-00025_AC2_empty_override_uses_pinned_default"
        else
            fail "AR-00025_AC2_empty_override_uses_pinned_default" "rc=$rc"
        fi
        continue
    fi
    if [[ $rc -eq 1 && "$out" == *"pinned by digest"* ]] && ! engine_called; then
        pass "AR-00025_AC2_rejects_override[${image//$'\n'/\\n}]"
    else
        fail "AR-00025_AC2_rejects_override[${image//$'\n'/\\n}]" "rc=$rc engine_ran=$(engine_called && echo yes || echo no)"
    fi
done

# Shell metacharacters are not valid in an image reference:
# the override is rejected before the engine runs and is never shell-evaluated.
marker="$WORK/injected"
override="rust:1.0\$(touch $marker);touch $marker@${VALID_DIGEST}"
run_build CONTAINER_ENGINE="$WORK/bin/docker" RUST_IMAGE="$override"
if [[ $rc -eq 1 && ! -e "$marker" ]] && ! engine_called; then
    pass "AR-00025_AC2_override_not_shell_evaluated"
else
    fail "AR-00025_AC2_override_not_shell_evaluated" "rc=$rc marker=$([[ -e $marker ]] && echo yes || echo no)"
fi

# A valid override, including a registry port,
# reaches the image build verbatim as one argument.
override="registry.example:5000/team/rust:1.100.0-trixie@${VALID_DIGEST}"
run_build CONTAINER_ENGINE="$WORK/bin/docker" RUST_IMAGE="$override"
if [[ $rc -eq 0 ]] && build_argv_has_line "BASE_IMAGE=$override"; then
    pass "AR-00025_AC2_valid_override_passed_verbatim"
else
    fail "AR-00025_AC2_valid_override_passed_verbatim" "rc=$rc base=$(base_build_arg)"
fi

# "--" ends the engine options, so the image argument can never be parsed as an option.
if [[ "$(grep -B2 -xF bash "$WORK/argv" | head -1)" == "--" ]]; then
    pass "AR-00025_AC2_image_after_end_of_options"
else
    fail "AR-00025_AC2_image_after_end_of_options" "no -- before the image"
fi
if [[ "$(tail -2 "$FAKE_STATE/build.argv" | head -1)" == "--" ]]; then
    pass "AR-00034_T54_build_context_after_end_of_options"
else
    fail "AR-00034_T54_build_context_after_end_of_options" "no -- before the build context"
fi

# An override that starts with '-' is parsed by the engine as an option, not as the image,
# which bypasses the digest pin.
run_build CONTAINER_ENGINE="$WORK/bin/docker" RUST_IMAGE="--volume=/:/host@${VALID_DIGEST}"
if [[ $rc -eq 1 ]] && ! engine_called; then
    pass "AR-00025_AC2_rejects_option_like_override"
else
    fail "AR-00025_AC2_rejects_option_like_override" "rc=$rc; the engine received an option-like image argument"
fi

# The documented override format is <image>:<tag>@sha256:<digest>;
# a tag-less override loses the version.
run_build CONTAINER_ENGINE="$WORK/bin/docker" RUST_IMAGE="docker.io/library/rust@${VALID_DIGEST}"
if [[ $rc -eq 1 ]]; then
    pass "AR-00025_AC2_override_requires_version_tag"
else
    fail "AR-00025_AC2_override_requires_version_tag" "rc=$rc; a digest without a version tag is accepted"
fi

# --- T-54: the tool image is built when its tag is missing, and checked before use ---
BUILD_ARGS=()
run_build CONTAINER_ENGINE="$WORK/bin/docker"
if [[ $rc -eq 0 ]] && build_argv_has_line "$EXPECTED_TOOL_IMAGE" \
    && build_argv_has_line "--tag" \
    && build_argv_has_line "$SANDBOX/container/Containerfile"; then
    pass "AR-00034_T54_builds_missing_tag_from_containerfile"
else
    fail "AR-00034_T54_builds_missing_tag_from_containerfile" "rc=$rc build=$(tr '\n' ' ' 2>/dev/null <"$FAKE_STATE/build.argv")"
fi
run_build_cached CONTAINER_ENGINE="$WORK/bin/docker"
if [[ $rc -eq 0 && ! -e "$FAKE_STATE/build.argv" && -e "$FAKE_STATE/check.argv" \
    && -e "$FAKE_STATE/inspect.argv" && -e "$WORK/argv" ]]; then
    pass "AR-00034_T54_reuses_existing_tag_and_checks_it"
else
    fail "AR-00034_T54_reuses_existing_tag_and_checks_it" "rc=$rc rebuilt=$([[ -e $FAKE_STATE/build.argv ]] && echo yes || echo no)"
fi
# The version check runs the image without network and without mounts.
if grep -qxF -- "--network" "$FAKE_STATE/check.argv" && grep -qxF -- "none" "$FAKE_STATE/check.argv" \
    && ! grep -qxE -- '-v|--volume|--mount' "$FAKE_STATE/check.argv" \
    && grep -qxF -- "$EXPECTED_TOOL_IMAGE" "$FAKE_STATE/check.argv"; then
    pass "AR-00034_T54_version_check_without_network_or_mounts"
else
    fail "AR-00034_T54_version_check_without_network_or_mounts" "argv=$(tr '\n' ' ' 2>/dev/null <"$FAKE_STATE/check.argv")"
fi
if grep -qxF -- '{{ index .Config.Labels "org.opencontainers.image.base.name" }}' \
    "$FAKE_STATE/inspect.argv"; then
    pass "AR-00034_T54_label_check_reads_base_label"
else
    fail "AR-00034_T54_label_check_reads_base_label" "argv=$(tr '\n' ' ' 2>/dev/null <"$FAKE_STATE/inspect.argv")"
fi

# Every failed content check stops the build before the main run (fail-closed).
# Asserts the last run failed with the message and never started the main run.
expect_check_stops() {
    local name="$1" message="$2"
    if [[ $rc -ne 0 && ! -e "$WORK/argv" && "$out" == *"$message"* && "$out" != *"Built "* ]]; then
        pass "$name"
    else
        fail "$name" "rc=$rc main_run=$([[ -e $WORK/argv ]] && echo yes || echo no) out=$out"
    fi
}
# Builds the fake image, then replaces one part of its state before a cached run.
tamper_and_run() {
    local file="$1" content="$2"
    run_build CONTAINER_ENGINE="$WORK/bin/docker"
    printf '%s\n' "$content" >"$FAKE_STATE/$file"
    run_build_cached CONTAINER_ENGINE="$WORK/bin/docker"
}
pinned_name="${PINNED_DEFAULT%@*}"
tamper_and_run image.label "${pinned_name}@${OTHER_DIGEST}"
expect_check_stops "AR-00034_T54_tampered_base_digest_label_stops" "was not built from"
tamper_and_run image.label "<no value>"
expect_check_stops "AR-00034_T54_missing_base_label_stops" "was not built from"
tamper_and_run image.label "${PINNED_DEFAULT} trailing"
expect_check_stops "AR-00034_T54_base_label_with_trailing_text_stops" "was not built from"
tamper_and_run image.label "${PINNED_DEFAULT}"$'\n'"${pinned_name}@${OTHER_DIGEST}"
expect_check_stops "AR-00034_T54_multiline_base_label_stops" "was not built from"
# Another base gives another tag,
# so an override builds a new image next to the cached one.
other_base="registry.example/rust:1.100.0-trixie@${OTHER_DIGEST}"
other_tool_image="$(expected_tool_image "$other_base")"
run_build CONTAINER_ENGINE="$WORK/bin/docker"
run_build_cached CONTAINER_ENGINE="$WORK/bin/docker" RUST_IMAGE="$other_base"
if [[ $rc -eq 0 && "$other_tool_image" != "$EXPECTED_TOOL_IMAGE" ]] \
    && build_argv_has_line "$other_tool_image" && build_argv_has_line "BASE_IMAGE=$other_base" \
    && [[ "$(image_arg)" == "$other_tool_image" ]]; then
    pass "AR-00034_T54_other_base_builds_new_tag"
else
    fail "AR-00034_T54_other_base_builds_new_tag" "rc=$rc image=$(image_arg) out=$out"
fi
# The same Containerfile and base give the same tag, which is reused.
run_build_cached CONTAINER_ENGINE="$WORK/bin/docker" RUST_IMAGE="$other_base"
if [[ $rc -eq 0 && ! -e "$FAKE_STATE/build.argv" && "$(image_arg)" == "$other_tool_image" ]]; then
    pass "AR-00034_T54_same_base_reuses_tag"
else
    fail "AR-00034_T54_same_base_reuses_tag" "rc=$rc image=$(image_arg) out=$out"
fi
# A tag of the override base whose label names another digest still fails closed.
printf '%s\n' "$PINNED_DEFAULT" >"$FAKE_STATE/image.label"
run_build_cached CONTAINER_ENGINE="$WORK/bin/docker" RUST_IMAGE="$other_base"
expect_check_stops "AR-00034_T54_override_tag_with_tampered_label_stops" "was not built from"
# A mirror of the same digest under another name holds the same base, so it passes.
mirror_base="registry.example/rust:1.100.0-trixie@${PINNED_DEFAULT##*@}"
run_build CONTAINER_ENGINE="$WORK/bin/docker"
run_build_cached CONTAINER_ENGINE="$WORK/bin/docker" RUST_IMAGE="$mirror_base"
if [[ $rc -eq 0 && -e "$WORK/argv" && "$(image_arg)" == "$(expected_tool_image "$mirror_base")" ]]; then
    pass "AR-00034_T54_same_digest_from_mirror_passes"
else
    fail "AR-00034_T54_same_digest_from_mirror_passes" "rc=$rc out=$out"
fi
# A changed Containerfile gives another tag, so the image is built again.
cp "$SANDBOX/container/Containerfile" "$WORK/Containerfile.orig"
echo "# changed" >>"$SANDBOX/container/Containerfile"
changed_tool_image="$(expected_tool_image "$PINNED_DEFAULT")"
run_build_cached CONTAINER_ENGINE="$WORK/bin/docker"
if [[ $rc -eq 0 && "$changed_tool_image" != "$EXPECTED_TOOL_IMAGE" ]] \
    && build_argv_has_line "$changed_tool_image" && [[ "$(image_arg)" == "$changed_tool_image" ]]; then
    pass "AR-00034_T54_changed_containerfile_builds_new_tag"
else
    fail "AR-00034_T54_changed_containerfile_builds_new_tag" "rc=$rc image=$(image_arg) out=$out"
fi
cp "$WORK/Containerfile.orig" "$SANDBOX/container/Containerfile"
remove_fake_image

wrong_audit="${PINNED_TOOL_VERSIONS/0.23.0/0.22.9}"
tamper_and_run tool.versions "$wrong_audit"
expect_check_stops "AR-00034_T54_wrong_cargo_audit_version_stops" "other tool versions"
wrong_deny="${PINNED_TOOL_VERSIONS/cargo-deny 0.22.0/cargo-deny 0.21.0}"
tamper_and_run tool.versions "$wrong_deny"
expect_check_stops "AR-00034_T54_wrong_cargo_deny_version_stops" "other tool versions"
tamper_and_run tool.versions "${PINNED_TOOL_VERSIONS/rustfmt /rustfmt 0.0.0-}"
expect_check_stops "AR-00034_T54_wrong_rustfmt_version_stops" "other tool versions"
tamper_and_run tool.versions "$(sed '/^clippy /d' <<<"$PINNED_TOOL_VERSIONS")"
expect_check_stops "AR-00034_T54_missing_clippy_stops" "other tool versions"
tamper_and_run tool.versions "$PINNED_TOOL_VERSIONS"$'\nextra'
expect_check_stops "AR-00034_T54_extra_version_output_stops" "other tool versions"
tamper_and_run tool.versions ""
expect_check_stops "AR-00034_T54_empty_version_output_stops" "other tool versions"
# The version output is compared as text and never evaluated.
tamper_and_run tool.versions "\$(touch $marker)"
expect_check_stops "AR-00034_T54_version_output_mismatch_stops" "other tool versions"
if [[ ! -e "$marker" ]]; then
    pass "AR-00034_T54_version_output_not_evaluated"
else
    fail "AR-00034_T54_version_output_not_evaluated" "marker created"
    rm -f "$marker"
fi
remove_fake_image
touch "$FAKE_STATE/build-fails"
run_build_cached CONTAINER_ENGINE="$WORK/bin/docker"
expect_check_stops "AR-00034_T54_failed_image_build_stops" "Building"
remove_fake_image

# A cargo-home bin directory would shadow the checked tools, so build.sh stops first.
mkdir -p "$SANDBOX/.cargo-home/bin"
run_build CONTAINER_ENGINE="$WORK/bin/docker"
if [[ $rc -eq 1 && "$out" == *".cargo-home/bin"* ]] && ! engine_called; then
    pass "AR-00034_T54_cargo_home_bin_stops"
else
    fail "AR-00034_T54_cargo_home_bin_stops" "rc=$rc out=$out"
fi
rmdir "$SANDBOX/.cargo-home/bin"

# --- T-01 AC3 and AC4: locked build, audit, and deny run inside the tool image ---
BUILD_ARGS=()
run_build CONTAINER_ENGINE="$WORK/bin/docker"
script="$(sed -n '/^set -euo pipefail$/,$p' "$WORK/argv")"
for expected in \
    'cargo build --locked $PROFILE_FLAG' \
    '/usr/local/cargo/bin/cargo-audit audit --db "$ADVISORY_DB_DIR/cargo-audit" --deny warnings' \
    '/usr/local/cargo/bin/cargo-deny --locked --config deny-gui_editor.toml check --deny warnings' \
    'set -euo pipefail'; do
    if grep -qxF -- "$expected" <<<"$script"; then
        pass "AR-00025_AC3_AC4_container_runs[$expected]"
    else
        fail "AR-00025_AC3_AC4_container_runs[$expected]" "missing from container script"
    fi
done
for env_arg in CARGO_HOME=/app/.cargo-home CARGO_TARGET_DIR=/app/target PROFILE_FLAG=--release; do
    if argv_has_line "$env_arg"; then
        pass "AR-00025_AC3_env[$env_arg]"
    else
        fail "AR-00025_AC3_env[$env_arg]" "missing engine argument"
    fi
done
if argv_has_line "--user" && argv_has_line "$(id -u):$(id -g)"; then
    pass "AR-00025_docker_runs_as_caller"
else
    fail "AR-00025_docker_runs_as_caller" "no --user uid:gid"
fi
if [[ -f "$GUI_DIR/Cargo.lock" ]] && ! git -C "$REPO_DIR" check-ignore -q "gui_editor/Cargo.lock"; then
    pass "AR-00025_AC3_cargo_lock_present_and_not_ignored"
else
    fail "AR-00025_AC3_cargo_lock_present_and_not_ignored" "Cargo.lock missing or ignored"
fi

# --debug selects the debug profile.
BUILD_ARGS=(--debug)
run_build CONTAINER_ENGINE="$WORK/bin/docker"
if [[ $rc -eq 0 && "$out" == *"target/debug/gui_editor"* ]] && argv_has_line "PROFILE_FLAG="; then
    pass "AR-00025_AC1_debug_profile"
else
    fail "AR-00025_AC1_debug_profile" "rc=$rc out=$out"
fi
BUILD_ARGS=()

# An unknown argument, such as a typo of --debug,
# fails instead of silently building release.
for bad_arg in --relase --debug-x release; do
    BUILD_ARGS=("$bad_arg")
    run_build CONTAINER_ENGINE="$WORK/bin/docker"
    if [[ $rc -eq 1 ]] && ! engine_called; then
        pass "AR-00025_AC1_rejects_unknown_argument[$bad_arg]"
    else
        fail "AR-00025_AC1_rejects_unknown_argument[$bad_arg]" "rc=$rc"
    fi
done
BUILD_ARGS=(--debug --debug)
run_build CONTAINER_ENGINE="$WORK/bin/docker"
if [[ $rc -eq 1 ]] && ! engine_called; then
    pass "AR-00025_AC1_rejects_extra_argument"
else
    fail "AR-00025_AC1_rejects_extra_argument" "rc=$rc"
fi
BUILD_ARGS=()

# podman gets keep-id instead of --user.
run_build CONTAINER_ENGINE="$WORK/bin/podman"
if [[ $rc -eq 0 ]] && argv_has_line "--userns=keep-id" && ! argv_has_line "--user"; then
    pass "AR-00025_podman_keep_id"
else
    fail "AR-00025_podman_keep_id" "rc=$rc"
fi

# GEDT-AR-00025 AC1: a fake `docker` whose `--version` and `info` replies come from files,
# so a case can mimic podman behind a docker name, rootless docker, or a broken engine.
# Runs build.sh against a probe engine; sets $rc, $out, and $probe_dir.
run_probe() {
    probe_dir="$WORK/probe-$1"
    make_engine "$probe_dir/docker" 0 "$2" "$3" "$4"
    run_build CONTAINER_ENGINE="$probe_dir/docker"
}
user_arg="$(id -u):$(id -g)"
# Asserts the user flags of the last run: keep-id, none, or user.
expect_user_args() {
    local want="$1" name="$2" ok=false
    if [[ $rc -eq 0 && -e "$WORK/argv" ]]; then
        case "$want" in
            keep-id)
                argv_has_line "--userns=keep-id" && ! argv_has_line "--user" && ok=true ;;
            none)
                ! argv_has_line "--userns=keep-id" && ! argv_has_line "--user" && ok=true ;;
            user)
                argv_has_line "--user" && argv_has_line "$user_arg" \
                    && ! argv_has_line "--userns=keep-id" && ok=true ;;
        esac
    fi
    if $ok; then
        pass "$name"
    else
        fail "$name" "rc=$rc, expected $want, argv=$(tr '\n' ' ' 2>/dev/null <"$WORK/argv")"
    fi
}
docker_version="Docker version 27.3.1, build ce12230"
rootful_info='["name=seccomp,profile=builtin","name=cgroupns"]'
rootless_info='["name=seccomp,profile=builtin","name=rootless","name=cgroupns"]'

run_probe podman-shim "podman version 5.2.3" 0 "$rootful_info"
expect_user_args keep-id "AR-00025_AC1_engine_user_mapping_docker_name_reporting_podman_gets_keep_id"
run_probe rootless "$docker_version" 0 "$rootless_info"
expect_user_args none "AR-00025_AC1_engine_user_mapping_rootless_docker_gets_no_user"
if [[ "$(cat "$probe_dir/docker.info-argv" 2>/dev/null)" == $'info\n--format\n{{json .SecurityOptions}}' ]]
then
    pass "AR-00025_AC1_engine_user_mapping_rootless_probe_reads_security_options_as_json"
else
    fail "AR-00025_AC1_engine_user_mapping_rootless_probe_reads_security_options_as_json" \
        "info argv=$(cat "$probe_dir/docker.info-argv" 2>/dev/null)"
fi
run_probe rootless-only "$docker_version" 0 '["name=rootless"]'
expect_user_args none "AR-00025_AC1_engine_user_mapping_rootless_docker_single_option_gets_no_user"
run_probe rootless-last "$docker_version" 0 '["name=seccomp,profile=builtin","name=rootless"]'
expect_user_args none "AR-00025_AC1_engine_user_mapping_rootless_docker_last_option_gets_no_user"
run_probe rootful "$docker_version" 0 "$rootful_info"
expect_user_args user "AR-00025_AC1_engine_user_mapping_rootful_docker_gets_user"
run_probe empty-options "$docker_version" 0 '[]'
expect_user_args user "AR-00025_AC1_engine_user_mapping_docker_without_options_gets_user"

# Every unclear answer keeps --user, the fail-safe choice.
run_probe info-fails "$docker_version" 1 "$rootless_info"
expect_user_args user "AR-00025_AC1_engine_user_mapping_failing_info_keeps_user"
run_probe garbage "$docker_version" 0 'Cannot connect to the Docker daemon, name=rootless'
expect_user_args user "AR-00025_AC1_engine_user_mapping_garbage_info_keeps_user"
run_probe unbracketed "$docker_version" 0 '"name=rootless"'
expect_user_args user "AR-00025_AC1_engine_user_mapping_unbracketed_rootless_keeps_user"
run_probe rootlessx "$docker_version" 0 '["name=rootlessx"]'
expect_user_args user "AR-00025_AC1_engine_user_mapping_rootlessx_keeps_user"
run_probe prefixed "$docker_version" 0 '["xname=rootless"]'
expect_user_args user "AR-00025_AC1_engine_user_mapping_prefixed_rootless_keeps_user"
run_probe two-arrays "$docker_version" 0 $'["name=seccomp"]\n["name=rootless"]'
expect_user_args user "AR-00025_AC1_engine_user_mapping_multiline_info_keeps_user"
run_probe trailing-text "$docker_version" 0 '["name=rootless"] trailing'
expect_user_args user "AR-00025_AC1_engine_user_mapping_trailing_text_keeps_user"
run_probe leading-text "$docker_version" 0 'leading ["name=rootless"]'
expect_user_args user "AR-00025_AC1_engine_user_mapping_leading_text_keeps_user"

# Engine output is compared as text and never evaluated.
probe_marker="$WORK/probe-evaluated"
probe_index=0
for payload in \
    "[\"name=rootless\$(touch $probe_marker)\"]" \
    "[\"\$(touch $probe_marker)\"]" \
    "[\"name=\`touch $probe_marker\`rootless\"]"; do
    probe_index=$((probe_index + 1))
    run_probe "subst-$probe_index" "$docker_version" 0 "$payload"
    expect_user_args user "AR-00025_AC1_engine_user_mapping_command_substitution_keeps_user[${payload//$WORK/WORK}]"
    if [[ ! -e "$probe_marker" ]]; then
        pass "AR-00025_AC1_engine_user_mapping_info_output_not_evaluated[${payload//$WORK/WORK}]"
    else
        fail "AR-00025_AC1_engine_user_mapping_info_output_not_evaluated[${payload//$WORK/WORK}]" "marker created"
        rm -f "$probe_marker"
    fi
done
run_probe subst-version "podman\$(touch $probe_marker)" 0 "$rootful_info"
if [[ ! -e "$probe_marker" ]]; then
    pass "AR-00025_AC1_engine_user_mapping_version_output_not_evaluated"
else
    fail "AR-00025_AC1_engine_user_mapping_version_output_not_evaluated" "marker created"
    rm -f "$probe_marker"
fi

# An engine path with spaces is quoted correctly.
run_build CONTAINER_ENGINE="$WORK/dir with space/docker"
if [[ $rc -eq 0 && -e "$WORK/argv" ]]; then
    pass "AR-00025_engine_path_with_space"
else
    fail "AR-00025_engine_path_with_space" "rc=$rc out=$out"
fi

# A failing container step (cargo audit or cargo deny) fails build.sh with the same code.
run_build CONTAINER_ENGINE="$WORK/bin/failing-engine"
if [[ $rc -eq 4 && "$out" != *"Built "* ]]; then
    pass "AR-00025_AC4_container_failure_propagates"
else
    fail "AR-00025_AC4_container_failure_propagates" "rc=$rc out=$out"
fi

# GEDT-AR-00025: no engine found fails cleanly, with a clear message and no run.
# bash runs build.sh directly, so the shebang's `env bash` lookup cannot fail first.
# The PATH holds only the tools that build.sh uses before the engine check,
# so the check itself, not a missing tool, ends the run.
CORE_BIN="$WORK/core-bin"
mkdir -p "$CORE_BIN"
ln -s "$(command -v dirname)" "$CORE_BIN/dirname"
rm -f "$WORK/argv"
set +e
out="$(env CONTAINER_ENGINE="" PATH="$CORE_BIN" /bin/bash "$SANDBOX/build.sh" 2>&1)"
rc=$?
set -e
if [[ $rc -eq 1 && "$out" == *"neither podman nor docker found"* && ! -e "$WORK/argv" ]]; then
    pass "AR-00025_no_engine_fails"
else
    fail "AR-00025_no_engine_fails" "rc=$rc out=$out"
fi

# --- T-54: container/tool_image.sh, used by CI and agents ---
# Runs the scratch tool_image.sh without a built image; sets $rc, $out (stdout), and $err.
run_tool_image() {
    remove_fake_image
    rm -f "$WORK/argv" "$FAKE_STATE/build.argv" "$FAKE_STATE/calls"
    set +e
    out="$(env "$@" 2>"$WORK/stderr")"
    rc=$?
    set -e
    err="$(cat "$WORK/stderr")"
}
run_tool_image CONTAINER_ENGINE="$WORK/bin/docker" bash "$TOOL_IMAGE_SH" build
if [[ $rc -eq 0 && "$out" == "$EXPECTED_TOOL_IMAGE" && -e "$FAKE_STATE/build.argv" ]]; then
    pass "AR-00034_T54_tool_image_build_prints_only_reference"
else
    fail "AR-00034_T54_tool_image_build_prints_only_reference" "rc=$rc out=$out err=$err"
fi
run_tool_image CONTAINER_ENGINE="$WORK/bin/docker" bash "$TOOL_IMAGE_SH" \
    run gui_editor cargo audit --deny warnings --file "../x/Cargo.lock"
# The run arguments after the image are the command, passed verbatim.
expected_tail="$EXPECTED_TOOL_IMAGE
cargo
audit
--deny
warnings
--file
../x/Cargo.lock"
if [[ $rc -eq 0 && "$(grep -A99 -xF -- "$EXPECTED_TOOL_IMAGE" "$WORK/argv")" == "$expected_tail" ]] \
    && [[ "$(grep -B1 -xF -- "$EXPECTED_TOOL_IMAGE" "$WORK/argv" | head -1)" == "--" ]]; then
    pass "AR-00034_T54_tool_image_run_passes_command_verbatim"
else
    fail "AR-00034_T54_tool_image_run_passes_command_verbatim" "rc=$rc argv=$(tr '\n' ' ' 2>/dev/null <"$WORK/argv") err=$err"
fi
# The repository, <dir> included, is mounted read-only;
# only the registry cache is writable.
for expected in "$WORK:/repo:ro,z" "$SANDBOX/.cargo-home:/cargo-home:z" /repo/gui_editor \
    CARGO_HOME=/cargo-home CARGO_TARGET_DIR=/tmp/target --user "$(id -u):$(id -g)"; do
    if argv_has_line "$expected"; then
        pass "AR-00034_T54_tool_image_run_argument[$expected]"
    else
        fail "AR-00034_T54_tool_image_run_argument[$expected]" "argv=$(tr '\n' ' ' 2>/dev/null <"$WORK/argv")"
    fi
done
run_tool_image CONTAINER_ENGINE="$WORK/bin/podman" bash "$TOOL_IMAGE_SH" run . true
if [[ $rc -eq 0 ]] && argv_has_line "--userns=keep-id" && ! argv_has_line "--user" \
    && argv_has_line "/repo/." && ! grep -q -- ":/repo/.:" "$WORK/argv"; then
    pass "AR-00034_T54_tool_image_run_podman_keep_id"
else
    fail "AR-00034_T54_tool_image_run_podman_keep_id" "rc=$rc err=$err"
fi
run_probe_tool_image() {
    make_engine "$WORK/probe-tool-rootless/docker" 0 "Docker version 27.3.1" 0 \
        '["name=seccomp,profile=builtin","name=rootless"]'
    run_tool_image CONTAINER_ENGINE="$WORK/probe-tool-rootless/docker" bash "$TOOL_IMAGE_SH" run . true
}
run_probe_tool_image
if [[ $rc -eq 0 ]] && ! argv_has_line "--userns=keep-id" && ! argv_has_line "--user"; then
    pass "AR-00034_T54_tool_image_run_rootless_docker_gets_no_user"
else
    fail "AR-00034_T54_tool_image_run_rootless_docker_gets_no_user" "rc=$rc err=$err"
fi
# Only . or an existing top-level directory of the repository is a valid <dir>.
mkdir -p "$WORK/outside"
for bad_dir in .. ../outside /etc gui_editor/src missing -v "" "gui editor" "gui_editor/.."; do
    run_tool_image CONTAINER_ENGINE="$WORK/bin/docker" bash "$TOOL_IMAGE_SH" run "$bad_dir" true
    if [[ $rc -ne 0 && ! -e "$WORK/argv" ]]; then
        pass "AR-00034_T54_tool_image_run_rejects_dir[$bad_dir]"
    else
        fail "AR-00034_T54_tool_image_run_rejects_dir[$bad_dir]" "rc=$rc"
    fi
done
for args in "" "build extra" "run gui_editor" "run" "shell"; do
    # shellcheck disable=SC2086 # Word splitting builds the argument list on purpose.
    run_tool_image CONTAINER_ENGINE="$WORK/bin/docker" bash "$TOOL_IMAGE_SH" $args
    if [[ $rc -eq 1 && "$err" == *usage* ]] && ! engine_called; then
        pass "AR-00034_T54_tool_image_usage[$args]"
    else
        fail "AR-00034_T54_tool_image_usage[$args]" "rc=$rc err=$err"
    fi
done
run_tool_image CONTAINER_ENGINE="$WORK/bin/docker" RUST_IMAGE="rust:latest" bash "$TOOL_IMAGE_SH" build
if [[ $rc -eq 1 && "$err" == *"pinned by digest"* ]] && ! engine_called; then
    pass "AR-00034_T54_tool_image_rejects_unpinned_base"
else
    fail "AR-00034_T54_tool_image_rejects_unpinned_base" "rc=$rc err=$err"
fi
# A failed content check stops `run` before the command.
run_tool_image CONTAINER_ENGINE="$WORK/bin/docker" bash "$TOOL_IMAGE_SH" build
printf '%s\n' "wrong" >"$FAKE_STATE/tool.versions"
rm -f "$WORK/argv"
set +e
err="$(env CONTAINER_ENGINE="$WORK/bin/docker" bash "$TOOL_IMAGE_SH" run . true 2>&1 >/dev/null)"
rc=$?
set -e
if [[ $rc -ne 0 && ! -e "$WORK/argv" && "$err" == *"other tool versions"* ]]; then
    pass "AR-00034_T54_tool_image_run_stops_on_failed_check"
else
    fail "AR-00034_T54_tool_image_run_stops_on_failed_check" "rc=$rc err=$err"
fi
remove_fake_image
mkdir -p "$SANDBOX/.cargo-home/bin"
run_tool_image CONTAINER_ENGINE="$WORK/bin/docker" bash "$TOOL_IMAGE_SH" run . true
if [[ $rc -eq 1 && ! -e "$WORK/argv" && "$err" == *".cargo-home/bin"* ]]; then
    pass "AR-00034_T54_tool_image_run_stops_on_cargo_home_bin"
else
    fail "AR-00034_T54_tool_image_run_stops_on_cargo_home_bin" "rc=$rc err=$err"
fi
rmdir "$SANDBOX/.cargo-home/bin"

# --- T-54 D-T54-01: planted cargo and rustup overrides stop build.sh and run ---
# A file planted in a writable mount could replace or weaken the checked tools,
# and the content check runs without mounts, so both scripts stop before the main run.
# Each case plants one path, as a file and as a symlink that dangles on the host.
# Prints "ok" when build.sh and `run gui_editor` both stopped and named the path in $1.
expect_both_stop() {
    local planted="$1" build_ok=no
    run_build CONTAINER_ENGINE="$WORK/bin/docker"
    if [[ $rc -eq 1 && "$out" == *"$planted"* ]] && ! engine_called; then
        build_ok=yes
    fi
    run_tool_image CONTAINER_ENGINE="$WORK/bin/docker" bash "$TOOL_IMAGE_SH" run gui_editor true
    if [[ $build_ok == yes && $rc -eq 1 && ! -e "$WORK/argv" && "$err" == *"$planted"* ]]; then
        echo ok
    else
        echo "build=$build_ok run_rc=$rc err=$err"
    fi
}
for planted in .cargo-home/config .cargo-home/config.toml .cargo-home/audit.toml \
    .cargo/config rust-toolchain rust-toolchain.toml; do
    mkdir -p "$(dirname "$SANDBOX/$planted")"
    for kind in file symlink; do
        if [[ $kind == file ]]; then
            printf '%s\n' '[alias]' 'audit = ["version"]' >"$SANDBOX/$planted"
        else
            ln -s /app/.cargo-home/planted "$SANDBOX/$planted"
        fi
        result="$(expect_both_stop "$planted")"
        if [[ "$result" == ok ]]; then
            pass "AR-00034_T54_planted_override_stops[$planted,$kind]"
        else
            fail "AR-00034_T54_planted_override_stops[$planted,$kind]" "$result"
        fi
        rm -f "$SANDBOX/$planted"
    done
done
rmdir "$SANDBOX/.cargo"
# `run` also searches the repository root, which cargo and rustup walk up to from <dir>.
for planted in .cargo/config rust-toolchain rust-toolchain.toml; do
    mkdir -p "$(dirname "$WORK/$planted")"
    printf '%s\n' '[toolchain]' 'path = "/repo/gui_editor/.cargo-home/planted"' >"$WORK/$planted"
    for dir in gui_editor .; do
        run_tool_image CONTAINER_ENGINE="$WORK/bin/docker" bash "$TOOL_IMAGE_SH" run "$dir" true
        if [[ $rc -eq 1 && ! -e "$WORK/argv" && "$err" == *"$planted"* ]]; then
            pass "AR-00034_T54_run_stops_on_repo_root_override[$planted,$dir]"
        else
            fail "AR-00034_T54_run_stops_on_repo_root_override[$planted,$dir]" "rc=$rc err=$err"
        fi
    done
    rm -f "$WORK/$planted"
done
rmdir "$WORK/.cargo"
# The committed .cargo/config.toml is expected, so the guard lets it pass.
# build.sh mounts gui_editor read-only and does not ask git;
# `run` needs git to track the file, so $WORK becomes a git checkout for this case.
mkdir -p "$SANDBOX/.cargo"
printf '%s\n' '[build]' 'target-dir = "/tmp/gui_editor"' >"$SANDBOX/.cargo/config.toml"
git -C "$WORK" init -q
git -C "$WORK" add -- gui_editor/.cargo/config.toml
run_build CONTAINER_ENGINE="$WORK/bin/docker"
build_rc=$rc
run_tool_image CONTAINER_ENGINE="$WORK/bin/docker" bash "$TOOL_IMAGE_SH" run gui_editor true
if [[ $build_rc -eq 0 && $rc -eq 0 && -e "$WORK/argv" ]]; then
    pass "AR-00034_T54_committed_cargo_config_toml_passes"
else
    fail "AR-00034_T54_committed_cargo_config_toml_passes" "build_rc=$build_rc run_rc=$rc err=$err"
fi

# --- T-54 question 2A: `run` refuses an untracked <dir>/.cargo/config.toml ---
# Asserts that the last `run` stopped before the engine run and named the file.
expect_run_refuses_config() {
    if [[ $rc -eq 1 && ! -e "$WORK/argv" && "$err" == *"$1"* ]]; then
        pass "$2"
    else
        fail "$2" "rc=$rc err=$err"
    fi
}
mkdir -p "$WORK/othercrate/.cargo"
printf '%s\n' '[alias]' 'zzprobe = "version"' >"$WORK/othercrate/.cargo/config.toml"
run_tool_image CONTAINER_ENGINE="$WORK/bin/docker" bash "$TOOL_IMAGE_SH" run othercrate true
expect_run_refuses_config othercrate/.cargo/config.toml "AR-00034_T54_Q2A_untracked_dir_config_refused"
# Once git tracks the file, `run` accepts it.
git -C "$WORK" add -- othercrate/.cargo/config.toml
run_tool_image CONTAINER_ENGINE="$WORK/bin/docker" bash "$TOOL_IMAGE_SH" run othercrate true
if [[ $rc -eq 0 && -e "$WORK/argv" ]]; then
    pass "AR-00034_T54_Q2A_tracked_dir_config_accepted"
else
    fail "AR-00034_T54_Q2A_tracked_dir_config_accepted" "rc=$rc err=$err"
fi
# Without git on PATH the tracked file cannot be confirmed, so `run` refuses it.
# The PATH holds every other tool that tool_image.sh and the fake engine use.
NOGIT_BIN="$WORK/nogit-bin"
mkdir -p "$NOGIT_BIN"
for tool in bash basename cat dirname env grep id mkdir printf sha256sum; do
    ln -s "$(command -v "$tool")" "$NOGIT_BIN/$tool"
done
run_tool_image PATH="$NOGIT_BIN" CONTAINER_ENGINE="$WORK/bin/docker" \
    bash "$TOOL_IMAGE_SH" run othercrate true
expect_run_refuses_config othercrate/.cargo/config.toml "AR-00034_T54_Q2A_git_missing_fails_closed"
# Without a config file, `run` does not need git.
mv "$WORK/othercrate/.cargo/config.toml" "$WORK/othercrate/.cargo/config.toml.off"
run_tool_image PATH="$NOGIT_BIN" CONTAINER_ENGINE="$WORK/bin/docker" \
    bash "$TOOL_IMAGE_SH" run othercrate true
if [[ $rc -eq 0 && -e "$WORK/argv" ]]; then
    pass "AR-00034_T54_Q2A_no_config_needs_no_git"
else
    fail "AR-00034_T54_Q2A_no_config_needs_no_git" "rc=$rc err=$err"
fi
rm -f "$WORK/othercrate/.cargo/config.toml.off"
# A tracked path that is a symlink can point anywhere in the container, so it is refused.
ln -s ../../gui_editor/.cargo-home/planted.toml "$WORK/othercrate/.cargo/config.toml"
git -C "$WORK" add -- othercrate/.cargo/config.toml
run_tool_image CONTAINER_ENGINE="$WORK/bin/docker" bash "$TOOL_IMAGE_SH" run othercrate true
expect_run_refuses_config othercrate/.cargo/config.toml "AR-00034_T54_Q2A_tracked_symlink_config_refused"
# A git checkout above the repository root does not vouch for the file:
# git must answer for the repository itself.
rm -rf "$WORK/.git"
OUTER="$WORK/outer"
mkdir -p "$OUTER/inner/gui_editor/container" "$OUTER/inner/othercrate/.cargo"
cp "$SANDBOX/container/Containerfile" "$SANDBOX/container/tool_image.sh" \
    "$OUTER/inner/gui_editor/container/"
printf '%s\n' '[alias]' 'zzprobe = "version"' >"$OUTER/inner/othercrate/.cargo/config.toml"
git -C "$OUTER" init -q
git -C "$OUTER" add -- inner/othercrate/.cargo/config.toml
run_tool_image CONTAINER_ENGINE="$WORK/bin/docker" \
    bash "$OUTER/inner/gui_editor/container/tool_image.sh" run othercrate true
expect_run_refuses_config othercrate/.cargo/config.toml "AR-00034_T54_Q2A_outer_git_checkout_refused"
rm -rf "$OUTER" "$WORK/othercrate" "$SANDBOX/.cargo"
# `run .` reads only the repository root config, which is always refused (D-T54-05).
mkdir -p "$WORK/.cargo"
printf '%s\n' '[alias]' 'zzprobe = "version"' >"$WORK/.cargo/config.toml"
git -C "$WORK" init -q
git -C "$WORK" add -- .cargo/config.toml
for dir in gui_editor .; do
    run_tool_image CONTAINER_ENGINE="$WORK/bin/docker" bash "$TOOL_IMAGE_SH" run "$dir" true
    expect_run_refuses_config .cargo/config.toml "AR-00034_T54_Q2A_tracked_repo_root_config_refused[$dir]"
done
rm -rf "$WORK/.git" "$WORK/.cargo"

# --- T-54 tester re-test round 3: question 2A and D-T54-05 bypass attempts ---
# The repository root config is refused as a symlink and as a directory too (D-T54-05).
mkdir -p "$WORK/.cargo"
for kind in symlink directory; do
    if [[ $kind == symlink ]]; then
        ln -s ../gui_editor/.cargo-home/planted.toml "$WORK/.cargo/config.toml"
    else
        mkdir "$WORK/.cargo/config.toml"
    fi
    for dir in gui_editor .; do
        run_tool_image CONTAINER_ENGINE="$WORK/bin/docker" bash "$TOOL_IMAGE_SH" run "$dir" true
        expect_run_refuses_config .cargo/config.toml \
            "AR-00034_T54_tester_D05_repo_root_config_refused[$kind,$dir]"
    done
    rm -rf "$WORK/.cargo/config.toml"
done
rm -rf "$WORK/.cargo"
# The repository is a git checkout in which othercrate/.cargo/config.toml is untracked.
# A second checkout, $FOREIGN, tracks the same path,
# so any git state that leaks into the check would make the file look tracked.
git -C "$WORK" init -q
mkdir -p "$WORK/othercrate/.cargo"
printf '%s\n' '[alias]' 'zzprobe = "version"' >"$WORK/othercrate/.cargo/config.toml"
FOREIGN="$WORK/foreign"
mkdir -p "$FOREIGN/othercrate/.cargo"
cp "$WORK/othercrate/.cargo/config.toml" "$FOREIGN/othercrate/.cargo/config.toml"
git -C "$FOREIGN" init -q
git -C "$FOREIGN" add -- othercrate/.cargo/config.toml
# A .git file in <dir> makes <dir> look like another checkout; the repository decides.
printf 'gitdir: %s\n' "$FOREIGN/.git" >"$WORK/othercrate/.git"
run_tool_image CONTAINER_ENGINE="$WORK/bin/docker" bash "$TOOL_IMAGE_SH" run othercrate true
expect_run_refuses_config othercrate/.cargo/config.toml "AR-00034_T54_tester_Q2A_git_file_in_dir_refused"
rm -f "$WORK/othercrate/.git"
# A <dir>/.cargo symlink to a directory whose config.toml is tracked elsewhere is refused:
# git does not track the path that cargo reads.
mkdir -p "$WORK/tracked/.cargo"
cp "$WORK/othercrate/.cargo/config.toml" "$WORK/tracked/.cargo/config.toml"
git -C "$WORK" add -- tracked/.cargo/config.toml
mv "$WORK/othercrate/.cargo" "$WORK/othercrate.cargo.saved"
ln -s ../tracked/.cargo "$WORK/othercrate/.cargo"
run_tool_image CONTAINER_ENGINE="$WORK/bin/docker" bash "$TOOL_IMAGE_SH" run othercrate true
expect_run_refuses_config othercrate/.cargo/config.toml "AR-00034_T54_tester_Q2A_symlinked_cargo_dir_refused"
rm -f "$WORK/othercrate/.cargo"
mv "$WORK/othercrate.cargo.saved" "$WORK/othercrate/.cargo"
# Git variables in the caller environment must not decide whether the repository
# tracks the file: each one points git at the index of $FOREIGN.
for git_env in "GIT_DIR=$FOREIGN/.git" "GIT_DIR=$FOREIGN/.git GIT_WORK_TREE=$WORK" \
    "GIT_INDEX_FILE=$FOREIGN/.git/index"; do
    read -r -a git_env_args <<<"$git_env"
    run_tool_image "${git_env_args[@]}" CONTAINER_ENGINE="$WORK/bin/docker" \
        bash "$TOOL_IMAGE_SH" run othercrate true
    expect_run_refuses_config othercrate/.cargo/config.toml \
        "AR-00034_T54_tester_Q2A_host_git_env_ignored[${git_env//$WORK/WORK}]"
done
# A core.fsmonitor command from the caller environment does not run during the check.
printf '#!/bin/sh\ntouch "%s/fsmonitor.ran"\n' "$WORK" >"$WORK/fsmonitor.sh"
chmod +x "$WORK/fsmonitor.sh"
git -C "$WORK" add -- othercrate/.cargo/config.toml
run_tool_image GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=core.fsmonitor \
    GIT_CONFIG_VALUE_0="$WORK/fsmonitor.sh" CONTAINER_ENGINE="$WORK/bin/docker" \
    bash "$TOOL_IMAGE_SH" run othercrate true
if [[ ! -e "$WORK/fsmonitor.ran" ]]; then
    pass "AR-00034_T54_tester_Q2A_env_fsmonitor_not_run"
else
    fail "AR-00034_T54_tester_Q2A_env_fsmonitor_not_run" "rc=$rc; the fsmonitor command ran"
fi
rm -rf "$WORK/.git" "$WORK/othercrate" "$WORK/tracked" "$FOREIGN" "$WORK/fsmonitor.sh" \
    "$WORK/fsmonitor.ran"

# --- T-54 tester re-test round 4: D-T54-06 bypass attempts ---
# The caller environment must not change the answer in either direction:
# an untracked config stays refused, and a tracked config stays accepted.
# $FOREIGN tracks the path, $EMPTY tracks nothing, and $FOREIGN/.gitconfig moves the work tree,
# so a leaked variable either makes the untracked file look tracked or the tracked file look
# untracked.
git -C "$WORK" init -q
mkdir -p "$WORK/othercrate/.cargo"
printf '%s\n' '[alias]' 'zzprobe = "version"' >"$WORK/othercrate/.cargo/config.toml"
FOREIGN="$WORK/foreign"
EMPTY="$WORK/empty"
mkdir -p "$FOREIGN/othercrate/.cargo" "$EMPTY"
cp "$WORK/othercrate/.cargo/config.toml" "$FOREIGN/othercrate/.cargo/config.toml"
git -C "$FOREIGN" init -q
git -C "$FOREIGN" add -- othercrate/.cargo/config.toml
git -C "$EMPTY" init -q
printf '[core]\n\tworktree = %s\n' "$FOREIGN" >"$FOREIGN/.gitconfig"
# Each entry is one environment, with variables separated by spaces.
hostile_git_envs=(
    "GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=core.worktree GIT_CONFIG_VALUE_0=$FOREIGN"
    "GIT_CONFIG_PARAMETERS='core.worktree'='$FOREIGN'"
    "GIT_CONFIG=$FOREIGN/.gitconfig"
    "GIT_CONFIG_GLOBAL=$FOREIGN/.gitconfig"
    "GIT_CONFIG_SYSTEM=$FOREIGN/.gitconfig"
    "HOME=$FOREIGN"
    "XDG_CONFIG_HOME=$FOREIGN"
    "GIT_CEILING_DIRECTORIES=$WORK"
    "GIT_CEILING_DIRECTORIES=$WORK/othercrate GIT_DISCOVERY_ACROSS_FILESYSTEM=1"
    "GIT_DISCOVERY_ACROSS_FILESYSTEM=1"
    "GIT_COMMON_DIR=$FOREIGN/.git"
    "GIT_DIR=$EMPTY/.git"
    "GIT_INDEX_FILE=$EMPTY/.git/index"
    "GIT_WORK_TREE=$FOREIGN"
)
for state in untracked tracked; do
    if [[ $state == tracked ]]; then
        git -C "$WORK" add -- othercrate/.cargo/config.toml
    fi
    for git_env in "${hostile_git_envs[@]}"; do
        read -r -a git_env_args <<<"$git_env"
        run_tool_image "${git_env_args[@]}" CONTAINER_ENGINE="$WORK/bin/docker" \
            bash "$TOOL_IMAGE_SH" run othercrate true
        name="AR-00034_T54_tester_D06_env_ignored[$state,${git_env//$WORK/WORK}]"
        if [[ $state == untracked ]]; then
            expect_run_refuses_config othercrate/.cargo/config.toml "$name"
        elif [[ $rc -eq 0 && -e "$WORK/argv" ]]; then
            pass "$name"
        else
            fail "$name" "rc=$rc err=$err"
        fi
    done
done
# git must be an executable: a shell function or an alias named git, from an exported
# function or from BASH_ENV, could answer anything, so the check refuses the file.
# The file is tracked here, so only a refusal of the replacement explains the refusal.
printf '%s\n' 'git() { printf "%s\n" "$PWD"; return 0; }' >"$WORK/bash_env_function"
printf '%s\n' 'shopt -s expand_aliases' "alias git='true'" >"$WORK/bash_env_alias"
for git_replacement in 'BASH_FUNC_git%%=() { return 0; }' "BASH_ENV=$WORK/bash_env_function" \
    "BASH_ENV=$WORK/bash_env_alias"; do
    run_tool_image "$git_replacement" CONTAINER_ENGINE="$WORK/bin/docker" \
        bash "$TOOL_IMAGE_SH" run othercrate true
    expect_run_refuses_config othercrate/.cargo/config.toml \
        "AR-00034_T54_tester_D06_git_replacement_refused[${git_replacement//$WORK/WORK}]"
done
rm -rf "$WORK/.git" "$WORK/othercrate" "$FOREIGN" "$EMPTY" "$WORK/bash_env_function" \
    "$WORK/bash_env_alias"

# --- T-54 tester cases: inputs against the <dir> rule and the content check ---
# More <dir> values that are not a top-level directory name of the repository.
for bad_dir in .git gui_editor/ ./gui_editor "$WORK/gui_editor" "../gui_editor" "-w"; do
    run_tool_image CONTAINER_ENGINE="$WORK/bin/docker" bash "$TOOL_IMAGE_SH" run "$bad_dir" true
    if [[ $rc -ne 0 && ! -e "$WORK/argv" ]]; then
        pass "AR-00034_T54_tester_run_rejects_dir[${bad_dir//$WORK/WORK}]"
    else
        fail "AR-00034_T54_tester_run_rejects_dir[${bad_dir//$WORK/WORK}]" "rc=$rc"
    fi
done
# A top-level symlink to an outside directory is not a directory of the repository;
# mounting it writable would let the command write outside the repository.
mkdir -p "$WORK/outside"
ln -s "$WORK/outside" "$WORK/linkdir"
run_tool_image CONTAINER_ENGINE="$WORK/bin/docker" bash "$TOOL_IMAGE_SH" run linkdir true
if [[ $rc -ne 0 && ! -e "$WORK/argv" ]]; then
    pass "AR-00034_T54_tester_run_rejects_symlinked_dir"
else
    fail "AR-00034_T54_tester_run_rejects_symlinked_dir" "rc=$rc; the symlink target is mounted writable: $(grep -F ':/repo/linkdir' "$WORK/argv" 2>/dev/null)"
fi
rm -f "$WORK/linkdir"
# An option-like or unpinned RUST_IMAGE stops tool_image.sh before the engine runs.
for image in "--volume=/:/host@${VALID_DIGEST}" "-x:1@${VALID_DIGEST}" "rust@${VALID_DIGEST}"; do
    run_tool_image CONTAINER_ENGINE="$WORK/bin/docker" RUST_IMAGE="$image" bash "$TOOL_IMAGE_SH" run . true
    if [[ $rc -eq 1 && "$err" == *"pinned by digest"* ]] && ! engine_called; then
        pass "AR-00034_T54_tester_tool_image_rejects_override[$image]"
    else
        fail "AR-00034_T54_tester_tool_image_rejects_override[$image]" "rc=$rc err=$err"
    fi
done
# A label that names another registry with the pinned digest passes:
# the check compares only the digest, as the definition of done states.
run_build CONTAINER_ENGINE="$WORK/bin/docker"
printf '%s\n' "other.example/rust:1.100.0-trixie@${PINNED_DEFAULT##*@}" >"$FAKE_STATE/image.label"
run_build_cached CONTAINER_ENGINE="$WORK/bin/docker"
if [[ $rc -eq 0 && -e "$WORK/argv" ]]; then
    pass "AR-00034_T54_tester_label_other_registry_same_digest_passes"
else
    fail "AR-00034_T54_tester_label_other_registry_same_digest_passes" "rc=$rc out=$out"
fi
# A label with the pinned digest but no version tag is not a pinned reference.
printf '%s\n' "docker.io/library/rust@${PINNED_DEFAULT##*@}" >"$FAKE_STATE/image.label"
run_build_cached CONTAINER_ENGINE="$WORK/bin/docker"
expect_check_stops "AR-00034_T54_tester_label_without_tag_stops" "was not built from"
# Duplicated or reordered version lines are not the pinned output.
tamper_and_run tool.versions "$(sed -n '1p' <<<"$PINNED_TOOL_VERSIONS")"$'\n'"$PINNED_TOOL_VERSIONS"
expect_check_stops "AR-00034_T54_tester_duplicated_version_line_stops" "other tool versions"
tamper_and_run tool.versions "$(tac <<<"$PINNED_TOOL_VERSIONS")"
expect_check_stops "AR-00034_T54_tester_reordered_version_lines_stop" "other tool versions"
tamper_and_run tool.versions "${PINNED_TOOL_VERSIONS/cargo-deny 0.22.0/cargo-deny 0.22.0 }"
expect_check_stops "AR-00034_T54_tester_trailing_space_version_line_stops" "other tool versions"
remove_fake_image
# A .cargo-home/bin symlink that dangles on the host but resolves in the container,
# where cargo-home is mounted, would still put other tools before the checked ones.
ln -s /app/.cargo-home/shadow "$SANDBOX/.cargo-home/bin"
run_build CONTAINER_ENGINE="$WORK/bin/docker"
if [[ $rc -ne 0 && ! -e "$WORK/argv" ]]; then
    pass "AR-00034_T54_tester_build_stops_on_dangling_cargo_home_bin_symlink"
else
    fail "AR-00034_T54_tester_build_stops_on_dangling_cargo_home_bin_symlink" "rc=$rc; build.sh ran with .cargo-home/bin -> /app/.cargo-home/shadow"
fi
rm -f "$SANDBOX/.cargo-home/bin"
ln -s /cargo-home/shadow "$SANDBOX/.cargo-home/bin"
run_tool_image CONTAINER_ENGINE="$WORK/bin/docker" bash "$TOOL_IMAGE_SH" run . true
if [[ $rc -ne 0 && ! -e "$WORK/argv" ]]; then
    pass "AR-00034_T54_tester_run_stops_on_dangling_cargo_home_bin_symlink"
else
    fail "AR-00034_T54_tester_run_stops_on_dangling_cargo_home_bin_symlink" "rc=$rc; tool_image.sh ran with .cargo-home/bin -> /cargo-home/shadow"
fi
rm -f "$SANDBOX/.cargo-home/bin"

# --- T-54 tester re-test of D-T54-01: guard bypass attempts ---
# Host variables that select another toolchain, wrapper, or cargo home must not reach
# the container: every -e entry sets a literal value, and only the expected names are set.
# A bare `-e NAME` would forward the host value of NAME.
# Prints the -e values of the main run, one per line, sorted.
env_args() { grep -A1 -xF -- -e "$WORK/argv" 2>/dev/null | grep -vxF -e -e -e -- | sort; }
hostile_env=(RUSTUP_TOOLCHAIN=planted RUSTC_WRAPPER=/bin/false CARGO_BUILD_RUSTC_WRAPPER=/bin/false
    CARGO_ALIAS_AUDIT=version CARGO_HOME=/planted CARGO_TARGET_DIR=/planted RUSTFLAGS=-Cplanted)
run_build CONTAINER_ENGINE="$WORK/bin/docker" "${hostile_env[@]}"
expected_env="$(printf '%s\n' CARGO_HOME=/app/.cargo-home CARGO_TARGET_DIR=/app/target \
    PROFILE_FLAG=--release | sort)"
if [[ $rc -eq 0 && "$(env_args)" == "$expected_env" ]] && ! grep -q -- '--env' "$WORK/argv"; then
    pass "AR-00034_T54_tester_build_does_not_forward_host_env"
else
    fail "AR-00034_T54_tester_build_does_not_forward_host_env" "rc=$rc env=$(env_args | tr '\n' ' ')"
fi
run_tool_image CONTAINER_ENGINE="$WORK/bin/docker" "${hostile_env[@]}" \
    bash "$TOOL_IMAGE_SH" run gui_editor true
expected_env="$(printf '%s\n' CARGO_HOME=/cargo-home CARGO_TARGET_DIR=/tmp/target | sort)"
if [[ $rc -eq 0 && "$(env_args)" == "$expected_env" ]] && ! grep -q -- '--env' "$WORK/argv"; then
    pass "AR-00034_T54_tester_run_does_not_forward_host_env"
else
    fail "AR-00034_T54_tester_run_does_not_forward_host_env" "rc=$rc env=$(env_args | tr '\n' ' ') err=$err"
fi
# The guard searches the <dir> of the request, not only gui_editor.
mkdir -p "$WORK/othercrate/.cargo"
for planted in .cargo/config rust-toolchain rust-toolchain.toml; do
    printf '%s\n' '[toolchain]' 'path = "/repo/othercrate/planted"' >"$WORK/othercrate/$planted"
    run_tool_image CONTAINER_ENGINE="$WORK/bin/docker" bash "$TOOL_IMAGE_SH" run othercrate true
    if [[ $rc -eq 1 && ! -e "$WORK/argv" && "$err" == *"othercrate/$planted"* ]]; then
        pass "AR-00034_T54_tester_run_stops_on_override_in_other_dir[$planted]"
    else
        fail "AR-00034_T54_tester_run_stops_on_override_in_other_dir[$planted]" "rc=$rc err=$err"
    fi
    rm -f "$WORK/othercrate/$planted"
done
rm -rf "$WORK/othercrate"
# A directory at a guarded path is refused as well, for example a config.toml directory.
mkdir -p "$SANDBOX/.cargo-home/config.toml"
result="$(expect_both_stop .cargo-home/config.toml)"
if [[ "$result" == ok ]]; then
    pass "AR-00034_T54_tester_planted_override_stops[.cargo-home/config.toml,directory]"
else
    fail "AR-00034_T54_tester_planted_override_stops[.cargo-home/config.toml,directory]" "$result"
fi
rmdir "$SANDBOX/.cargo-home/config.toml"
# D-T54-05: the repository root commits no .cargo/config.toml,
# and cargo reads /repo/.cargo/config.toml for every <dir>, like /repo/.cargo/config.
mkdir -p "$WORK/.cargo"
printf '%s\n' '[build]' 'rustc-wrapper = "/repo/gui_editor/.cargo-home/planted"' \
    >"$WORK/.cargo/config.toml"
for dir in gui_editor .; do
    run_tool_image CONTAINER_ENGINE="$WORK/bin/docker" bash "$TOOL_IMAGE_SH" run "$dir" true
    if [[ $rc -eq 1 && ! -e "$WORK/argv" && "$err" == *".cargo/config.toml"* ]]; then
        pass "AR-00034_T54_tester_run_stops_on_repo_root_cargo_config_toml[$dir]"
    else
        fail "AR-00034_T54_tester_run_stops_on_repo_root_cargo_config_toml[$dir]" "rc=$rc; run started with /repo/.cargo/config.toml planted"
    fi
done
rm -rf "$WORK/.cargo"

# --- T-54 D-T54-04: code in the container cannot change files that run on the host ---
# Prints the -v values of the main run, one per line, in argv order.
volume_args() { grep -A1 -xF -- -v "$WORK/argv" 2>/dev/null | grep -vxF -e -v -e -- || true; }
# Succeeds when a writable mount of the main run holds the host path $1.
# Any writable mount whose source contains $1 counts, even one hidden by a deeper mount,
# so the check errs on the side of reporting a path as writable.
is_writable_in_container() {
    local spec source options
    while IFS= read -r spec; do
        source="${spec%%:*}"
        options="${spec##*:}"
        if [[ ",$options," != *,ro,* && ( "$1" == "$source" || "$1" == "$source"/* ) ]]; then
            return 0
        fi
    done < <(volume_args)
    return 1
}
# Host files that run on the host or set the checks, relative to $WORK.
host_only_files=(gui_editor/build.sh gui_editor/container/tool_image.sh
    gui_editor/container/Containerfile gui_editor/check_advisory_ignores.sh
    gui_editor/deny-gui_editor.toml gui_editor/deny.toml gui_editor/.cargo/audit.toml
    gui_editor/.cargo/config.toml gui_editor/Cargo.lock gui_editor/src/main.rs
    sad2md/release.sh sad2md/.cargo/config.toml tui_editor/build.sh .git/config
    .github/workflows/ci.yml)
# Prints "ok" when no file of host_only_files is writable in the main run,
# and every path in the arguments is.
check_writable_paths() {
    local path result=ok
    for path in "${host_only_files[@]}"; do
        if is_writable_in_container "$WORK/$path"; then
            result="writable: $path"
        fi
    done
    for path in "$@"; do
        if ! is_writable_in_container "$WORK/$path"; then
            result="read-only: $path"
        fi
    done
    echo "$result"
}
run_build CONTAINER_ENGINE="$WORK/bin/docker"
expected_volumes="$SANDBOX:/app:ro,Z
$SANDBOX/target:/app/target:Z
$SANDBOX/.cargo-home:/app/.cargo-home:Z"
if [[ $rc -eq 0 && "$(volume_args)" == "$expected_volumes" ]]; then
    pass "AR-00034_T54_D04_build_mounts_source_read_only"
else
    fail "AR-00034_T54_D04_build_mounts_source_read_only" "rc=$rc volumes=$(volume_args | tr '\n' ' ')"
fi
result="$(check_writable_paths gui_editor/target/release/gui_editor gui_editor/.cargo-home/registry)"
if [[ "$result" == ok ]]; then
    pass "AR-00034_T54_D04_build_writes_only_target_and_cargo_home"
else
    fail "AR-00034_T54_D04_build_writes_only_target_and_cargo_home" "$result"
fi
# build.sh creates target/ on the host, so the engine does not create it as root.
if [[ -d "$SANDBOX/target" && ! -L "$SANDBOX/target" && -O "$SANDBOX/target" ]]; then
    pass "AR-00034_T54_D04_build_creates_target_owned_by_caller"
else
    fail "AR-00034_T54_D04_build_creates_target_owned_by_caller" "$(ls -ld "$SANDBOX/target" 2>&1)"
fi
mkdir -p "$WORK/sad2md"
for dir in gui_editor sad2md .; do
    run_tool_image CONTAINER_ENGINE="$WORK/bin/docker" bash "$TOOL_IMAGE_SH" run "$dir" true
    expected_volumes="$WORK:/repo:ro,z
$SANDBOX/.cargo-home:/cargo-home:z"
    writable_paths=(gui_editor/.cargo-home/registry)
    # `run .` has no crate, so it gets no test output folder.
    if [[ "$dir" != . ]]; then
        expected_volumes+="
$WORK/$dir/test-output:/repo/$dir/test-output:z"
        writable_paths+=("$dir/test-output/out.xml")
    fi
    result="$(check_writable_paths "${writable_paths[@]}")"
    if [[ $rc -eq 0 && "$(volume_args)" == "$expected_volumes" && "$result" == ok ]]; then
        pass "AR-00034_T54_D04_run_mounts_repository_read_only[$dir]"
    else
        fail "AR-00034_T54_D04_run_mounts_repository_read_only[$dir]" "rc=$rc result=$result volumes=$(volume_args | tr '\n' ' ') err=$err"
    fi
done
# --- T-54 round 4: <dir>/test-output is the only writable folder of the repository ---
# `run` creates the folder on the host, so the engine does not create it as root.
if [[ -d "$WORK/sad2md/test-output" && ! -L "$WORK/sad2md/test-output" \
    && -O "$WORK/sad2md/test-output" && ! -e "$WORK/test-output" ]]; then
    pass "AR-00034_T54_run_creates_test_output_owned_by_caller"
else
    fail "AR-00034_T54_run_creates_test_output_owned_by_caller" "$(ls -ld "$WORK/sad2md/test-output" "$WORK/test-output" 2>&1)"
fi
# Paths next to the folder, or with the folder name as a prefix, stay read-only.
for path in sad2md/Cargo.toml sad2md/test-output.xml sad2md/test-outputx/a sad2md/src/main.rs \
    gui_editor/test-output/a test-output/a; do
    run_tool_image CONTAINER_ENGINE="$WORK/bin/docker" bash "$TOOL_IMAGE_SH" run sad2md true
    if [[ $rc -eq 0 ]] && ! is_writable_in_container "$WORK/$path"; then
        pass "AR-00034_T54_run_test_output_only[$path]"
    else
        fail "AR-00034_T54_run_test_output_only[$path]" "rc=$rc volumes=$(volume_args | tr '\n' ' ')"
    fi
done
# A symlinked test-output would mount its target writable, so `run` stops before the engine run.
# Each case is a symlink to an outside directory and a symlink that dangles on the host.
rm -rf "$WORK/sad2md/test-output"
mkdir -p "$WORK/outside"
for link_target in "$WORK/outside" /repo/sad2md/src; do
    ln -s "$link_target" "$WORK/sad2md/test-output"
    run_tool_image CONTAINER_ENGINE="$WORK/bin/docker" bash "$TOOL_IMAGE_SH" run sad2md true
    if [[ $rc -eq 1 && ! -e "$WORK/argv" && "$err" == *"sad2md/test-output; a writable mount must be a directory"* ]]; then
        pass "AR-00034_T54_run_refuses_test_output_symlink[${link_target//$WORK/WORK}]"
    else
        fail "AR-00034_T54_run_refuses_test_output_symlink[${link_target//$WORK/WORK}]" "rc=$rc err=$err"
    fi
    rm -f "$WORK/sad2md/test-output"
done
# A file named test-output cannot become the folder, so `run` stops before the engine run.
printf 'x\n' >"$WORK/sad2md/test-output"
run_tool_image CONTAINER_ENGINE="$WORK/bin/docker" bash "$TOOL_IMAGE_SH" run sad2md true
if [[ $rc -ne 0 && ! -e "$WORK/argv" ]]; then
    pass "AR-00034_T54_run_refuses_test_output_file"
else
    fail "AR-00034_T54_run_refuses_test_output_file" "rc=$rc err=$err"
fi
rm -rf "$WORK/sad2md"
# A symlink at a writable mount point would mount its target writable,
# so both scripts stop.
# Each case is a symlink to an outside directory and a symlink that dangles on the host.
mkdir -p "$WORK/outside"
for link_target in "$WORK/outside" /app/elsewhere; do
    rm -rf "$SANDBOX/target"
    ln -s "$link_target" "$SANDBOX/target"
    run_build CONTAINER_ENGINE="$WORK/bin/docker"
    if [[ $rc -eq 1 && "$out" == *"symlink target"* ]] && ! engine_called; then
        pass "AR-00034_T54_D04_build_refuses_target_symlink[${link_target//$WORK/WORK}]"
    else
        fail "AR-00034_T54_D04_build_refuses_target_symlink[${link_target//$WORK/WORK}]" "rc=$rc out=$out"
    fi
    rm -f "$SANDBOX/target"
    mv "$SANDBOX/.cargo-home" "$WORK/cargo-home.saved"
    ln -s "$link_target" "$SANDBOX/.cargo-home"
    result="$(expect_both_stop ".cargo-home; a writable mount must be a directory")"
    if [[ "$result" == ok ]]; then
        pass "AR-00034_T54_D04_cargo_home_symlink_stops[${link_target//$WORK/WORK}]"
    else
        fail "AR-00034_T54_D04_cargo_home_symlink_stops[${link_target//$WORK/WORK}]" "$result"
    fi
    rm -f "$SANDBOX/.cargo-home"
    mv "$WORK/cargo-home.saved" "$SANDBOX/.cargo-home"
done
if [[ -z "$(ls -A "$WORK/outside")" ]]; then
    pass "AR-00034_T54_D04_symlink_target_left_untouched"
else
    fail "AR-00034_T54_D04_symlink_target_left_untouched" "$(ls -A "$WORK/outside")"
fi

# --- T-54 R-T54-01: advisory databases live in the container /tmp ---
# An earlier container could edit a database in the writable .cargo-home,
# so audit and deny fetch a clean copy into the container /tmp on every run.
# cargo-audit takes the path with --db; cargo-deny reads ADVISORY_DB_DIR through db-path.
# Fails the case named $1 when the last engine run mounts anything at or below /tmp.
expect_no_tmp_mount() {
    if [[ $rc -eq 0 && -s "$WORK/argv" ]] && ! grep -qE -- ':/tmp(/|:|$)' "$WORK/argv"; then
        pass "$1"
    else
        fail "$1" "rc=$rc argv=$(tr '\n' ' ' 2>/dev/null <"$WORK/argv")"
    fi
}
run_build CONTAINER_ENGINE="$WORK/bin/docker"
expect_no_tmp_mount "AR-00025_T54_R01_build_mounts_nothing_in_tmp"
build_script="$(sed -n '/^set -euo pipefail$/,$p' "$WORK/argv")"
for expected in 'export ADVISORY_DB_DIR=/tmp/advisory-dbs' \
    '/usr/local/cargo/bin/cargo-audit audit --db "$ADVISORY_DB_DIR/cargo-audit" --deny warnings'; do
    if grep -qxF -- "$expected" <<<"$build_script"; then
        pass "AR-00025_T54_R01_build_db_in_tmp[$expected]"
    else
        fail "AR-00025_T54_R01_build_db_in_tmp[$expected]" "missing from container script"
    fi
done
run_tool_image CONTAINER_ENGINE="$WORK/bin/docker" bash "$TOOL_IMAGE_SH" \
    run gui_editor env ADVISORY_DB_DIR=/tmp/advisory-dbs /usr/local/cargo/bin/cargo-deny check
expect_no_tmp_mount "AR-00025_T54_R01_run_mounts_nothing_in_tmp"
for policy in deny.toml deny-gui_editor.toml; do
    if [[ "$(grep -cxF 'db-path = "$ADVISORY_DB_DIR/cargo-deny"' "$GUI_DIR/$policy")" == 1 ]] \
        && [[ "$(grep -c 'db-path' "$GUI_DIR/$policy")" == 1 ]]; then
        pass "AR-00025_T54_R01_deny_db_path[$policy]"
    else
        fail "AR-00025_T54_R01_deny_db_path[$policy]" "db-path missing, changed, or repeated"
    fi
done
# Prints the uncommented ci.yml lines that run the tool $1 from the tool image.
ci_tool_lines() {
    grep -F "/usr/local/cargo/bin/$1 " "$REPO_DIR/.github/workflows/ci.yml" \
        | grep -vE '^[[:space:]]*#' || true
}
for tool_and_arg in "cargo-audit: --db /tmp/advisory-dbs/cargo-audit " \
    "cargo-deny: env ADVISORY_DB_DIR=/tmp/advisory-dbs /usr/local/cargo/bin/cargo-deny "; do
    tool="${tool_and_arg%%:*}"
    lines="$(ci_tool_lines "$tool")"
    if [[ -n "$lines" && "$(wc -l <<<"$lines")" == 1 && "$lines" == *"${tool_and_arg#*:}"* ]]; then
        pass "AR-00025_T54_R01_ci_db_in_tmp[$tool]"
    else
        fail "AR-00025_T54_R01_ci_db_in_tmp[$tool]" "$lines"
    fi
done

# --- T-54 tester re-test of R-T54-01: the database path cannot come from outside ---
# A host ADVISORY_DB_DIR that points into the writable cargo home must not reach the
# container: the container script sets the value itself, and no -e entry names it.
for host_value in /app/.cargo-home /cargo-home ''; do
    run_build CONTAINER_ENGINE="$WORK/bin/docker" ADVISORY_DB_DIR="$host_value"
    build_script="$(sed -n '/^set -euo pipefail$/,$p' "$WORK/argv")"
    if [[ $rc -eq 0 ]] && ! env_args | grep -q '^ADVISORY_DB_DIR' \
        && [[ "$(grep -c 'ADVISORY_DB_DIR=' <<<"$build_script")" == 1 ]] \
        && grep -qxF 'export ADVISORY_DB_DIR=/tmp/advisory-dbs' <<<"$build_script"; then
        pass "AR-00025_T54_tester_R01_build_ignores_host_db_dir[$host_value]"
    else
        fail "AR-00025_T54_tester_R01_build_ignores_host_db_dir[$host_value]" \
            "rc=$rc env=$(env_args | tr '\n' ' ')"
    fi
    run_tool_image CONTAINER_ENGINE="$WORK/bin/docker" ADVISORY_DB_DIR="$host_value" \
        bash "$TOOL_IMAGE_SH" run gui_editor /usr/local/cargo/bin/cargo-deny check
    if [[ $rc -eq 0 ]] && ! grep -q 'ADVISORY_DB_DIR' "$WORK/argv"; then
        pass "AR-00025_T54_tester_R01_run_ignores_host_db_dir[$host_value]"
    else
        fail "AR-00025_T54_tester_R01_run_ignores_host_db_dir[$host_value]" \
            "rc=$rc argv=$(tr '\n' ' ' 2>/dev/null <"$WORK/argv")"
    fi
done
# The image must not ship a database that a run would reuse instead of fetching a clean one:
# no Containerfile instruction names /tmp, an advisory database, a fetch, or a volume.
if ! grep -vE '^[[:space:]]*(#|$)' "$GUI_DIR/container/Containerfile" \
    | grep -qiE 'tmp|advisor|fetch|VOLUME'; then
    pass "AR-00025_T54_tester_R01_image_seeds_no_advisory_db"
else
    fail "AR-00025_T54_tester_R01_image_seeds_no_advisory_db" "Containerfile seeds /tmp or a database"
fi
# Every cargo-audit and cargo-deny step fetches: none skips the fetch,
# which would read whatever database the path already holds.
for path in gui_editor/build.sh .github/workflows/ci.yml; do
    lines="$(grep -E '/usr/local/cargo/bin/cargo-(audit|deny) ' "$REPO_DIR/$path" \
        | grep -vE '^[[:space:]]*#' | grep -v -- '--version' || true)"
    if [[ "$(wc -l <<<"$lines")" == 2 ]] \
        && ! grep -qE -- '--(no-fetch|disable-fetch|offline|frozen|stale)' <<<"$lines"; then
        pass "AR-00025_T54_tester_R01_checks_fetch_clean_db[$path]"
    else
        fail "AR-00025_T54_tester_R01_checks_fetch_clean_db[$path]" "$lines"
    fi
done

# --- T-01 AC4: deny.toml policy ---
# The deny.toml policy checks live in tests/test_build_pipeline_config.rs,
# run by cargo test.

# --- T-01 AC4: cargo audit and cargo deny share one advisory exception list ---
# The container runs the ignore-list check before any cargo step.
if grep -qxF 'bash ./check_advisory_ignores.sh' <<<"$script" \
    && [[ "$(grep -nxF 'bash ./check_advisory_ignores.sh' <<<"$script" | cut -d: -f1)" \
        -lt "$(grep -nxF 'cargo build --locked $PROFILE_FLAG' <<<"$script" | cut -d: -f1)" ]]; then
    pass "AR-00025_AC4_container_checks_ignore_lists_first"
else
    fail "AR-00025_AC4_container_checks_ignore_lists_first" "check missing or after cargo build"
fi
# GEDT-AR-00025 AC4: the supply-chain checks run before the build,
# so a flagged dependency never runs its build script or proc macros.
# The exact sequence also rules out any other command slipping in before the checks.
expected_script='set -euo pipefail
export ADVISORY_DB_DIR=/tmp/advisory-dbs
bash ./check_advisory_ignores.sh
/usr/local/cargo/bin/cargo-audit audit --db "$ADVISORY_DB_DIR/cargo-audit" --deny warnings
/usr/local/cargo/bin/cargo-deny --locked --config deny-gui_editor.toml check --deny warnings
cargo build --locked $PROFILE_FLAG'
actual_script="$(grep -v '^[[:space:]]*$' <<<"$script" || true)"
if [[ "$actual_script" == "$expected_script" ]]; then
    pass "AR-00025_AC4_container_order_checks_before_build"
else
    fail "AR-00025_AC4_container_order_checks_before_build" "script=$actual_script"
fi
if bash "$GUI_DIR/check_advisory_ignores.sh" >/dev/null 2>&1 \
    && [[ -f "$GUI_DIR/.cargo/audit.toml" ]]; then
    pass "AR-00025_AC4_committed_ignore_lists_match"
else
    fail "AR-00025_AC4_committed_ignore_lists_match" "check fails or .cargo/audit.toml missing"
fi
IGN="$WORK/ignores"
mkdir -p "$IGN/.cargo"
cp "$GUI_DIR/check_advisory_ignores.sh" "$IGN/"
# Runs the check on a scratch audit.toml and on two scratch policy files,
# deny.toml and deny-gui_editor.toml, with the same content; sets $rc.
check_ignores() {
    printf '%s\n' "$1" >"$IGN/deny.toml"
    printf '%s\n' "$1" >"$IGN/deny-gui_editor.toml"
    printf '%s\n' "$2" >"$IGN/.cargo/audit.toml"
    set +e
    (cd "$IGN" && bash ./check_advisory_ignores.sh >/dev/null 2>&1)
    rc=$?
    set -e
}
deny_one='[advisories]
ignore = [
    { id = "RUSTSEC-2021-0003", reason = "not reachable" },
]'
audit_one='[advisories]
# RUSTSEC-2021-0003: not reachable
ignore = ["RUSTSEC-2021-0003"]'
# Asserts the exit status of the last check_ignores run:
# "pass" for 0, "fail" for non-zero.
expect_check() {
    local want="$1" name="$2"
    if [[ ( $want == pass && $rc -eq 0 ) || ( $want == fail && $rc -ne 0 ) ]]; then
        pass "$name"
    else
        fail "$name" "rc=$rc, expected $want"
    fi
}
check_ignores "$deny_one" "$audit_one"
expect_check pass "AR-00025_AC4_ignores_mirrored_pass"
check_ignores '[advisories]
ignore = []' "$audit_one"
expect_check fail "AR-00025_AC4_audit_only_ignore_fails"
check_ignores "$deny_one" '[advisories]
# RUSTSEC-2021-0003: mentioned in a comment only
ignore = []'
expect_check fail "AR-00025_AC4_deny_only_ignore_fails"
check_ignores '[advisories]
ignore = [
    { id = "RUSTSEC-2021-0003" },
]' "$audit_one"
expect_check fail "AR-00025_AC4_deny_ignore_without_reason_fails"
check_ignores '[advisories]
ignore = ["RUSTSEC-2021-0003"]' "$audit_one"
expect_check fail "AR-00025_AC4_deny_bare_id_without_reason_fails"
check_ignores "$deny_one" "$audit_one
severity_threshold = \"critical\""
expect_check fail "AR-00025_AC4_audit_toml_other_setting_fails"
check_ignores "$deny_one" '[advisories]
ignore = ["RUSTSEC-2021-0003", "GHSA-xxxx-xxxx-xxxx"]'
expect_check fail "AR-00025_AC4_audit_toml_non_rustsec_id_fails"
mkdir -p "$IGN/.cargo-home"
touch "$IGN/.cargo-home/audit.toml"
check_ignores "$deny_one" "$audit_one"
expect_check fail "AR-00025_AC4_cargo_home_audit_toml_fails"
rm -rf "$IGN/.cargo-home"
# TOML literal strings ('...') are valid in both files, and both tools honour them.
# The check must not let a single-quoted ID skip the reason and mirroring rules.
check_ignores "[advisories]
ignore = ['RUSTSEC-2021-0003']" "[advisories]
ignore = ['RUSTSEC-2021-0003']"
expect_check fail "AR-00025_AC4_single_quoted_ids_without_reason_fail"
check_ignores "[advisories]
ignore = [
    { id = 'RUSTSEC-2021-0003' },
]" "$audit_one"
expect_check fail "AR-00025_AC4_deny_literal_id_without_reason_fails"
check_ignores '[advisories]
ignore = []' "[advisories]
ignore = ['RUSTSEC-2021-0003']"
expect_check fail "AR-00025_AC4_audit_single_quoted_id_not_in_deny_fails"
check_ignores '[advisories]
ignore = [
    { id = "RUSTSEC-2021-0003", reason = " " },
]' "$audit_one"
expect_check fail "AR-00025_AC4_deny_blank_reason_fails"
# TOML reads each of these headers as [advisories] or as its ignore array,
# and a \u escape hides the RUSTSEC text,
# so cargo deny ignores an ID that audit.toml lacks.
hidden_id="\\u0052USTSEC-2021-0003"
audit_empty='[advisories]
ignore = []'
check_ignores "[ advisories ]
ignore = [
    { id = \"${hidden_id}\", reason = \"not reachable\" },
]" "$audit_empty"
expect_check fail "AR-00025_AC4_deny_spaced_header_hidden_id_fails"
check_ignores "[\"advisories\"]
ignore = [
    { id = \"${hidden_id}\", reason = \"not reachable\" },
]" "$audit_empty"
expect_check fail "AR-00025_AC4_deny_quoted_header_hidden_id_fails"
check_ignores "[advisories]
version = 2
[[advisories.ignore]]
id = \"${hidden_id}\"
reason = \"not reachable\"" "$audit_empty"
expect_check fail "AR-00025_AC4_deny_array_of_tables_hidden_id_fails"

# GEDT-AR-00025, T-02: deny-gui_editor.toml is checked like deny.toml.
# Runs the check with its own deny-gui_editor.toml content, or without the file; sets $rc.
check_ignores_gui() {
    check_ignores "$1" "$3"
    if [[ "$2" == missing ]]; then
        rm -f "$IGN/deny-gui_editor.toml"
    else
        printf '%s\n' "$2" >"$IGN/deny-gui_editor.toml"
    fi
    set +e
    (cd "$IGN" && bash ./check_advisory_ignores.sh >/dev/null 2>&1)
    rc=$?
    set -e
}
check_ignores_gui "$deny_one" "$deny_one" "$audit_one"
expect_check pass "AR-00025_T02_gui_policy_ignores_mirrored_pass"
check_ignores_gui "$deny_one" missing "$audit_one"
expect_check fail "AR-00025_T02_gui_policy_missing_fails"
check_ignores_gui "$deny_one" "$audit_empty" "$audit_one"
expect_check fail "AR-00025_T02_gui_policy_without_audit_ignore_fails"
check_ignores_gui "$audit_empty" "$deny_one" "$audit_empty"
expect_check fail "AR-00025_T02_gui_policy_only_ignore_fails"
check_ignores_gui "$deny_one" "[advisories]
ignore = [
    { id = \"RUSTSEC-2021-0003\" },
]" "$audit_one"
expect_check fail "AR-00025_T02_gui_policy_ignore_without_reason_fails"
# Tester cases for T-02: the shared file stays required,
# and the subset rules reject a hidden ID in the gui_editor file too.
check_ignores_gui "$deny_one" "$deny_one" "$audit_one"
rm -f "$IGN/deny.toml"
set +e
(cd "$IGN" && bash ./check_advisory_ignores.sh >/dev/null 2>&1)
rc=$?
set -e
expect_check fail "AR-00025_T02_shared_policy_missing_fails"
check_ignores_gui "$audit_empty" "[advisories]
ignore = []
[bans]
note = 'RUSTSEC-2021-0003'" "$audit_empty"
expect_check fail "AR-00025_T02_gui_policy_single_quoted_id_fails"
check_ignores_gui "$audit_empty" "[advisories]
ignore = []
[[advisories.ignore]]
id = \"RUSTSEC-2021-0003\"" "$audit_empty"
expect_check fail "AR-00025_T02_gui_policy_array_of_tables_id_fails"

# GEDT-AR-00025 AC4: a file without a final newline is read to its last line, fail-closed.
# Runs the check on files written without a trailing newline; sets $rc.
check_ignores_no_eol() {
    printf '%s' "$1" >"$IGN/deny.toml"
    printf '%s' "$1" >"$IGN/deny-gui_editor.toml"
    printf '%s' "$2" >"$IGN/.cargo/audit.toml"
    set +e
    (cd "$IGN" && bash ./check_advisory_ignores.sh >/dev/null 2>&1)
    rc=$?
    set -e
}
check_ignores_no_eol "$deny_one" "$audit_one"
expect_check pass "AR-00025_AC4_valid_files_without_final_newline_pass"
check_ignores_no_eol "$deny_one" "$(printf '%s\n' "$audit_one")"
expect_check pass "AR-00025_AC4_valid_deny_without_final_newline_passes"
check_ignores_no_eol "$(printf '%s\n' "$deny_one")" "$audit_one"
expect_check pass "AR-00025_AC4_valid_audit_without_final_newline_passes"
check_ignores_no_eol '[advisories]
ignore = []
note = "RUSTSEC-2021-0003"' "$audit_empty"
expect_check fail "AR-00025_AC4_deny_id_on_unterminated_last_line_fails"
check_ignores_no_eol '[advisories]
ignore = []
[bans]
note = "RUSTSEC-2021-0003"' "$audit_empty"
expect_check fail "AR-00025_AC4_deny_id_outside_advisories_on_last_line_fails"
check_ignores_no_eol '[advisories]
ignore = []' '[advisories]
ignore = []
ignore = ["RUSTSEC-2021-0003"]'
expect_check fail "AR-00025_AC4_audit_id_on_unterminated_last_line_fails"
check_ignores_no_eol "$deny_one" "$audit_one
severity_threshold = \"critical\""
expect_check fail "AR-00025_AC4_audit_setting_on_unterminated_last_line_fails"

# --- T-01 AC5 and AC6: workflow structure ---
# The ci.yml structure checks live in tests/test_build_pipeline_config.rs,
# run by cargo test.
for crate in gui_editor sad2md sad_xml_sql tui_editor; do
    if [[ -f "$REPO_DIR/$crate/Cargo.toml" && -f "$REPO_DIR/$crate/Cargo.lock" ]]; then
        pass "AR-00030_AC6_matrix_crate_exists[$crate]"
    else
        fail "AR-00030_AC6_matrix_crate_exists[$crate]" "Cargo.toml or Cargo.lock missing"
    fi
done

# --- T-01 AC7: .cargo-home/ ignored in gui_editor and aws2xml ---
for path in gui_editor/.cargo-home/registry/x aws2xml/.cargo-home/registry/x; do
    if git -C "$REPO_DIR" check-ignore -q "$path"; then
        pass "AR-00024_AC7_ignored[$path]"
    else
        fail "AR-00024_AC7_ignored[$path]" "not ignored"
    fi
done
# --- T-54 round 4: the test output folder of `tool_image.sh run` is ignored in every crate ---
for path in gui_editor/test-output/x sad2md/test-output/x sad_xml_sql/test-output/x \
    tui_editor/test-output/x; do
    if git -C "$REPO_DIR" check-ignore -q "$path"; then
        pass "AR-00034_T54_test_output_ignored[$path]"
    else
        fail "AR-00034_T54_test_output_ignored[$path]" "not ignored"
    fi
done
# Code in the container can write any file or symlink into test-output,
# so no host-side script or CI step reads, sources, or runs anything from it.
for path in gui_editor/build.sh gui_editor/check_advisory_ignores.sh .github/workflows/ci.yml; do
    if [[ -f "$REPO_DIR/$path" ]] && ! grep -q "test-output" "$REPO_DIR/$path"; then
        pass "AR-00034_T54_tester_test_output_unused_on_host[$path]"
    else
        fail "AR-00034_T54_tester_test_output_unused_on_host[$path]" "missing or names test-output"
    fi
done

echo
echo "passed=$passed failed=$failed"
[[ $failed -eq 0 ]]
