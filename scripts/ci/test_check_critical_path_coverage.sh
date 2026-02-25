#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"
CHECKER="$ROOT_DIR/scripts/ci/check_critical_path_coverage.py"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

test_harness_require_executable "$CHECKER" "expected critical-path coverage checker to be executable"

CORE_JSON="$TMP_DIR/core-coverage.json"
NODE_JSON="$TMP_DIR/node-coverage.json"
PASS_THRESHOLDS="$TMP_DIR/pass-thresholds.json"
FAIL_THRESHOLDS="$TMP_DIR/fail-thresholds.json"
PASS_REPORT="$TMP_DIR/pass-policy.json"
FAIL_REPORT="$TMP_DIR/fail-policy.json"

bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$CORE_JSON" <<'JSON'
{
  "data": [
    {
      "files": [
        {
          "filename": "/workspace/crates/kamn-core/src/direct_message_crypto.rs",
          "summary": {
            "lines": { "percent": 60.0 },
            "functions": { "percent": 55.0 }
          }
        }
      ]
    }
  ]
}
JSON

bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$NODE_JSON" <<'JSON'
{
  "data": [
    {
      "files": [
        {
          "filename": "/workspace/crates/kamn-node/src/runtime_orchestration.rs",
          "summary": {
            "lines": { "percent": 20.0 },
            "functions": { "percent": 20.0 }
          }
        }
      ]
    }
  ]
}
JSON

bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$PASS_THRESHOLDS" <<'JSON'
{
  "schema_version": "kamn.ci.critical-path-coverage-thresholds.v1",
  "targets": [
    {
      "path": "crates/kamn-core/src/direct_message_crypto.rs",
      "line_percent_min": 50.0,
      "function_percent_min": 50.0
    },
    {
      "path": "crates/kamn-node/src/runtime_orchestration.rs",
      "line_percent_min": 15.0,
      "function_percent_min": 18.0
    }
  ]
}
JSON

bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$FAIL_THRESHOLDS" <<'JSON'
{
  "schema_version": "kamn.ci.critical-path-coverage-thresholds.v1",
  "targets": [
    {
      "path": "crates/kamn-core/src/direct_message_crypto.rs",
      "line_percent_min": 65.0,
      "function_percent_min": 50.0
    },
    {
      "path": "crates/kamn-node/src/service_api_endpoint.rs",
      "line_percent_min": 10.0,
      "function_percent_min": 10.0
    }
  ]
}
JSON

pass_output="$(
  python3 "$CHECKER" \
    --core-coverage-json "$CORE_JSON" \
    --node-coverage-json "$NODE_JSON" \
    --threshold-file "$PASS_THRESHOLDS" \
    --output-json "$PASS_REPORT"
)"
if ! printf '%s\n' "$pass_output" | grep -q '^status=ok$'; then
  echo "expected status=ok for passing critical-path coverage report" >&2
  printf '%s\n' "$pass_output" >&2
  exit 1
fi
if ! printf '%s\n' "$pass_output" | grep -q '^reason_codes_csv=none$'; then
  echo "expected reason_codes_csv=none for passing critical-path coverage report" >&2
  printf '%s\n' "$pass_output" >&2
  exit 1
fi
if ! printf '%s\n' "$pass_output" | grep -q '^failed_targets=0$'; then
  echo "expected failed_targets=0 for passing critical-path coverage report" >&2
  printf '%s\n' "$pass_output" >&2
  exit 1
fi

if python3 "$CHECKER" \
  --core-coverage-json "$CORE_JSON" \
  --node-coverage-json "$NODE_JSON" \
  --threshold-file "$FAIL_THRESHOLDS" \
  --output-json "$FAIL_REPORT" \
  >"$TMP_DIR/fail.out" \
  2>"$TMP_DIR/fail.err"
then
  echo "expected checker to fail for threshold regression/missing target case" >&2
  cat "$TMP_DIR/fail.out" >&2 || true
  cat "$TMP_DIR/fail.err" >&2 || true
  exit 1
fi
if ! grep -q '^reason_codes_csv=critical_path_coverage_target_missing,critical_path_coverage_line_below_threshold$' "$TMP_DIR/fail.out"; then
  echo "expected deterministic target-missing + line-below reason codes" >&2
  cat "$TMP_DIR/fail.out" >&2 || true
  exit 1
fi
python3 - "$FAIL_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.ci.critical-path-coverage-policy-report.v1":
    raise SystemExit("unexpected schema_version")
if payload.get("final_decision") != "NO-GO":
    raise SystemExit("expected final_decision=NO-GO")
if payload.get("failed_targets") != 2:
    raise SystemExit("expected failed_targets=2")
if payload.get("missing_targets") != 1:
    raise SystemExit("expected missing_targets=1")
PY

echo "critical-path coverage policy checker tests passed."
