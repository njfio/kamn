#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LANE_SCRIPT="$ROOT_DIR/scripts/sdk/run_localhost_signed_demo_contract_lane.sh"
DEMO_SCRIPT="$ROOT_DIR/scripts/sdk/run_localhost_signed_demo.sh"
INTEGRATION_CONTRACT_LANE="$ROOT_DIR/scripts/sdk/run_localhost_signed_integration_contract_lane.sh"
INTEGRATION_POLICY="$ROOT_DIR/scripts/sdk/check_localhost_signed_integration_evidence_policy.sh"
README_FILE="$ROOT_DIR/README.md"
DEVNET_DOC="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$LANE_SCRIPT" ]; then
  echo "expected localhost signed demo contract lane script to be executable" >&2
  exit 1
fi

if [ ! -x "$DEMO_SCRIPT" ]; then
  echo "expected localhost signed demo script to be executable" >&2
  exit 1
fi

if [ ! -x "$INTEGRATION_CONTRACT_LANE" ]; then
  echo "expected localhost signed integration contract lane script to be executable" >&2
  exit 1
fi

if [ ! -x "$INTEGRATION_POLICY" ]; then
  echo "expected localhost signed integration evidence policy checker to be executable" >&2
  exit 1
fi

if ! grep -q "run_localhost_signed_demo_contract_lane.sh" "$README_FILE"; then
  echo "expected README to reference localhost signed demo contract lane command" >&2
  exit 1
fi

if ! grep -q "/tmp/localhost-signed-demo-contract-report.json" "$README_FILE"; then
  echo "expected README to reference localhost signed demo contract lane report path" >&2
  exit 1
fi

if ! grep -q "run_localhost_signed_demo_contract_lane.sh" "$DEVNET_DOC"; then
  echo "expected Kolme devnet ops doc to reference localhost signed demo contract lane command" >&2
  exit 1
fi

if ! grep -q "/tmp/localhost-signed-demo-contract-report.json" "$DEVNET_DOC"; then
  echo "expected Kolme devnet ops doc to reference localhost signed demo contract lane report path" >&2
  exit 1
fi

report_file="$TMP_DIR/localhost-signed-demo-contract-report.json"
lane_output="$(bash "$LANE_SCRIPT" --output-json "$report_file")"

for marker in \
  "localhost_signed_demo_status=pass" \
  "localhost_signed_integration_status=pass" \
  "localhost signed demo contract lane tests passed." \
  "localhost_signed_demo_contract_report=$report_file"; do
  if ! printf '%s\n' "$lane_output" | grep -q "$marker"; then
    echo "expected localhost signed demo contract lane marker '$marker'" >&2
    exit 1
  fi
done

python3 - "$report_file" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
assert report["schema_version"] == "kamn.sdk.localhost-signed.demo-contract.v1"
assert report["status"] == "pass"
assert report["reason_codes"] == ["none"]
assert report["demo_artifact_schema"] == "kamn.sdk.localhost-signed.demo-receipt-artifact.v1"
assert report["integration_report_schema"] == "kamn.sdk.localhost-signed.integration-contract.v1"
assert report["demo_status"] == "pass"
assert report["integration_status"] == "pass"
assert report["budget_status"] == "within_budget"
PY

echo "localhost signed demo contract lane script tests passed."
