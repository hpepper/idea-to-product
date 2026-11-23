#!/usr/bin/env bash
# Fails when the cargo audit and cargo deny advisory exception lists differ
# (GEDT-AR-00025).
# `cargo audit` reads .cargo/audit.toml, which has no reason field,
# while `cargo deny` reads deny.toml, which requires one.
# Mirroring the lists keeps every exception documented with a reason in deny.toml.
# Uses only bash, grep, sed, and sort,
# so it runs in the pinned Rust image without extra tools.
#
#   ./check_advisory_ignores.sh    run from any directory;
#   checks the files next to the script
#
# The check is an allow-list parse of a TOML subset, not a TOML parser.
# Any form outside the subset fails the check, even when the TOML is valid (DEF-04).
# Accepted forms, after comment stripping:
# - deny.toml: no single quote anywhere, so literal strings cannot carry an ID.
# - deny.toml: no backslash anywhere, so escape sequences cannot spell a key or an ID.
# - deny.toml: every line that starts with `[` is a bare table header such as `[bans]`,
#   with no spaces, quotes, or dots inside the brackets,
#   and no `[[` array of tables (DEF-06).
# - deny.toml: no `advisories` dotted key or inline table; only the `[advisories]` header.
# - deny.toml [advisories]: one-line scalar settings, `ignore = []`,
#   or a multi-line `ignore = [` array closed by `]`,
#   with one `{ id = "RUSTSEC-...", reason = "..." },` entry per line.
# - deny.toml: each reason holds a non-whitespace character and no quote or backslash.
# - deny.toml: no RUSTSEC ID outside those entry lines.
# - .cargo/audit.toml: only `[advisories]`,
#   and an `ignore` array of double-quoted RUSTSEC IDs.
# Comment stripping cuts at the first '#', also inside a string; the cut line then fails.
set -euo pipefail

cd "$(dirname "$0")"

DENY_TOML="deny.toml"
AUDIT_TOML=".cargo/audit.toml"
ID_RE='RUSTSEC-[0-9]{4}-[0-9]{4}'
SP='[[:space:]]*'
DENY_ENTRY_RE="^${SP}\{${SP}id${SP}=${SP}\"(${ID_RE})\"${SP},${SP}reason${SP}=${SP}\"[^\"\\\\]*[^\"\\\\[:space:]][^\"\\\\]*\"${SP}\}${SP},?${SP}$"
DENY_SCALAR_RE="^${SP}[A-Za-z0-9_-]+${SP}=${SP}(\"[^\"\\\\]*\"|[0-9]+|true|false)${SP}$"
QUOTED_ID_RE="\"${ID_RE}\""
AUDIT_IDS_RE="(${SP}${QUOTED_ID_RE}${SP},)*(${SP}${QUOTED_ID_RE}${SP},?)?${SP}"
AUDIT_IGNORE_LINE_RE="^${SP}ignore${SP}=${SP}\[${AUDIT_IDS_RE}(\])?${SP}$"
AUDIT_ID_LINE_RE="^${AUDIT_IDS_RE}$"
# Any line opening with '[' must be a bare header; cargo-deny also reads `[ advisories ]`,
# `["advisories"]`, and `[[advisories.ignore]]` as the advisories table (DEF-06).
BRACKET_START_RE="^${SP}\["
HEADER_RE="^${SP}\[[A-Za-z0-9_-]+\]${SP}$"
OPEN_RE="^${SP}ignore${SP}=${SP}\[${SP}$"
EMPTY_RE="^${SP}ignore${SP}=${SP}\[${SP}\]${SP}$"
CLOSE_RE="^${SP}\]${SP}$"
# A dotted key or inline table outside [advisories], such as advisories.ignore = [...].
ADVISORIES_KEY_RE="^${SP}\"?advisories\"?${SP}[.=]"

fail() { echo "error: $*" >&2; exit 1; }

[[ -f "$DENY_TOML" ]] || fail "$DENY_TOML not found"
[[ -f "$AUDIT_TOML" ]] || fail "$AUDIT_TOML not found"
# cargo audit also reads $CARGO_HOME/audit.toml,
# which would add an uncommitted exception list.
[[ ! -e .cargo-home/audit.toml ]] || fail ".cargo-home/audit.toml must not exist"

strip_comments() { sed -e 's/#.*//' -e '/^[[:space:]]*$/d' "$1"; }

# The read loops below also accept a last line without a trailing newline.
# GNU sed keeps that line unterminated, and a bare `read` would drop it unchecked.

