#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/sdk/run_example_fixture_drift_contract_lane.sh"
SHARED_SCRIPT="$ROOT_DIR/scripts/sdk/example_fixture_drift_contract_lane_contract.py"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$SCRIPT" ]; then
  echo "expected sdk example fixture drift contract lane script to be executable" >&2
  exit 1
fi

if ! grep -q 'example_fixture_drift_contract_lane_contract.py' "$SCRIPT"; then
  echo "expected sdk example fixture drift contract lane wrapper to delegate to shared implementation" >&2
  exit 1
fi

if [ ! -x "$SHARED_SCRIPT" ]; then
  echo "expected shared sdk example fixture drift contract lane implementation to be executable" >&2
  exit 1
fi

output_report="$TMP_DIR/sdk-example-fixture-drift-contract-report.json"
output="$(bash "$SCRIPT" --output-report "$output_report")"

if ! printf '%s\n' "$output" | grep -q "sdk example fixture drift contract lane tests passed."; then
  echo "expected success output from sdk example fixture drift contract lane" >&2
  exit 1
fi

if [ ! -f "$output_report" ]; then
  echo "expected sdk example fixture drift contract lane to emit report file" >&2
  exit 1
fi

if ! grep -q '"schema_version": "kamn.sdk.example-fixture-drift-report.v1"' "$output_report"; then
  echo "expected sdk example fixture drift report schema marker" >&2
  exit 1
fi

if ! grep -q "check_example_fixture_drift.py" "$SHARED_SCRIPT"; then
  echo "expected shared sdk example fixture drift contract lane implementation to run drift checker script" >&2
  exit 1
fi

if ! grep -q "check_example_fixture_drift_policy.sh" "$SHARED_SCRIPT"; then
  echo "expected shared sdk example fixture drift contract lane implementation to run policy checker script" >&2
  exit 1
fi

if ! grep -q "docs/planning/sdk-parity-wave.md" "$SHARED_SCRIPT"; then
  echo "expected shared sdk example fixture drift contract lane implementation to verify planning doc parity references" >&2
  exit 1
fi

echo "sdk example fixture drift contract lane script tests passed."
