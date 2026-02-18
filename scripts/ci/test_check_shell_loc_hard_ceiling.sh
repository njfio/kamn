#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$ROOT_DIR/scripts/ci/check_shell_loc_hard_ceiling.sh"

if [ ! -x "$CHECKER" ]; then
  echo "expected shell LOC hard-ceiling checker to be executable: $CHECKER" >&2
  exit 1
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

PASS_CEILING_FILE="$TMP_DIR/pass-ceiling.env"
cat >"$PASS_CEILING_FILE" <<'EOF'
HARD_SHELL_LOC_MAX=999999
EOF

PASS_REPORT="$TMP_DIR/pass-report.json"
pass_output="$(
  bash "$CHECKER" \
    --repo-root "$ROOT_DIR" \
    --ceiling-file "$PASS_CEILING_FILE" \
    --output-json "$PASS_REPORT"
)"

if ! printf '%s\n' "$pass_output" | grep -q '^status=ok$'; then
  echo "expected status=ok for within-ceiling check path" >&2
  exit 1
fi
if ! printf '%s\n' "$pass_output" | grep -q '^final_decision=GO$'; then
  echo "expected final_decision=GO for within-ceiling check path" >&2
  exit 1
fi
if ! printf '%s\n' "$pass_output" | grep -q '^reason_taxonomy_version=kamn.ci.shell-loc-hard-ceiling-reason-taxonomy.v1$'; then
  echo "expected deterministic reason taxonomy marker for within-ceiling path" >&2
  exit 1
fi
if ! printf '%s\n' "$pass_output" | grep -q '^reason_codes=none$'; then
  echo "expected reason_codes=none for within-ceiling path" >&2
  exit 1
fi
if ! printf '%s\n' "$pass_output" | grep -Eq '^shell_line_total=[0-9]+$'; then
  echo "expected shell_line_total metric marker for within-ceiling path" >&2
  exit 1
fi
if ! printf '%s\n' "$pass_output" | grep -Eq '^hard_shell_loc_max=[0-9]+$'; then
  echo "expected hard_shell_loc_max metric marker for within-ceiling path" >&2
  exit 1
fi

FAIL_CEILING_FILE="$TMP_DIR/fail-ceiling.env"
cat >"$FAIL_CEILING_FILE" <<'EOF'
HARD_SHELL_LOC_MAX=1
EOF

set +e
fail_output="$(
  bash "$CHECKER" \
    --repo-root "$ROOT_DIR" \
    --ceiling-file "$FAIL_CEILING_FILE" \
    --output-json "$TMP_DIR/fail-report.json" 2>&1
)"
fail_exit=$?
set -e

if [ "$fail_exit" -eq 0 ]; then
  echo "expected checker to fail when shell LOC exceeds hard ceiling" >&2
  exit 1
fi
if ! printf '%s\n' "$fail_output" | grep -q '^status=fail$'; then
  echo "expected status=fail marker when shell LOC exceeds hard ceiling" >&2
  exit 1
fi
if ! printf '%s\n' "$fail_output" | grep -q '^final_decision=NO-GO$'; then
  echo "expected final_decision=NO-GO marker when shell LOC exceeds hard ceiling" >&2
  exit 1
fi
if ! printf '%s\n' "$fail_output" | grep -q '^reason_codes=shell_loc_hard_ceiling_exceeded$'; then
  echo "expected deterministic shell_loc_hard_ceiling_exceeded reason code marker" >&2
  exit 1
fi

INVALID_CEILING_FILE="$TMP_DIR/invalid-ceiling.env"
cat >"$INVALID_CEILING_FILE" <<'EOF'
NOT_THE_EXPECTED_KEY=100
EOF

set +e
invalid_output="$(
  bash "$CHECKER" \
    --repo-root "$ROOT_DIR" \
    --ceiling-file "$INVALID_CEILING_FILE" \
    --output-json "$TMP_DIR/invalid-report.json" 2>&1
)"
invalid_exit=$?
set -e

if [ "$invalid_exit" -eq 0 ]; then
  echo "expected checker to fail when ceiling file is missing required key" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_output" | grep -q 'shell_loc_hard_ceiling_ceiling_key_missing'; then
  echo "expected deterministic missing-key reason code marker" >&2
  exit 1
fi

echo "shell LOC hard ceiling checker tests passed."

