#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$ROOT_DIR/scripts/ci/check_docs_contract_test_file_budget.sh"

if [ ! -x "$CHECKER" ]; then
  echo "expected docs-contract test file budget checker to be executable: $CHECKER" >&2
  exit 1
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

PASS_THRESHOLD_FILE="$TMP_DIR/pass-threshold.env"
cat >"$PASS_THRESHOLD_FILE" <<'EOF'
DOCS_CONTRACT_TEST_FILE_MAX=9999
EOF

PASS_REPORT="$TMP_DIR/pass-report.json"
pass_output="$(
  bash "$CHECKER" \
    --repo-root "$ROOT_DIR" \
    --threshold-file "$PASS_THRESHOLD_FILE" \
    --output-json "$PASS_REPORT"
)"

if ! printf '%s\n' "$pass_output" | grep -q '^status=ok$'; then
  echo "expected status=ok when docs-contract test file count is within budget" >&2
  exit 1
fi
if ! printf '%s\n' "$pass_output" | grep -q '^final_decision=GO$'; then
  echo "expected final_decision=GO for within-budget path" >&2
  exit 1
fi
if ! printf '%s\n' "$pass_output" | grep -q '^reason_codes=none$'; then
  echo "expected reason_codes=none for within-budget path" >&2
  exit 1
fi
if ! printf '%s\n' "$pass_output" | grep -Eq '^docs_contract_test_file_count=[0-9]+$'; then
  echo "expected docs_contract_test_file_count metric marker" >&2
  exit 1
fi
if ! printf '%s\n' "$pass_output" | grep -Eq '^docs_contract_test_file_max=[0-9]+$'; then
  echo "expected docs_contract_test_file_max metric marker" >&2
  exit 1
fi

FAIL_THRESHOLD_FILE="$TMP_DIR/fail-threshold.env"
cat >"$FAIL_THRESHOLD_FILE" <<'EOF'
DOCS_CONTRACT_TEST_FILE_MAX=1
EOF

set +e
fail_output="$(
  bash "$CHECKER" \
    --repo-root "$ROOT_DIR" \
    --threshold-file "$FAIL_THRESHOLD_FILE" \
    --output-json "$TMP_DIR/fail-report.json" 2>&1
)"
fail_exit=$?
set -e

if [ "$fail_exit" -eq 0 ]; then
  echo "expected checker to fail when docs-contract test file count exceeds budget" >&2
  exit 1
fi
if ! printf '%s\n' "$fail_output" | grep -q '^status=fail$'; then
  echo "expected status=fail when docs-contract file budget is exceeded" >&2
  exit 1
fi
if ! printf '%s\n' "$fail_output" | grep -q '^final_decision=NO-GO$'; then
  echo "expected final_decision=NO-GO when docs-contract file budget is exceeded" >&2
  exit 1
fi
if ! printf '%s\n' "$fail_output" | grep -q '^reason_codes=docs_contract_test_file_budget_exceeded$'; then
  echo "expected deterministic docs-contract file-budget exceeded reason code marker" >&2
  exit 1
fi

INVALID_THRESHOLD_FILE="$TMP_DIR/invalid-threshold.env"
cat >"$INVALID_THRESHOLD_FILE" <<'EOF'
NOT_THE_EXPECTED_KEY=10
EOF

set +e
invalid_output="$(
  bash "$CHECKER" \
    --repo-root "$ROOT_DIR" \
    --threshold-file "$INVALID_THRESHOLD_FILE" \
    --output-json "$TMP_DIR/invalid-report.json" 2>&1
)"
invalid_exit=$?
set -e

if [ "$invalid_exit" -eq 0 ]; then
  echo "expected checker to fail when threshold file misses required key" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_output" | grep -q 'docs_contract_test_file_budget_threshold_key_missing'; then
  echo "expected deterministic missing-threshold-key reason marker" >&2
  exit 1
fi

echo "docs-contract test file budget checker tests passed."
