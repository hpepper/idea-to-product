#!/usr/bin/env bash

# TODO what does this slop do? where is it used????
# TODO What needs to be updated in claude instructions to turn these shit comments into something that can be read by a human being???????

# Shellcheck gate of the checked crates (GEDT-AR-00055, GEDT-AR-00056, T-60).
# Fails when a hand-written script that git tracks under gui_editor, sad2md, sad_xml_sql,
# or tui_editor holds a shellcheck finding,
# or a disable directive with no reason comment after it on the same line.
# The repository-root scripts and aws2xml stay unchecked, as the product owner decided.
# build.sh sources this file and runs the gate before the build; agents run it directly:
#
#   gui_editor/check_shellcheck.sh
#
#   CONTAINER_ENGINE and RUST_IMAGE work as for container/tool_image.sh.

# A tracked file is a script when its extension is .sh or .bash, in any letter case,
# or when its first line starts with #!, as in the committed-language check of T-55.
# A first line that starts with #![ is a Rust inner attribute, not an interpreter line.
SHELLCHECK_DIRS=(gui_editor sad2md sad_xml_sql tui_editor)
SCRIPT_EXTENSIONS=(sh bash)
RUST_ATTRIBUTE_RE='^#![[:space:]]*\['

# The absolute path that check_tool_image verifies as the file of the Debian package,
# so no PATH entry of the image can replace it.
SHELLCHECK_BIN="/usr/bin/shellcheck"

# A directive is a comment, so its # starts a word:
# it opens the line or follows a blank or one of the shell metacharacters ;&|()<>.
# Shellcheck reads directives only in lowercase.
# Group 2 holds the key=value pairs up to the next #, and group 3 the comment after them.
# The match is wider than what shellcheck honors, so it never misses a directive.
SHELLCHECK_DIRECTIVE_RE='(^|[[:space:];&|()<>])#[[:space:]]*shellcheck[[:space:]]+([^#]*)(#.*)?$'
DISABLE_KEY_RE='(^|[[:space:]])disable='
# A reason holds at least one character other than # and blanks.
REASON_COMMENT_RE='^#+[[:space:]]*[^#[:space:]]'

