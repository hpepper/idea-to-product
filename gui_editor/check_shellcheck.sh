#!/usr/bin/env bash

# TODO what does this slop do? where is it used????
# TODO What needs to be updated in claude instructions to turn these shit comments into something that can be read by a human being???????

# Shellcheck gate (GEDT-AR-00055, GEDT-AR-00056, T-60).
# Fails when a script that git tracks under gui_editor, sad2md, sad_xml_sql, or tui_editor
# has a shellcheck finding,
# or a disable directive with no reason comment on the same line.
# build.sh sources this file and runs the gate before the build; agents run it directly:
#
#   gui_editor/check_shellcheck.sh
#
# CONTAINER_ENGINE and RUST_IMAGE work as for container/tool_image.sh.

SHELLCHECK_DIRS=(gui_editor sad2md sad_xml_sql tui_editor)
# An interpreter line starts with #!/,
# so a Rust inner attribute such as #![allow] never matches.
BASH_INTERPRETER_RE='^#!/.*bash'
# Matches a disable directive followed by nothing but blanks and #, so by no reason.
REASONLESS_DISABLE_RE='#[[:space:]]*shellcheck[[:space:]][^#]*disable=[^#]*(#[#[:space:]]*)?$'

# Sets CHECKED_SCRIPTS to the tracked scripts under SHELLCHECK_DIRS,
# as paths relative to the repository root.
# A script has a .sh or .bash extension or a bash interpreter line.
list_checked_scripts() {
    local paths=() path first_line
    CHECKED_SCRIPTS=()
    mapfile -d "" paths \
        < <(git -C "$TOOL_IMAGE_REPO_DIR" ls-files -z -- "${SHELLCHECK_DIRS[@]}")
    # The exit status of a process substitution is only visible through wait.
    wait "$!" || {
        echo "error: git cannot list the tracked files of $TOOL_IMAGE_REPO_DIR" >&2
        return 1
    }
    for path in "${paths[@]}"; do
        first_line=""
        if [[ "$path" != *.sh && "$path" != *.bash ]]; then
            read -r first_line <"$TOOL_IMAGE_REPO_DIR/$path" 2>/dev/null || true
            [[ "$first_line" =~ $BASH_INTERPRETER_RE ]] || continue
        fi
        CHECKED_SCRIPTS+=("$path")
    done
    if [[ ${#CHECKED_SCRIPTS[@]} -eq 0 ]]; then
        echo "error: the shellcheck gate found no tracked script" >&2
        return 1
    fi
}

# Fails on each disable directive in CHECKED_SCRIPTS that has no reason (GEDT-AR-00056),
# and prints each such line as <path>:<line number>:<line>.
check_disable_reasons() {
    local status=0
    (cd "$TOOL_IMAGE_REPO_DIR" &&
        grep -HnE -- "$REASONLESS_DISABLE_RE" "${CHECKED_SCRIPTS[@]}") \
        >&2 || status=$?
    # grep exits 1 when no line matches;
    # 0 means a reasonless directive, and 2 an unreadable file.
    if [[ $status -ne 1 ]]; then
        echo "error: a disable directive above needs a reason comment" \
            "after it on the same line (GEDT-AR-00056)" >&2
        return 1
    fi
}

# Runs shellcheck at default settings over CHECKED_SCRIPTS in the tool image
# (GEDT-AR-00055).
# Needs ENGINE, USER_ARGS, and TOOL_IMAGE from container/tool_image.sh.
# --norc keeps any .shellcheckrc from changing the default settings.
# One run over all scripts lets shellcheck follow a sourced tracked script without -x.
# The repository is mounted read-only,
# so shellcheck writes nothing to the host (GEDT-AR-00043).
run_shellcheck_gate() {
    list_checked_scripts || return 1
    check_disable_reasons || return 1
    echo "Shellcheck gate: ${#CHECKED_SCRIPTS[@]} scripts" >&2
    "$ENGINE" run --rm --network none "${USER_ARGS[@]}" \
        -v "$TOOL_IMAGE_REPO_DIR":/repo:ro,z -w /repo -- \
        "$TOOL_IMAGE" /usr/bin/shellcheck --norc -- "${CHECKED_SCRIPTS[@]}" || {
        echo "error: shellcheck reports findings in the scripts above (GEDT-AR-00055)" >&2
        return 1
    }
}

check_shellcheck_main() {
    set -euo pipefail
    if [[ $# -ne 0 ]]; then
        echo "usage: $0" >&2
        return 1
    fi
    # shellcheck source=SCRIPTDIR/container/tool_image.sh
    source "$(dirname "${BASH_SOURCE[0]}")/container/tool_image.sh"
    BASE_IMAGE="${RUST_IMAGE:-$PINNED_RUST_IMAGE}"
    require_pinned_base_image "$BASE_IMAGE"
    select_engine
    select_user_args
    ensure_tool_image
    run_shellcheck_gate
}

# Runs only when executed; build.sh sources the functions above.
if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
    check_shellcheck_main "$@"
fi
