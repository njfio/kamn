#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/runtime/validate_local_full_runtime_live_contract_lane.sh"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_local_full_runtime_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_local_full_runtime_live_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected local full-runtime contract lane script to be executable" >&2
  exit 1
fi
if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected local full-runtime validation script to be executable" >&2
  exit 1
fi
if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected local full-runtime policy checker script to be executable" >&2
  exit 1
fi

lane_report="$TMP_DIR/local-full-runtime-contract-lane-report.json"
policy_report="$TMP_DIR/local-full-runtime-policy-report.json"

lane_output="$(
  bash "$CONTRACT_LANE" \
    --mode dry-run \
    --max-seconds 240 \
    --ci-fast-gate PASS \
    --output-json "$lane_report" \
    --policy-output-json "$policy_report"
)"
if ! printf '%s\n' "$lane_output" | grep -q '^status=pass$'; then
  echo "expected local full-runtime contract lane status=pass marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^final_decision=GO$'; then
  echo "expected local full-runtime contract lane final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^lane_mode=dry-run$'; then
  echo "expected local full-runtime contract lane mode marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^local_full_runtime_policy_status=verified$'; then
  echo "expected local full-runtime contract lane policy status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^local_full_runtime_contract_status=verified$'; then
  echo "expected local full-runtime contract lane status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^three_node_convergence_status=verified$'; then
  echo "expected local full-runtime contract lane three-node convergence marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^runtime_shutdown_gate_status=verified$'; then
  echo "expected local full-runtime contract lane runtime shutdown gate marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^runtime_transport_mode_status=verified$'; then
  echo "expected local full-runtime contract lane runtime transport mode marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^runtime_fallback_classification_status=verified$'; then
  echo "expected local full-runtime contract lane runtime fallback classification marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^runtime_error_reason_taxonomy_version=kamn.runtime.local-full-runtime-error-reason-taxonomy.v1$'; then
  echo "expected local full-runtime contract lane runtime error reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^runtime_error_reason_codes_csv=runtime_full_shutdown_gate_drift_detected,runtime_fallback_classification_unstable,ci_local_runtime_extraction_budget_boundary_exceeded$'; then
  echo "expected local full-runtime contract lane runtime error reason codes marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^reason_taxonomy_version=kamn.runtime.local-full-runtime-error-reason-taxonomy.v1$'; then
  echo "expected local full-runtime contract lane policy reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^reason_codes_csv=runtime_full_shutdown_gate_drift_detected,runtime_fallback_classification_unstable,ci_local_runtime_extraction_budget_boundary_exceeded$'; then
  echo "expected local full-runtime contract lane policy reason codes marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^ci_local_runtime_extraction_budget_boundary_status=verified$'; then
  echo "expected local full-runtime contract lane ci-local extraction budget boundary marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^fail_closed_reason_code=local_full_runtime_policy_fast_gate_exclusion_mismatch$'; then
  echo "expected local full-runtime contract lane fail-closed reason marker" >&2
  exit 1
fi

python3 - "$lane_report" "$policy_report" <<'PY'
import json
import pathlib
import sys

lane_payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if lane_payload.get("schema_version") != "kamn.runtime.local-full-runtime-live-contract-lane-report.v1":
    raise SystemExit("unexpected local full-runtime contract lane report schema")
if lane_payload.get("status") != "pass":
    raise SystemExit("expected contract lane status=pass")
if lane_payload.get("final_decision") != "GO":
    raise SystemExit("expected contract lane final_decision=GO")
if lane_payload.get("local_full_runtime_policy_status") != "verified":
    raise SystemExit("expected local_full_runtime_policy_status=verified")
if lane_payload.get("local_full_runtime_contract_status") != "verified":
    raise SystemExit("expected local_full_runtime_contract_status=verified")
if lane_payload.get("docs_contract_status") != "verified":
    raise SystemExit("expected docs_contract_status=verified")
if lane_payload.get("three_node_convergence_status") != "verified":
    raise SystemExit("expected three_node_convergence_status=verified")
