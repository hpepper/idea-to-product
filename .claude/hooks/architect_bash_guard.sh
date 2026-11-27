#!/usr/bin/env bash
# PreToolUse guard for the architect agent's Bash tool.
# The architect needs Bash only to read the committed requirements file,
# to decide whether a requirement ID may change in place.
# See the Identifier changes rules of the requirements-writing skill.
# Exit code 2 blocks the command; any other non-zero exit code would let the command run,
# so every failure path exits 2 to fail closed.

if ! command -v jq >/dev/null 2>&1; then
  echo "Blocked: jq is not installed, so the architect Bash guard cannot check the command." >&2
  exit 2
fi

if ! requested_command=$(jq -r '.tool_input.command // empty' 2>/dev/null); then
  echo "Blocked: the architect Bash guard could not parse the hook input." >&2
  exit 2
fi

# Bash regex anchors match the whole string, so a newline or a shell operator fails the match.
allowed_pattern='^git show HEAD:[A-Za-z0-9_][A-Za-z0-9_/-]*/docs/REQUIREMENTS\.md$'

if [[ "$requested_command" =~ $allowed_pattern ]]; then
  exit 0
fi

echo "Blocked: the architect may only run 'git show HEAD:<app>/docs/REQUIREMENTS.md'." >&2
exit 2
