#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/sdk/run_sdk_parity_matrix.sh"
FIXTURE="$ROOT_DIR/fixtures/sdk_parity/register_validation_cases.json"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

PASS_REPORT="$TMP_DIR/pass-report.json"
bash "$SCRIPT" --fixture "$FIXTURE" --output-json "$PASS_REPORT" >"$TMP_DIR/pass.out"
grep -q '"status": "pass"' "$PASS_REPORT"

MISMATCH_FIXTURE="$TMP_DIR/mismatch.json"
cat >"$MISMATCH_FIXTURE" <<'JSON'
{
  "cases": [
    {
      "id": "mismatch-empty-capability",
      "agent_type": "autonomous",
      "model_family": "claude-4",
      "capabilities": ["text", ""],
      "expected": {
        "status": "ok"
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
  echo "expected parity matrix to fail for mismatch fixture" >&2
  exit 1
fi

# Regression: #583
if ! grep -q "status=fail" "$TMP_DIR/fail.out"; then
  echo "expected mismatch failure status output" >&2
  exit 1
fi

echo "sdk parity matrix tests passed."
