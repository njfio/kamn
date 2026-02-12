#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/ci/check_script_duplication_budget.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$SCRIPT" ]; then
  echo "expected script duplication budget checker to be executable" >&2
  exit 1
fi

SCRIPTS_ROOT="$TMP_DIR/scripts"
mkdir -p "$SCRIPTS_ROOT/a" "$SCRIPTS_ROOT/b" "$SCRIPTS_ROOT/c" "$SCRIPTS_ROOT/d"
cat >"$SCRIPTS_ROOT/a/run_alpha.sh" <<'EOF_SCRIPT'
#!/usr/bin/env bash
echo "alpha"
EOF_SCRIPT
cat >"$SCRIPTS_ROOT/b/run_beta.sh" <<'EOF_SCRIPT'
#!/usr/bin/env bash
echo "beta"
EOF_SCRIPT
ln -s ../a/run_alpha.sh "$SCRIPTS_ROOT/c/run_alpha_dispatch.sh"

PASS_BUDGET="$TMP_DIR/pass-budget.env"
cat >"$PASS_BUDGET" <<'EOF_BUDGET'
SCRIPT_COUNT_MAX=5
SHELL_LINE_TOTAL_MAX=20
DUPLICATE_BASENAME_MAX=0
DUPLICATE_CONTENT_MAX=0
EOF_BUDGET

BASELINE_FILE="$TMP_DIR/baseline.env"
cat >"$BASELINE_FILE" <<'EOF_BASELINE'
SCRIPT_COUNT_BASELINE=1
SHELL_LINE_TOTAL_BASELINE=4
DUPLICATE_BASENAME_BASELINE=0
DUPLICATE_CONTENT_BASELINE=0
EOF_BASELINE

pass_output="$(
  bash "$SCRIPT" \
    --scripts-root "$SCRIPTS_ROOT" \
    --budget-file "$PASS_BUDGET" \
    --baseline-file "$BASELINE_FILE"
)"

if ! printf '%s\n' "$pass_output" | grep -q '^status=pass$'; then
  echo "expected pass status for script duplication budget checker pass path" >&2
  exit 1
fi
if ! printf '%s\n' "$pass_output" | grep -q '^violations=none$'; then
  echo "expected no violations on pass path" >&2
  exit 1
fi
if ! printf '%s\n' "$pass_output" | grep -q '^delta_script_count=2$'; then
  echo "expected deterministic script_count delta output on pass path" >&2
  exit 1
fi
if ! printf '%s\n' "$pass_output" | grep -q '^duplicate_content=0$'; then
  echo "expected symlink wrappers to be excluded from duplicate_content metric" >&2
  exit 1
fi
if ! printf '%s\n' "$pass_output" | grep -q '^remediation=none$'; then
  echo "expected remediation marker to be none on pass path" >&2
  exit 1
fi

FAIL_BUDGET="$TMP_DIR/fail-budget.env"
cat >"$FAIL_BUDGET" <<'EOF_BUDGET'
SCRIPT_COUNT_MAX=1
SHELL_LINE_TOTAL_MAX=20
DUPLICATE_BASENAME_MAX=0
DUPLICATE_CONTENT_MAX=0
EOF_BUDGET

set +e
fail_output="$(
  bash "$SCRIPT" \
    --scripts-root "$SCRIPTS_ROOT" \
    --budget-file "$FAIL_BUDGET" \
    --baseline-file "$BASELINE_FILE" \
    --waiver-file "$TMP_DIR/missing-waiver.json" 2>&1
)"
fail_code=$?
set -e

if [ "$fail_code" -eq 0 ]; then
  echo "expected checker to fail when script_count exceeds threshold" >&2
  exit 1
fi
if ! printf '%s\n' "$fail_output" | grep -q '^status=fail$'; then
  echo "expected fail status on threshold exceed path" >&2
  exit 1
fi
if ! printf '%s\n' "$fail_output" | grep -q 'violations=script_count'; then
  echo "expected script_count violation marker" >&2
  exit 1
