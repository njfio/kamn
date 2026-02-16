#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/runtime/validate_service_api_axum_ingress_live_contract_lane.sh"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_service_api_axum_ingress_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_service_api_axum_ingress_live_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected service api axum ingress contract lane script to be executable" >&2
  exit 1
fi
if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected service api axum ingress validation script to be executable" >&2
  exit 1
fi
if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected service api axum ingress policy checker script to be executable" >&2
  exit 1
fi

lane_report="$TMP_DIR/service-api-axum-ingress-contract-lane-report.json"
policy_report="$TMP_DIR/service-api-axum-ingress-policy-report.json"

lane_output="$(
  bash "$CONTRACT_LANE" \
    --output-json "$lane_report" \
    --policy-output-json "$policy_report"
)"
if ! printf '%s\n' "$lane_output" | grep -q '^status=pass$'; then
  echo "expected service api axum ingress contract lane status=pass marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^final_decision=GO$'; then
  echo "expected service api axum ingress contract lane final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^service_api_axum_ingress_contract_status=verified$'; then
  echo "expected service api axum ingress contract lane status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^service_api_axum_ingress_policy_status=verified$'; then
  echo "expected service api axum ingress contract lane policy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^ingress_limit_config_status=verified$'; then
  echo "expected service api axum ingress contract lane config matrix marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^docs_ingress_limit_matrix_status=verified$'; then
  echo "expected service api axum ingress contract lane docs parity marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^protocol_compliance_status=verified$'; then
  echo "expected service api axum ingress contract lane protocol compliance marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^route_contract_parity_status=verified$'; then
  echo "expected service api axum ingress contract lane route-contract parity marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^protocol_compliance_reason_taxonomy_version=kamn.runtime.service-api-protocol-compliance-reason-taxonomy.v1$'; then
  echo "expected service api axum ingress contract lane protocol-compliance reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^protocol_compliance_reason_codes_csv=method_path_contract_mismatch,payload_shape_contract_mismatch,route_contract_bypass_detected$'; then
  echo "expected service api axum ingress contract lane protocol-compliance reason taxonomy csv marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -Eq '^api_max_requests_default=[1-9][0-9]*$'; then
  echo "expected service api axum ingress contract lane max-requests default marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -Eq '^api_idle_timeout_default_ms=[1-9][0-9]*$'; then
  echo "expected service api axum ingress contract lane idle-timeout default marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -Eq '^body_size_limit_bytes=[1-9][0-9]*$'; then
  echo "expected service api axum ingress contract lane body-size limit marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -Eq '^api_concurrency_limit_default=[1-9][0-9]*$'; then
  echo "expected service api axum ingress contract lane concurrency-limit default marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -Eq '^api_rate_limit_per_second_default=[1-9][0-9]*$'; then
  echo "expected service api axum ingress contract lane rate-limit default marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^fail_closed_reason_code=service_api_axum_policy_marker_missing:route_contract_parity_status$'; then
  echo "expected service api axum ingress contract lane fail-closed reason marker" >&2
  exit 1
fi

python3 - "$lane_report" "$policy_report" <<'PY'
import json
import pathlib
import sys

lane_payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if lane_payload.get("schema_version") != "kamn.runtime.service-api-axum-ingress-live-contract-lane-report.v1":
    raise SystemExit("unexpected service api axum ingress contract lane report schema")
if lane_payload.get("status") != "pass":
    raise SystemExit("expected contract lane status=pass")
if lane_payload.get("final_decision") != "GO":
    raise SystemExit("expected contract lane final_decision=GO")
if lane_payload.get("service_api_axum_ingress_contract_status") != "verified":
    raise SystemExit("expected service_api_axum_ingress_contract_status=verified")
if lane_payload.get("service_api_axum_ingress_policy_status") != "verified":
    raise SystemExit("expected service_api_axum_ingress_policy_status=verified")
if lane_payload.get("ingress_limit_config_status") != "verified":
    raise SystemExit("expected ingress_limit_config_status=verified")
