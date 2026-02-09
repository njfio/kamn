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
bash "$GENERATOR" \
  --output-file "$bundle_file" \
  --lane contract \
  --matrix-report-file "$matrix_report" \
  --compatibility-suite-status pass \
  --runtime-budget-status within \
  --ci-fast-gate PASS >/dev/null

tampered_bundle="$TMP_DIR/sdk-schema-compatibility-tampered-reason-codes.json"
cp "$bundle_file" "$tampered_bundle"
python3 - "$tampered_bundle" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text())
payload["reason_codes"] = ["compatibility_suite_failed"]
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n")
PY

set +e
tampered_output="$(bash "$CHECKER" --bundle-file "$tampered_bundle" 2>&1)"
tampered_code=$?
set -e

if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered reason_codes payload to fail sdk schema compatibility policy validation" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_output" | grep -q "reason_codes mismatch"; then
  echo "expected reason_codes mismatch failure for tampered sdk schema compatibility payload" >&2
  exit 1
fi

# Regression: #937
if ! printf '%s\n' "$tampered_output" | grep -q "expected reason_codes"; then
  echo "expected explicit reason code mismatch output for sdk schema compatibility regression path" >&2
  exit 1
fi

echo "sdk schema compatibility reason-code policy checker tests passed."