if lane_payload.get("runtime_shutdown_gate_status") != "verified":
    raise SystemExit("expected runtime_shutdown_gate_status=verified")
if lane_payload.get("runtime_transport_mode_status") != "verified":
    raise SystemExit("expected runtime_transport_mode_status=verified")
if lane_payload.get("runtime_fallback_classification_status") != "verified":
    raise SystemExit("expected runtime_fallback_classification_status=verified")
if lane_payload.get("runtime_error_reason_taxonomy_version") != "kamn.runtime.local-full-runtime-error-reason-taxonomy.v1":
    raise SystemExit("expected runtime_error_reason_taxonomy_version marker")
if lane_payload.get("runtime_error_reason_codes_csv") != "runtime_full_shutdown_gate_drift_detected,runtime_fallback_classification_unstable,ci_local_runtime_extraction_budget_boundary_exceeded":
    raise SystemExit("expected runtime_error_reason_codes_csv marker")
if lane_payload.get("reason_taxonomy_version") != "kamn.runtime.local-full-runtime-error-reason-taxonomy.v1":
    raise SystemExit("expected reason_taxonomy_version marker")
if lane_payload.get("reason_codes_csv") != "runtime_full_shutdown_gate_drift_detected,runtime_fallback_classification_unstable,ci_local_runtime_extraction_budget_boundary_exceeded":
    raise SystemExit("expected reason_codes_csv marker")
if lane_payload.get("ci_local_runtime_extraction_budget_boundary_status") != "verified":
    raise SystemExit("expected ci_local_runtime_extraction_budget_boundary_status=verified")
if lane_payload.get("performance_budget_status") != "verified":
    raise SystemExit("expected performance_budget_status=verified")

policy_payload = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
if policy_payload.get("schema_version") != "kamn.runtime.local-full-runtime-live-policy-report.v1":
    raise SystemExit("unexpected local full-runtime policy report schema")
if policy_payload.get("final_decision") != "GO":
    raise SystemExit("expected policy final_decision=GO")
if policy_payload.get("local_full_runtime_policy_status") != "verified":
    raise SystemExit("expected local_full_runtime_policy_status=verified in policy report")
if policy_payload.get("reason_taxonomy_version") != "kamn.runtime.local-full-runtime-error-reason-taxonomy.v1":
    raise SystemExit("expected policy reason_taxonomy_version marker")
if policy_payload.get("reason_codes_csv") != "runtime_full_shutdown_gate_drift_detected,runtime_fallback_classification_unstable,ci_local_runtime_extraction_budget_boundary_exceeded":
    raise SystemExit("expected policy reason_codes_csv marker")
PY

if ! grep -q "check_local_full_runtime_live_policy.sh" "$CONTRACT_LANE"; then
  echo "expected local full-runtime contract lane to compose policy checker" >&2
  exit 1
fi
if ! grep -q "validate_local_full_runtime_live.sh" "$CONTRACT_LANE"; then
  echo "expected local full-runtime contract lane to compose validation lane" >&2
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
  echo "expected local full-runtime contract lane to reject invalid ci-fast-gate value" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_ci_fast_gate_output" | grep -q 'ci-fast-gate must be PASS or FAIL'; then
  echo "expected deterministic invalid ci-fast-gate marker for local full-runtime contract lane" >&2
  exit 1
fi

set +e
invalid_budget_output="$(
  bash "$CONTRACT_LANE" \
    --mode dry-run \
    --max-seconds 241 2>&1
)"
invalid_budget_code=$?
set -e
if [ "$invalid_budget_code" -eq 0 ]; then
  echo "expected local full-runtime contract lane to reject ci-local extraction budget overrun" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_budget_output" | grep -q 'max-seconds must be <= 240 for ci-local contract lane'; then
  echo "expected deterministic ci-local extraction budget boundary marker for local full-runtime contract lane" >&2
  exit 1
fi

echo "local full-runtime contract lane tests passed."
