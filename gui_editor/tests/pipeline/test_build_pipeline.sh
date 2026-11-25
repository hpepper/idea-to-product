#!/usr/bin/env bash
# Checks for the T-01 build pipeline of GEDT-STORY-00019.
# Covers GEDT-AR-00024, GEDT-AR-00025, and GEDT-AR-00030 without a container engine:
# static inspection of build.sh, deny.toml, and ci.yml,
# plus build.sh runs against a fake engine,
# which records its arguments instead of starting a container.
# The deny.toml and ci.yml structure checks are in tests/test_build_pipeline_config.rs.
# Needs bash and git.
#
#   gui_editor/tests/pipeline/test_build_pipeline.sh

# Expected container script lines are literal text, so single-quoted '$VAR' is intended.
# shellcheck disable=SC2016
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
GUI_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
REPO_DIR="$(cd "$GUI_DIR/.." && pwd)"
WORKFLOW="$REPO_DIR/.github/workflows/ci.yml"
PINNED_IMAGE_RE='^docker\.io/library/rust:[0-9]+\.[0-9]+\.[0-9]+-[a-z]+@sha256:[0-9a-f]{64}$'
VALID_DIGEST="sha256:$(printf 'a%.0s' {1..64})"

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

passed=0
failed=0
pass() { passed=$((passed + 1)); echo "PASS $1"; }
fail() { failed=$((failed + 1)); echo "FAIL $1: $2"; }

# Fake runs use a scratch copy of build.sh, so .cargo-home never lands in the repository.
SANDBOX="$WORK/gui_editor"
mkdir -p "$SANDBOX"
cp "$GUI_DIR/build.sh" "$SANDBOX/build.sh"

# The fake engine writes one argument per line, so tests can assert on exact argv entries.
make_fake_engine() {
    local path="$1" exit_code="$2"
    cat >"$path" <<EOF
#!/usr/bin/env bash
printf '%s\n' "\$@" >"$WORK/argv"
exit $exit_code
EOF
    chmod +x "$path"
}
mkdir -p "$WORK/bin" "$WORK/dir with space"
make_fake_engine "$WORK/bin/docker" 0
make_fake_engine "$WORK/bin/podman" 0
make_fake_engine "$WORK/bin/failing-engine" 4
make_fake_engine "$WORK/dir with space/docker" 0

# Runs the scratch build.sh; sets $rc and $out.
run_build() {
    rm -f "$WORK/argv"
    set +e
    out="$(env "$@" "$SANDBOX/build.sh" "${BUILD_ARGS[@]}" 2>&1)"
    rc=$?
    set -e
}
argv_has_line() { grep -qxF -- "$1" "$WORK/argv" 2>/dev/null; }
image_arg() {
    # The image is the argument right before "bash".
    grep -B1 -xF bash "$WORK/argv" | head -1
}

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

# --- T-01 AC2: image pinned by version and digest ---
default_image="$(image_arg)"
if [[ "$default_image" =~ $PINNED_IMAGE_RE ]]; then
    pass "AR-00025_AC2_default_image_pinned_version_and_digest"
else
    fail "AR-00025_AC2_default_image_pinned_version_and_digest" "image=$default_image"
fi
ci_images="$(grep -oE 'docker\.io/library/rust:[^[:space:]"]+' "$WORKFLOW" | sort -u)"
if [[ "$ci_images" == "$default_image" ]]; then
    pass "AR-00025_AC2_ci_image_matches_build_sh"
else
    fail "AR-00025_AC2_ci_image_matches_build_sh" "ci=$ci_images build.sh=$default_image"
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
        if [[ $rc -eq 0 && "$(image_arg)" == "$default_image" ]]; then
            pass "AR-00025_AC2_empty_override_uses_pinned_default"
        else
            fail "AR-00025_AC2_empty_override_uses_pinned_default" "rc=$rc"
        fi
        continue
    fi
    if [[ $rc -eq 1 && ! -e "$WORK/argv" && "$out" == *"pinned by digest"* ]]; then
        pass "AR-00025_AC2_rejects_override[${image//$'\n'/\\n}]"
    else
        fail "AR-00025_AC2_rejects_override[${image//$'\n'/\\n}]" "rc=$rc engine_ran=$([[ -e $WORK/argv ]] && echo yes || echo no)"
    fi
done

# Shell metacharacters are not valid in an image reference:
# the override is rejected before the engine runs and is never shell-evaluated.
marker="$WORK/injected"
override="rust:1.0\$(touch $marker);touch $marker@${VALID_DIGEST}"
run_build CONTAINER_ENGINE="$WORK/bin/docker" RUST_IMAGE="$override"
if [[ $rc -eq 1 && ! -e "$marker" && ! -e "$WORK/argv" ]]; then
    pass "AR-00025_AC2_override_not_shell_evaluated"
else
    fail "AR-00025_AC2_override_not_shell_evaluated" "rc=$rc marker=$([[ -e $marker ]] && echo yes || echo no)"
fi

# A valid override, including a registry port,
# reaches the engine verbatim as one argument.
override="registry.example:5000/team/rust:1.100.0-trixie@${VALID_DIGEST}"
run_build CONTAINER_ENGINE="$WORK/bin/docker" RUST_IMAGE="$override"
if [[ $rc -eq 0 && "$(image_arg)" == "$override" ]]; then
    pass "AR-00025_AC2_valid_override_passed_verbatim"
