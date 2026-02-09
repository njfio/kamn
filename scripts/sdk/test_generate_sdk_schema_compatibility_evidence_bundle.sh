#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
MATRIX_RUNNER="$ROOT_DIR/scripts/sdk/run_sdk_parity_matrix.sh"
GENERATOR="$ROOT_DIR/scripts/sdk/generate_sdk_schema_compatibility_evidence_bundle.sh"
CHECKER="$ROOT_DIR/scripts/sdk/check_sdk_schema_compatibility_policy.sh"
FIXTURE="$ROOT_DIR/fixtures/sdk_parity/register_validation_cases.json"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$MATRIX_RUNNER" ]; then
  echo "expected sdk parity matrix runner to be executable" >&2
  exit 1
fi

if [ ! -x "$GENERATOR" ]; then
  echo "expected sdk schema compatibility evidence generator to be executable" >&2
  exit 1
fi

if [ ! -x "$CHECKER" ]; then
  echo "expected sdk schema compatibility policy checker to be executable" >&2
  exit 1
fi

matrix_report="$TMP_DIR/sdk-parity-matrix-report.json"
bash "$MATRIX_RUNNER" --fixture "$FIXTURE" --output-json "$matrix_report" >/dev/null

bundle_file="$TMP_DIR/sdk-schema-compatibility-go.json"
generator_output="$(
  bash "$GENERATOR" \
    --output-file "$bundle_file" \
    --lane contract \
    --matrix-report-file "$matrix_report" \
    --compatibility-suite-status pass \
    --runtime-budget-status within \
    --ci-fast-gate PASS
)"

if ! printf '%s\n' "$generator_output" | grep -q "^status=generated$"; then
  echo "expected generated status for sdk schema compatibility go case" >&2
  exit 1
fi

if ! printf '%s\n' "$generator_output" | grep -q "^reason_key=sdk_schema_compatibility_reason_codes:GO:v1$"; then
  echo "expected GO reason key for sdk schema compatibility go case" >&2
  exit 1
fi

if ! printf '%s\n' "$generator_output" | grep -q "^final_decision=GO$"; then
  echo "expected GO final decision for sdk schema compatibility go case" >&2
  exit 1
fi

python3 - "$bundle_file" <<'PY'
import json
import pathlib
import sys

bundle_path = pathlib.Path(sys.argv[1])
payload = json.loads(bundle_path.read_text(encoding="utf-8"))

assert payload["schema_version"] == "kamn.sdk.schema-compatibility-evidence.v1"
assert payload["reason_key"] == "sdk_schema_compatibility_reason_codes:GO:v1"
assert payload["matrix_summary"]["schema_version"] == "kamn.sdk.parity.matrix.v1"
assert payload["reason_codes"] == []
assert payload["final_decision"] == "GO"
assert payload["policy_checks"]["matrix_passed"] is True
assert payload["policy_checks"]["compatibility_suite_passed"] is True
assert payload["policy_checks"]["runtime_budget_within"] is True
assert payload["policy_checks"]["ci_fast_gate_passed"] is True
PY

checker_output="$(bash "$CHECKER" --bundle-file "$bundle_file")"
if ! printf '%s\n' "$checker_output" | grep -q "^final_decision=GO$"; then
  echo "expected checker GO final decision for sdk schema compatibility go case" >&2
  exit 1
fi

if ! printf '%s\n' "$checker_output" | grep -q "^failed_checks=none$"; then
  echo "expected no failed checks for sdk schema compatibility go case" >&2
  exit 1
fi

no_go_bundle="$TMP_DIR/sdk-schema-compatibility-no-go.json"
no_go_output="$(
  bash "$GENERATOR" \
    --output-file "$no_go_bundle" \
    --lane contract \
    --matrix-report-file "$matrix_report" \
    --compatibility-suite-status fail \
    --runtime-budget-status exceeded \
    --ci-fast-gate FAIL
)"

if ! printf '%s\n' "$no_go_output" | grep -q "^final_decision=NO-GO$"; then
  echo "expected NO-GO decision for sdk schema compatibility no-go case" >&2
  exit 1
fi

if ! printf '%s\n' "$no_go_output" | grep -q "^reason_key=sdk_schema_compatibility_reason_codes:NO-GO:v1$"; then
  echo "expected NO-GO reason key for sdk schema compatibility no-go case" >&2
  exit 1
fi

no_go_checker_output="$(bash "$CHECKER" --bundle-file "$no_go_bundle")"
if ! printf '%s\n' "$no_go_checker_output" | grep -q "^final_decision=NO-GO$"; then
  echo "expected checker NO-GO decision for sdk schema compatibility no-go case" >&2
  exit 1
fi

if ! printf '%s\n' "$no_go_checker_output" | grep -q "compatibility_suite_failed"; then
  echo "expected failed checks to include compatibility suite failure" >&2
  exit 1
fi

if ! printf '%s\n' "$no_go_checker_output" | grep -q "runtime_budget_exceeded"; then
  echo "expected failed checks to include runtime budget failure" >&2
  exit 1
fi

echo "sdk schema compatibility evidence bundle generator tests passed."