fi
if ! printf '%s\n' "$fail_output" | grep -q '^remediation=reduce metrics (script_count) under thresholds in '; then
  echo "expected deterministic remediation guidance on fail path" >&2
  exit 1
fi

set +e
missing_baseline_output="$(
  bash "$SCRIPT" \
    --scripts-root "$SCRIPTS_ROOT" \
    --budget-file "$FAIL_BUDGET" \
    --baseline-file "$TMP_DIR/missing-baseline.env" 2>&1
)"
missing_baseline_code=$?
set -e

if [ "$missing_baseline_code" -eq 0 ]; then
  echo "expected checker to fail when baseline file is missing" >&2
  exit 1
fi
if ! printf '%s\n' "$missing_baseline_output" | grep -q '^error=baseline file not found:'; then
  echo "expected explicit missing baseline error output" >&2
  exit 1
fi

WAIVER_FILE="$TMP_DIR/waiver.json"
cat >"$WAIVER_FILE" <<'EOF_WAIVER'
{
  "reason": "temporary migration burst",
  "expires_on": "2099-12-31",
  "allow_metrics": ["script_count"]
}
EOF_WAIVER

waived_output="$(
  bash "$SCRIPT" \
    --scripts-root "$SCRIPTS_ROOT" \
    --budget-file "$FAIL_BUDGET" \
    --baseline-file "$BASELINE_FILE" \
    --waiver-file "$WAIVER_FILE"
)"

if ! printf '%s\n' "$waived_output" | grep -q '^status=pass$'; then
  echo "expected waiver path to pass when script_count is explicitly allowed" >&2
  exit 1
fi
if ! printf '%s\n' "$waived_output" | grep -q '^waived=script_count$'; then
  echo "expected waived metric marker for script_count" >&2
  exit 1
fi

cat >"$WAIVER_FILE" <<'EOF_WAIVER'
{
  "reason": "expired exception",
  "expires_on": "2020-01-01",
  "allow_metrics": ["script_count"]
}
EOF_WAIVER

set +e
expired_waiver_output="$(
  bash "$SCRIPT" \
    --scripts-root "$SCRIPTS_ROOT" \
    --budget-file "$FAIL_BUDGET" \
    --baseline-file "$BASELINE_FILE" \
    --waiver-file "$WAIVER_FILE" \
    --today "2026-02-10" 2>&1
)"
expired_waiver_code=$?
set -e

if [ "$expired_waiver_code" -eq 0 ]; then
  echo "expected expired waiver path to fail closed" >&2
  exit 1
fi
if ! printf '%s\n' "$expired_waiver_output" | grep -q 'waiver_error=waiver has expired'; then
  echo "expected explicit expired waiver error output" >&2
  exit 1
fi

cat >"$SCRIPTS_ROOT/d/run_alpha_copy.sh" <<'EOF_SCRIPT'
#!/usr/bin/env bash
echo "alpha"
EOF_SCRIPT

DUPLICATE_CONTENT_BUDGET="$TMP_DIR/duplicate-content-budget.env"
cat >"$DUPLICATE_CONTENT_BUDGET" <<'EOF_BUDGET'
SCRIPT_COUNT_MAX=20
SHELL_LINE_TOTAL_MAX=200
DUPLICATE_BASENAME_MAX=0
DUPLICATE_CONTENT_MAX=0
EOF_BUDGET

set +e
duplicate_content_output="$(
  bash "$SCRIPT" \
    --scripts-root "$SCRIPTS_ROOT" \
    --budget-file "$DUPLICATE_CONTENT_BUDGET" \
    --baseline-file "$BASELINE_FILE" \
    --waiver-file "$TMP_DIR/missing-waiver.json" 2>&1
)"
duplicate_content_code=$?
set -e

if [ "$duplicate_content_code" -eq 0 ]; then
  echo "expected checker to fail when regular files duplicate content" >&2
  exit 1
fi
if ! printf '%s\n' "$duplicate_content_output" | grep -q 'violations=duplicate_content'; then
  echo "expected duplicate_content violation marker for regular-file duplicate" >&2
  exit 1
fi

echo "script duplication budget checker tests passed."
