#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/runtime/validate_service_api_reason_code_compatibility_live_contract_lane.sh"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_service_api_reason_code_compatibility_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_service_api_reason_code_compatibility_live_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected service api reason-code compatibility contract lane script to be executable" >&2
  exit 1
fi
if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected service api reason-code compatibility validation script to be executable" >&2
  exit 1
fi
if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected service api reason-code compatibility policy checker script to be executable" >&2
  exit 1
fi

lane_report="$TMP_DIR/service-api-reason-code-compatibility-contract-lane-report.json"
policy_report="$TMP_DIR/service-api-reason-code-compatibility-policy-report.json"

lane_output="$(
  bash "$CONTRACT_LANE" \
    --output-json "$lane_report" \
    --policy-output-json "$policy_report"
)"
if ! printf '%s\n' "$lane_output" | grep -q '^status=pass$'; then
  echo "expected service api reason-code compatibility contract lane status=pass marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^final_decision=GO$'; then
  echo "expected service api reason-code compatibility contract lane final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^service_api_reason_code_contract_status=verified$'; then
  echo "expected service api reason-code compatibility contract lane status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^service_api_reason_code_policy_status=verified$'; then
  echo "expected service api reason-code compatibility contract lane policy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^error_envelope_field_status=verified$'; then
  echo "expected service api reason-code compatibility contract lane envelope marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^rust_sdk_reason_code_status=verified$'; then
  echo "expected service api reason-code compatibility contract lane rust sdk marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^python_sdk_reason_code_status=verified$'; then
  echo "expected service api reason-code compatibility contract lane python sdk marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^fail_closed_reason_code=service_api_reason_code_policy_marker_missing:route_error_mapping_status$'; then
  echo "expected service api reason-code compatibility contract lane fail-closed reason marker" >&2
  exit 1
fi

python3 - "$lane_report" "$policy_report" <<'PY'
import json
import pathlib
import sys

lane_payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if lane_payload.get("schema_version") != "kamn.runtime.service-api-reason-code-compatibility-live-contract-lane-report.v1":
    raise SystemExit("unexpected service api reason-code compatibility contract lane report schema")
if lane_payload.get("status") != "pass":
    raise SystemExit("expected contract lane status=pass")
if lane_payload.get("final_decision") != "GO":
    raise SystemExit("expected contract lane final_decision=GO")
if lane_payload.get("service_api_reason_code_contract_status") != "verified":
    raise SystemExit("expected service_api_reason_code_contract_status=verified")
if lane_payload.get("service_api_reason_code_policy_status") != "verified":
    raise SystemExit("expected service_api_reason_code_policy_status=verified")
if lane_payload.get("error_envelope_field_status") != "verified":
    raise SystemExit("expected error_envelope_field_status=verified")
if lane_payload.get("rust_sdk_reason_code_status") != "verified":
    raise SystemExit("expected rust_sdk_reason_code_status=verified")
if lane_payload.get("python_sdk_reason_code_status") != "verified":
    raise SystemExit("expected python_sdk_reason_code_status=verified")
if lane_payload.get("docs_contract_status") != "verified":
    raise SystemExit("expected docs_contract_status=verified")
if lane_payload.get("performance_budget_status") != "verified":
    raise SystemExit("expected performance_budget_status=verified")

policy_payload = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
if policy_payload.get("schema_version") != "kamn.runtime.service-api-reason-code-compatibility-live-policy-report.v1":
    raise SystemExit("unexpected service api reason-code compatibility policy report schema")
if policy_payload.get("final_decision") != "GO":
    raise SystemExit("expected policy final_decision=GO")
if policy_payload.get("service_api_reason_code_policy_status") != "verified":
    raise SystemExit("expected service_api_reason_code_policy_status=verified in policy report")
PY

if ! grep -q "check_service_api_reason_code_compatibility_live_policy.sh" "$CONTRACT_LANE"; then
  echo "expected service api reason-code compatibility contract lane to compose policy checker" >&2
  exit 1
fi
if ! grep -q "validate_service_api_reason_code_compatibility_live.sh" "$CONTRACT_LANE"; then
  echo "expected service api reason-code compatibility contract lane to compose validation lane" >&2
  exit 1
fi

set +e
invalid_ci_fast_gate_output="$(
  bash "$CONTRACT_LANE" \
    --ci-fast-gate MAYBE 2>&1
)"
invalid_ci_fast_gate_code=$?
set -e
if [ "$invalid_ci_fast_gate_code" -eq 0 ]; then
  echo "expected service api reason-code compatibility contract lane to reject invalid ci-fast-gate value" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_ci_fast_gate_output" | grep -q 'ci-fast-gate must be PASS or FAIL'; then
  echo "expected deterministic invalid ci-fast-gate marker for service api reason-code compatibility contract lane" >&2
  exit 1
fi

set +e
blocked_fast_gate_output="$(
  bash "$CONTRACT_LANE" \
    --ci-fast-gate FAIL 2>&1
)"
blocked_fast_gate_code=$?
set -e
if [ "$blocked_fast_gate_code" -eq 0 ]; then
  echo "expected service api reason-code compatibility contract lane to fail closed when ci-fast-gate=FAIL" >&2
  exit 1
fi
if ! printf '%s\n' "$blocked_fast_gate_output" | grep -q 'ci_fast_gate_failed'; then
  echo "expected deterministic ci_fast_gate_failed marker for service api reason-code compatibility contract lane" >&2
  exit 1
fi

echo "service api reason-code compatibility contract lane tests passed."