else
    fail "AR-00025_AC2_valid_override_passed_verbatim" "rc=$rc image=$(image_arg)"
fi

# "--" ends the engine options, so the image argument can never be parsed as an option.
if [[ "$(grep -B2 -xF bash "$WORK/argv" | head -1)" == "--" ]]; then
    pass "AR-00025_AC2_image_after_end_of_options"
else
    fail "AR-00025_AC2_image_after_end_of_options" "no -- before the image"
fi

# An override that starts with '-' is parsed by the engine as an option, not as the image,
# which bypasses the digest pin (the next argument, "bash", becomes the image).
run_build CONTAINER_ENGINE="$WORK/bin/docker" RUST_IMAGE="--volume=/:/host@${VALID_DIGEST}"
if [[ $rc -eq 1 && ! -e "$WORK/argv" ]]; then
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

# --- T-01 AC3 and AC4: locked build, audit, and deny run inside the container ---
BUILD_ARGS=()
run_build CONTAINER_ENGINE="$WORK/bin/docker"
script="$(sed -n '/^set -euo pipefail$/,$p' "$WORK/argv")"
for expected in \
    'cargo build --locked $PROFILE_FLAG' \
    'cargo audit --deny warnings' \
    'cargo deny --locked --config deny-gui_editor.toml check --deny warnings' \
    'cargo install --locked cargo-audit --version "$CARGO_AUDIT_VERSION"' \
    'cargo install --locked cargo-deny --version "$CARGO_DENY_VERSION"' \
    'set -euo pipefail'; do
    if grep -qxF -- "$expected" <<<"$script"; then
        pass "AR-00025_AC3_AC4_container_runs[$expected]"
    else
        fail "AR-00025_AC3_AC4_container_runs[$expected]" "missing from container script"
    fi
done
for env_arg in CARGO_HOME=/app/.cargo-home CARGO_TARGET_DIR=/app/target PROFILE_FLAG=--release \
    CARGO_AUDIT_VERSION=0.23.0 CARGO_DENY_VERSION=0.22.0; do
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
    if [[ $rc -eq 1 && ! -e "$WORK/argv" ]]; then
        pass "AR-00025_AC1_rejects_unknown_argument[$bad_arg]"
    else
        fail "AR-00025_AC1_rejects_unknown_argument[$bad_arg]" "rc=$rc"
    fi
done
BUILD_ARGS=(--debug --debug)
run_build CONTAINER_ENGINE="$WORK/bin/docker"
if [[ $rc -eq 1 && ! -e "$WORK/argv" ]]; then
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
# Only the `run` request overwrites $WORK/argv; `info` records its own arguments.
make_probe_engine() {
    local dir="$1" version="$2" info_rc="$3" info_out="$4"
    mkdir -p "$dir"
    printf '%s\n' "$version" >"$dir/version.out"
    printf '%s\n' "$info_out" >"$dir/info.out"
    cat >"$dir/docker" <<EOF
#!/usr/bin/env bash
case "\${1:-}" in
    --version) cat "$dir/version.out" ;;
    info)
        printf '%s\n' "\$@" >"$dir/info.argv"
        cat "$dir/info.out"
        exit $info_rc
        ;;
    *) printf '%s\n' "\$@" >"$WORK/argv" ;;
esac
EOF
    chmod +x "$dir/docker"
}
# Runs build.sh against a probe engine; sets $rc, $out, and $probe_dir.
run_probe() {
    probe_dir="$WORK/probe-$1"
    make_probe_engine "$probe_dir" "$2" "$3" "$4"
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
        fail "$name" "rc=$rc, expected $want, argv=$(tr '\n' ' ' <"$WORK/argv" 2>/dev/null)"
    fi
}
docker_version="Docker version 27.3.1, build ce12230"
rootful_info='["name=seccomp,profile=builtin","name=cgroupns"]'
rootless_info='["name=seccomp,profile=builtin","name=rootless","name=cgroupns"]'

run_probe podman-shim "podman version 5.2.3" 0 "$rootful_info"
expect_user_args keep-id "AR-00025_AC1_engine_user_mapping_docker_name_reporting_podman_gets_keep_id"
run_probe rootless "$docker_version" 0 "$rootless_info"
expect_user_args none "AR-00025_AC1_engine_user_mapping_rootless_docker_gets_no_user"
if [[ "$(cat "$probe_dir/info.argv" 2>/dev/null)" == $'info\n--format\n{{json .SecurityOptions}}' ]]
then
    pass "AR-00025_AC1_engine_user_mapping_rootless_probe_reads_security_options_as_json"
else
    fail "AR-00025_AC1_engine_user_mapping_rootless_probe_reads_security_options_as_json" \
        "info argv=$(cat "$probe_dir/info.argv" 2>/dev/null)"
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
bash ./check_advisory_ignores.sh
cargo install --locked cargo-audit --version "$CARGO_AUDIT_VERSION"
cargo install --locked cargo-deny --version "$CARGO_DENY_VERSION"
cargo audit --deny warnings
cargo deny --locked --config deny-gui_editor.toml check --deny warnings
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

echo
echo "passed=$passed failed=$failed"
[[ $failed -eq 0 ]]