# Prints the advisory IDs of deny.toml, one per line, or fails on any unaccepted form.
parse_deny() {
    local line section="" state=none
    while IFS= read -r line || [[ -n "$line" ]]; do
        [[ "$line" != *"'"* ]] || fail "$DENY_TOML single-quoted strings are not accepted: $line"
        [[ "$line" != *\\* ]] || fail "$DENY_TOML backslashes are not accepted: $line"
        if [[ "$line" =~ $BRACKET_START_RE ]]; then
            [[ "$line" =~ $HEADER_RE ]] \
                || fail "$DENY_TOML table header must be a bare [name]: $line"
            [[ "$state" != open ]] || fail "$DENY_TOML ignore array is not closed"
            section="$(sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//' <<<"$line")"
            if [[ "$section" == "[advisories]" ]]; then
                [[ "$state" == none ]] || fail "$DENY_TOML has more than one [advisories] table"
                state=seen
            fi
            continue
        fi
        [[ ! "$line" =~ $ADVISORIES_KEY_RE ]] \
            || fail "$DENY_TOML advisories settings belong in the [advisories] table: $line"
        if [[ "$section" != "[advisories]" ]]; then
            [[ ! "$line" =~ RUSTSEC ]] || fail "$DENY_TOML advisory ID outside advisories.ignore: $line"
            continue
        fi
        if [[ "$state" == open ]]; then
            if [[ "$line" =~ $DENY_ENTRY_RE ]]; then
                echo "${BASH_REMATCH[1]}"
            elif [[ "$line" =~ $CLOSE_RE ]]; then
                state=closed
            else
                fail "$DENY_TOML exception needs { id = \"RUSTSEC-...\", reason = \"...\" }: $line"
            fi
        elif [[ "$line" =~ $EMPTY_RE || "$line" =~ $OPEN_RE ]]; then
            [[ "$state" == seen ]] || fail "$DENY_TOML has more than one advisories.ignore"
            if [[ "$line" =~ $OPEN_RE ]]; then state=open; else state=closed; fi
        elif [[ "$line" =~ ^${SP}ignore${SP}= ]]; then
            fail "$DENY_TOML advisories.ignore must be [] or a multi-line array: $line"
        elif [[ "$line" =~ $DENY_SCALAR_RE ]]; then
            [[ ! "$line" =~ RUSTSEC ]] || fail "$DENY_TOML advisory ID outside advisories.ignore: $line"
        else
            fail "$DENY_TOML unrecognised [advisories] line: $line"
        fi
    done < <(strip_comments "$DENY_TOML")
    [[ "$state" != open ]] || fail "$DENY_TOML ignore array is not closed"
}

# Prints the advisory IDs of .cargo/audit.toml, one per line,
# or fails on any other content.
# The file may only hold [advisories] ignore, so it cannot lower the audit policy,
# for example through severity_threshold.
parse_audit() {
    local line state=start
    while IFS= read -r line || [[ -n "$line" ]]; do
        case "$state" in
            start)
                [[ "$line" =~ ^${SP}\[advisories\]${SP}$ ]] \
                    || fail "$AUDIT_TOML must start with [advisories]: $line"
                state=header
                ;;
            header)
                [[ "$line" =~ $AUDIT_IGNORE_LINE_RE ]] \
                    || fail "$AUDIT_TOML may only contain ignore = [\"RUSTSEC-...\"]: $line"
                if [[ -n "${BASH_REMATCH[3]}" ]]; then state=closed; else state=open; fi
                ;;
            open)
                if [[ "$line" =~ $CLOSE_RE ]]; then
                    state=closed
                    continue
                fi
                [[ "$line" =~ $AUDIT_ID_LINE_RE ]] \
                    || fail "$AUDIT_TOML ignore entry is not a double-quoted RUSTSEC ID: $line"
                ;;
            closed)
                fail "$AUDIT_TOML may only contain [advisories] ignore = [...]: $line"
                ;;
        esac
        grep -oE "$ID_RE" <<<"$line" || true
    done < <(strip_comments "$AUDIT_TOML")
    [[ "$state" == closed ]] || fail "$AUDIT_TOML needs a closed [advisories] ignore array"
}

# Process substitution would hide a parse failure, so capture each list in its own step.
deny_ids="$(parse_deny)" || exit 1
audit_ids="$(parse_audit)" || exit 1
deny_ids="$(sort -u <<<"$deny_ids")"
audit_ids="$(sort -u <<<"$audit_ids")"
if [[ "$deny_ids" != "$audit_ids" ]]; then
    echo "error: advisory ignores differ between $DENY_TOML and $AUDIT_TOML" >&2
    diff <(echo "$deny_ids") <(echo "$audit_ids") >&2 || true
    exit 1
fi
echo "Advisory ignore lists match ($DENY_TOML, $AUDIT_TOML)."
