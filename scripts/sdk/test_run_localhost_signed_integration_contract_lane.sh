#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LANE_SCRIPT="$ROOT_DIR/scripts/sdk/run_localhost_signed_integration_contract_lane.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$LANE_SCRIPT" ]; then
  echo "expected localhost signed integration contract lane script to be executable" >&2
  exit 1
fi

report_file="$TMP_DIR/localhost-signed-integration-contract.json"
lane_output="$(
  bash "$LANE_SCRIPT" --output-json "$report_file"
)"

required_markers=(
  "localhost_signed_integration_success=pass"
  "localhost_signed_integration_signature_mismatch=pass"
  "localhost_signed_integration_timeout=pass"
  "localhost signed integration contract lane tests passed."
)

for marker in "${required_markers[@]}"; do
  if ! printf '%s\n' "$lane_output" | grep -Fq -- "$marker"; then
    echo "expected localhost signed integration contract lane output marker '$marker'" >&2
    exit 1
  fi
done

python3 - "$report_file" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
assert report["schema_version"] == "kamn.sdk.localhost-signed.integration-contract.v1"
assert report["status"] == "pass"
assert report["success_scenario_status"] == "pass"
assert report["signature_mismatch_scenario_status"] == "pass"
assert report["timeout_scenario_status"] == "pass"
# Regression: #878
assert report["signature_mismatch_reason_code"] == "signature_mismatch_detected"
assert report["timeout_reason_code"] == "listener_timeout_detected"
PY

echo "localhost signed integration contract lane script tests passed."