if lane_payload.get("docs_ingress_limit_matrix_status") != "verified":
    raise SystemExit("expected docs_ingress_limit_matrix_status=verified")
if lane_payload.get("protocol_compliance_status") != "verified":
    raise SystemExit("expected protocol_compliance_status=verified")
if lane_payload.get("route_contract_parity_status") != "verified":
    raise SystemExit("expected route_contract_parity_status=verified")
if lane_payload.get("protocol_compliance_reason_taxonomy_version") != "kamn.runtime.service-api-protocol-compliance-reason-taxonomy.v1":
    raise SystemExit("expected deterministic protocol_compliance_reason_taxonomy_version marker")
if lane_payload.get("protocol_compliance_reason_codes_csv") != "method_path_contract_mismatch,payload_shape_contract_mismatch,route_contract_bypass_detected":
    raise SystemExit("expected deterministic protocol_compliance_reason_codes_csv marker")
if lane_payload.get("api_max_requests_default") != 1:
    raise SystemExit("expected api_max_requests_default=1")
if lane_payload.get("api_idle_timeout_default_ms") != 5000:
    raise SystemExit("expected api_idle_timeout_default_ms=5000")
if lane_payload.get("body_size_limit_bytes") != 65536:
    raise SystemExit("expected body_size_limit_bytes=65536")
if lane_payload.get("api_concurrency_limit_default") != 32:
    raise SystemExit("expected api_concurrency_limit_default=32")
if lane_payload.get("api_rate_limit_per_second_default") != 120:
    raise SystemExit("expected api_rate_limit_per_second_default=120")
if lane_payload.get("docs_contract_status") != "verified":
    raise SystemExit("expected docs_contract_status=verified")
if lane_payload.get("performance_budget_status") != "verified":
    raise SystemExit("expected performance_budget_status=verified")

policy_payload = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
if policy_payload.get("schema_version") != "kamn.runtime.service-api-axum-ingress-live-policy-report.v1":
    raise SystemExit("unexpected service api axum ingress policy report schema")
if policy_payload.get("final_decision") != "GO":
    raise SystemExit("expected policy final_decision=GO")
if policy_payload.get("service_api_axum_ingress_policy_status") != "verified":
    raise SystemExit("expected service_api_axum_ingress_policy_status=verified in policy report")
if policy_payload.get("protocol_compliance_reason_taxonomy_version") != "kamn.runtime.service-api-protocol-compliance-reason-taxonomy.v1":
    raise SystemExit("expected deterministic protocol_compliance_reason_taxonomy_version marker in policy report")
if policy_payload.get("protocol_compliance_reason_codes_csv") != "method_path_contract_mismatch,payload_shape_contract_mismatch,route_contract_bypass_detected":
    raise SystemExit("expected deterministic protocol_compliance_reason_codes_csv marker in policy report")
PY

if ! grep -q "check_service_api_axum_ingress_live_policy.sh" "$CONTRACT_LANE"; then
  echo "expected service api axum ingress contract lane to compose policy checker" >&2
  exit 1
fi
if ! grep -q "validate_service_api_axum_ingress_live.sh" "$CONTRACT_LANE"; then
  echo "expected service api axum ingress contract lane to compose validation lane" >&2
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
  echo "expected service api axum ingress contract lane to reject invalid ci-fast-gate value" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_ci_fast_gate_output" | grep -q 'ci-fast-gate must be PASS or FAIL'; then
  echo "expected deterministic invalid ci-fast-gate marker for service api axum ingress contract lane" >&2
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
  echo "expected service api axum ingress contract lane to fail closed when ci-fast-gate=FAIL" >&2
  exit 1
fi
if ! printf '%s\n' "$blocked_fast_gate_output" | grep -q 'ci_fast_gate_failed'; then
  echo "expected deterministic ci_fast_gate_failed marker for service api axum ingress contract lane" >&2
  exit 1
fi

echo "service api axum ingress contract lane tests passed."
