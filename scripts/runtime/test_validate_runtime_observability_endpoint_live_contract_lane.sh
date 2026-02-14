#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/runtime/validate_runtime_observability_endpoint_live_contract_lane.sh"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_runtime_observability_endpoint_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_runtime_observability_endpoint_live_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected runtime observability endpoint contract lane script to be executable" >&2
  exit 1
fi
if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected runtime observability endpoint validation script to be executable" >&2
  exit 1
fi
if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected runtime observability endpoint policy checker script to be executable" >&2
  exit 1
fi

lane_report="$TMP_DIR/runtime-observability-endpoint-contract-lane-report.json"
policy_report="$TMP_DIR/runtime-observability-endpoint-policy-report.json"

lane_output="$(
  bash "$CONTRACT_LANE" \
    --output-json "$lane_report" \
    --policy-output-json "$policy_report"
)"
if ! printf '%s\n' "$lane_output" | grep -q '^status=pass$'; then
  echo "expected runtime observability endpoint contract lane status=pass marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^final_decision=GO$'; then
  echo "expected runtime observability endpoint contract lane final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^runtime_observability_policy_status=verified$'; then
  echo "expected runtime observability endpoint contract lane policy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^runtime_observability_contract_lane_status=verified$'; then
  echo "expected runtime observability endpoint contract lane status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^fail_closed_reason_code=runtime_observability_policy_final_decision_mismatch$'; then
  echo "expected runtime observability endpoint contract lane fail-closed reason marker" >&2
  exit 1
fi

python3 - "$lane_report" "$policy_report" <<'PY'
import json
import pathlib
import sys

lane_payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if lane_payload.get("schema_version") != "kamn.runtime.observability-endpoint-live-contract-lane-report.v1":
    raise SystemExit("unexpected runtime observability endpoint contract lane report schema")
if lane_payload.get("status") != "pass":
    raise SystemExit("expected contract lane status=pass")
if lane_payload.get("final_decision") != "GO":
    raise SystemExit("expected contract lane final_decision=GO")
if lane_payload.get("runtime_observability_policy_status") != "verified":
    raise SystemExit("expected runtime_observability_policy_status=verified")
if lane_payload.get("runtime_observability_contract_lane_status") != "verified":
    raise SystemExit("expected runtime_observability_contract_lane_status=verified")
if lane_payload.get("fail_closed_reason_code") != "runtime_observability_policy_final_decision_mismatch":
    raise SystemExit("expected deterministic fail-closed reason code marker")
if lane_payload.get("performance_budget_status") != "verified":
    raise SystemExit("expected performance_budget_status=verified")

policy_payload = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
if policy_payload.get("schema_version") != "kamn.runtime.observability-endpoint-live-policy-report.v1":
    raise SystemExit("unexpected runtime observability endpoint policy report schema")
if policy_payload.get("final_decision") != "GO":
    raise SystemExit("expected policy final_decision=GO")
if policy_payload.get("runtime_observability_policy_status") != "verified":
    raise SystemExit("expected runtime_observability_policy_status=verified in policy report")
PY

if ! grep -q "check_runtime_observability_endpoint_live_policy.sh" "$CONTRACT_LANE"; then
  echo "expected runtime observability endpoint contract lane to compose policy checker" >&2
  exit 1
fi
if ! grep -q "validate_runtime_observability_endpoint_live.sh" "$CONTRACT_LANE"; then
  echo "expected runtime observability endpoint contract lane to compose validation lane" >&2
  exit 1
fi

echo "runtime observability endpoint contract lane tests passed."