# Sets CHECKED_SCRIPTS to the tracked scripts under SHELLCHECK_DIRS of the repository $1,
# as paths relative to $1, in git order.
# Fails when git is missing, when $1 is not the top level of a git checkout,
# when git cannot answer, or when a candidate is not a regular file in the working tree,
# so every unclear case stops the gate (fail-closed).
# git runs as in is_tracked_by_git of container/tool_image.sh (D-T54-06):
# an empty environment and no system or user configuration,
# so no GIT_* variable or monitor command changes the answer.
# Paths are read NUL-separated and used only as arguments, never evaluated.
list_checked_scripts() {
    local repo_dir="$1" git_bin top_level entry mode path previous="" name extension
    local first_line file
    CHECKED_SCRIPTS=()
    git_bin="$(command -v git || true)"
    if [[ "$git_bin" != /* ]]; then
        echo "error: git not found; the shellcheck gate needs the tracked file list" >&2
        return 1
    fi
    local git_cmd=(env -i "PATH=$GIT_SAFE_PATH" GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null
        "$git_bin" -c core.fsmonitor=false --literal-pathspecs -C "$repo_dir")
    top_level="$("${git_cmd[@]}" rev-parse --show-toplevel 2>/dev/null || true)"
    if [[ -z "$top_level" || "$top_level" != "$(cd -- "$repo_dir" && pwd -P)" ]]; then
        echo "error: $repo_dir is not the top level of a git checkout" >&2
        return 1
    fi
    # The last element is the git exit status,
    # because a process substitution would otherwise hide a failing git.
    local entries=()
    mapfile -d '' entries < <("${git_cmd[@]}" ls-files --stage -z -- "${SHELLCHECK_DIRS[@]}";
        printf 'rc=%s\0' "$?")
    if [[ "${entries[-1]:-}" != rc=0 ]]; then
        echo "error: git cannot list the tracked files of $repo_dir" >&2
        return 1
    fi
    unset 'entries[-1]'
    for entry in "${entries[@]}"; do
        # Each entry reads "<mode> <object> <stage>\t<path>".
        mode="${entry%% *}"
        path="${entry#*$'\t'}"
        # A conflicted path has one entry per stage; check the file once.
        [[ "$path" != "$previous" ]] || continue
        previous="$path"
        # A symlink names its target, which git lists as its own entry,
        # and a gitlink names a commit of another repository.
        [[ "$mode" == 100644 || "$mode" == 100755 ]] || continue
        file="$repo_dir/$path"
        name="${path##*/}"
        extension=""
        [[ "$name" != *.* ]] || extension="${name##*.}"
        extension="${extension,,}"
        # A missing file cannot be classified,
        # and a symlink would make the host read another file than the index names.
        if [[ ! -f "$file" || -L "$file" ]]; then
            echo "error: $path is tracked, but not a regular file in the working tree" >&2
            return 1
        fi
        if [[ -n "$extension" && " ${SCRIPT_EXTENSIONS[*]} " == *" $extension "* ]]; then
            CHECKED_SCRIPTS+=("$path")
            continue
        fi
        # Only the start of the first line matters,
        # so a large data file is not read whole.
        first_line=""
        { IFS= read -r -n 256 first_line || true; } <"$file" 2>/dev/null
        if [[ "$first_line" == '#!'* && ! "$first_line" =~ $RUST_ATTRIBUTE_RE ]]; then
            CHECKED_SCRIPTS+=("$path")
        fi
    done
    if [[ ${#CHECKED_SCRIPTS[@]} -eq 0 ]]; then
        echo "error: the shellcheck gate found no tracked script; it checks at least itself" >&2
        return 1
    fi
}

# Fails when a script in $2... under the repository $1 holds a disable directive
# with no reason comment after it on the same line (GEDT-AR-00056),
# and names each such line as <path>:<line number>.
# A source directive hides no finding, so it needs no reason.
check_disable_reasons() {
    local repo_dir="$1" path line line_number directive_keys directive_comment failed=0
    shift
    for path in "$@"; do
        line_number=0
        while IFS= read -r line || [[ -n "$line" ]]; do
            line_number=$((line_number + 1))
            [[ "$line" == *shellcheck* && "$line" =~ $SHELLCHECK_DIRECTIVE_RE ]] || continue
            directive_keys="${BASH_REMATCH[2]}"
            directive_comment="${BASH_REMATCH[3]:-}"
            [[ "$directive_keys" =~ $DISABLE_KEY_RE ]] || continue
            if [[ ! "$directive_comment" =~ $REASON_COMMENT_RE ]]; then
                echo "error: $path:$line_number: the disable directive needs a reason comment" \
                    "after it on the same line (GEDT-AR-00056)" >&2
                failed=1
            fi
        done <"$repo_dir/$path"
    done
    return "$failed"
}

# Runs the gate over the repository of container/tool_image.sh.
# Needs ENGINE, USER_ARGS, and TOOL_IMAGE from container/tool_image.sh,
# whose ensure_tool_image has checked the image.
# Shellcheck runs at the default settings (GEDT-AR-00055):
# no option changes the checks or the severity,
# and --norc keeps any .shellcheckrc of the repository or the image from changing them.
# All scripts go to one run,
# so a script that sources another tracked script finds it among the inputs,
# and shellcheck raises no SC1091 without -x.
# The container has no network and only the read-only repository mount,
# so shellcheck writes nothing to the host (GEDT-AR-00043).
# "--" ends the engine options and the shellcheck options,
# so no path is ever parsed as an option.
run_shellcheck_gate() {
    list_checked_scripts "$TOOL_IMAGE_REPO_DIR" || return 1
    check_disable_reasons "$TOOL_IMAGE_REPO_DIR" "${CHECKED_SCRIPTS[@]}" || return 1
    echo "Shellcheck gate: ${#CHECKED_SCRIPTS[@]} scripts" >&2
    "$ENGINE" run --rm --network none \
        "${USER_ARGS[@]}" \
        -v "$TOOL_IMAGE_REPO_DIR":/repo:ro,z \
        -w /repo \
        -- \
        "$TOOL_IMAGE" \
        "$SHELLCHECK_BIN" --norc -- "${CHECKED_SCRIPTS[@]}" || {
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
