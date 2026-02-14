#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/runtime/validate_full_io_scenario_matrix_live_contract_lane.sh"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_full_io_scenario_matrix_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_full_io_scenario_matrix_live_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected full I/O scenario matrix contract lane script to be executable" >&2
  exit 1
fi
if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected full I/O scenario matrix validation script to be executable" >&2
  exit 1
fi
if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected full I/O scenario matrix policy checker script to be executable" >&2
  exit 1
fi

lane_report="$TMP_DIR/full-io-scenario-matrix-contract-lane-report.json"
policy_report="$TMP_DIR/full-io-scenario-matrix-policy-report.json"

lane_output="$(
  bash "$CONTRACT_LANE" \
    --mode dry-run \
    --max-seconds 120 \
    --output-json "$lane_report" \
    --policy-output-json "$policy_report"
)"
if ! printf '%s\n' "$lane_output" | grep -q '^status=pass$'; then
  echo "expected full I/O scenario matrix contract lane status=pass marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^final_decision=GO$'; then
  echo "expected full I/O scenario matrix contract lane final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^lane_mode=dry-run$'; then
  echo "expected full I/O scenario matrix contract lane mode marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^full_io_scenario_matrix_policy_status=verified$'; then
  echo "expected full I/O scenario matrix contract lane policy status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^full_io_scenario_matrix_contract_status=verified$'; then
  echo "expected full I/O scenario matrix contract lane status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^fail_closed_reason_code=full_io_scenario_matrix_policy_multinode_propagation_mismatch$'; then
  echo "expected full I/O scenario matrix contract lane fail-closed reason marker" >&2
  exit 1
fi

python3 - "$lane_report" "$policy_report" <<'PY'
import json
import pathlib
import sys

lane_payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if lane_payload.get("schema_version") != "kamn.runtime.full-io-scenario-matrix-live-contract-lane-report.v1":
    raise SystemExit("unexpected full I/O scenario matrix contract lane report schema")
if lane_payload.get("status") != "pass":
    raise SystemExit("expected contract lane status=pass")
if lane_payload.get("final_decision") != "GO":
    raise SystemExit("expected contract lane final_decision=GO")
if lane_payload.get("full_io_scenario_matrix_policy_status") != "verified":
    raise SystemExit("expected full_io_scenario_matrix_policy_status=verified")
if lane_payload.get("full_io_scenario_matrix_contract_status") != "verified":
    raise SystemExit("expected full_io_scenario_matrix_contract_status=verified")
if lane_payload.get("docs_contract_status") != "verified":
    raise SystemExit("expected docs_contract_status=verified")
if lane_payload.get("performance_budget_status") != "verified":
    raise SystemExit("expected performance_budget_status=verified")

policy_payload = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
if policy_payload.get("schema_version") != "kamn.runtime.full-io-scenario-matrix-live-policy-report.v1":
    raise SystemExit("unexpected full I/O scenario matrix policy report schema")
if policy_payload.get("final_decision") != "GO":
    raise SystemExit("expected policy final_decision=GO")
if policy_payload.get("full_io_scenario_matrix_policy_status") != "verified":
    raise SystemExit("expected full_io_scenario_matrix_policy_status=verified in policy report")
PY

if ! grep -q "check_full_io_scenario_matrix_live_policy.sh" "$CONTRACT_LANE"; then
  echo "expected full I/O scenario matrix contract lane to compose policy checker" >&2
  exit 1
fi
if ! grep -q "validate_full_io_scenario_matrix_live.sh" "$CONTRACT_LANE"; then
  echo "expected full I/O scenario matrix contract lane to compose validation lane" >&2
  exit 1
fi

set +e
invalid_ci_fast_gate_output="$(
  bash "$CONTRACT_LANE" \
    --mode dry-run \
    --ci-fast-gate MAYBE 2>&1
)"
invalid_ci_fast_gate_code=$?
set -e
if [ "$invalid_ci_fast_gate_code" -eq 0 ]; then
  echo "expected full I/O scenario matrix contract lane to reject invalid ci-fast-gate value" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_ci_fast_gate_output" | grep -q 'ci-fast-gate must be PASS or FAIL'; then
  echo "expected deterministic invalid ci-fast-gate marker for full I/O scenario matrix contract lane" >&2
  exit 1
fi

echo "full I/O scenario matrix contract lane tests passed."
