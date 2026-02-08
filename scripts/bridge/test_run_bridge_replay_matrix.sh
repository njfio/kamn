#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/bridge/run_bridge_replay_matrix.sh"
FIXTURE="$ROOT_DIR/fixtures/bridge_replay/replay_validation_cases.json"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

PASS_REPORT="$TMP_DIR/pass-report.json"
bash "$SCRIPT" --fixture "$FIXTURE" --output-json "$PASS_REPORT" >"$TMP_DIR/pass.out"
grep -q '"status": "pass"' "$PASS_REPORT"

INVALID_FIXTURE="$TMP_DIR/invalid.json"
cat >"$INVALID_FIXTURE" <<'JSON'
{
  "cases": {}
}
JSON

set +e
bash "$SCRIPT" --fixture "$INVALID_FIXTURE" --output-json "$TMP_DIR/invalid-report.json" >"$TMP_DIR/invalid.out" 2>&1
invalid_code=$?
set -e

if [ "$invalid_code" -eq 0 ]; then
  echo "expected bridge replay matrix to fail for invalid fixture schema" >&2
  exit 1
fi

if ! grep -q "status=fail; reason=invalid-fixture-cases" "$TMP_DIR/invalid.out"; then
  echo "expected invalid fixture schema failure status output" >&2
  exit 1
fi

MISMATCH_FIXTURE="$TMP_DIR/mismatch.json"
cat >"$MISMATCH_FIXTURE" <<'JSON'
{
  "cases": [
    {
      "id": "replay-nonexistent-test",
      "suite": "telegram_bridge",
      "test_name": "does_not_exist",
      "class": "duplicate",
      "expected": {
        "status": "pass"
      }
    }
  ]
}
JSON

set +e
bash "$SCRIPT" --fixture "$MISMATCH_FIXTURE" --output-json "$TMP_DIR/fail-report.json" >"$TMP_DIR/fail.out" 2>&1
fail_code=$?
set -e

if [ "$fail_code" -eq 0 ]; then
  echo "expected bridge replay matrix to fail for mismatch fixture" >&2
  exit 1
fi

# Regression: #587
if ! grep -q "status=fail" "$TMP_DIR/fail.out"; then
  echo "expected mismatch failure status output" >&2
  exit 1
fi

echo "bridge replay matrix tests passed."
