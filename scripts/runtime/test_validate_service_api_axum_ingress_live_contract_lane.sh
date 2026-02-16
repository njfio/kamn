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
if ! printf '%s\n' "$lane_output" | grep -q '^request_validation_status=verified$'; then
  echo "expected service api axum ingress contract lane request-validation marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^error_envelope_field_status=verified$'; then
  echo "expected service api axum ingress contract lane error-envelope field marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^method_path_classification_status=verified$'; then
  echo "expected service api axum ingress contract lane method/path classification marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^ingress_resilience_gate_status=verified$'; then
  echo "expected service api axum ingress contract lane ingress-resilience gate marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^websocket_upgrade_parity_status=verified$'; then
  echo "expected service api axum ingress contract lane websocket-upgrade parity marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^ci_local_promotion_budget_boundary_status=verified$'; then
  echo "expected service api axum ingress contract lane ci/local promotion budget boundary marker" >&2
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
if ! printf '%s\n' "$lane_output" | grep -q '^request_validation_reason_taxonomy_version=kamn.runtime.service-api-request-validation-reason-taxonomy.v1$'; then
  echo "expected service api axum ingress contract lane request-validation reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^request_validation_reason_registry_status=verified$'; then
  echo "expected service api axum ingress contract lane request-validation reason registry marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^error_envelope_source_contract_status=verified$'; then
  echo "expected service api axum ingress contract lane error-envelope source contract marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^request_validation_reason_codes_csv=service_api_ws_upgrade_header_missing,service_api_ws_version_header_invalid,service_api_method_not_allowed,service_api_route_not_found,service_api_payload_json_syntax_invalid,service_api_payload_structure_invalid$'; then
  echo "expected service api axum ingress contract lane request-validation reason taxonomy csv marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^error_envelope_reason_taxonomy_version=kamn.runtime.service-api-error-envelope-reason-taxonomy.v1$'; then
  echo "expected service api axum ingress contract lane error-envelope reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^error_envelope_reason_codes_csv=service_api_ws_upgrade_header_missing,service_api_method_not_allowed,service_api_route_not_found$'; then
  echo "expected service api axum ingress contract lane error-envelope reason taxonomy csv marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^ingress_resilience_reason_taxonomy_version=kamn.runtime.service-api-ingress-resilience-reason-taxonomy.v1$'; then
  echo "expected service api axum ingress contract lane ingress-resilience reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^ingress_resilience_reason_codes_csv=ingress_readiness_progress_stalled,websocket_upgrade_parity_mismatch,ci_local_promotion_budget_boundary_exceeded$'; then
  echo "expected service api axum ingress contract lane ingress-resilience reason taxonomy csv marker" >&2
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
if ! printf '%s\n' "$lane_output" | grep -q '^fail_closed_reason_code=service_api_axum_policy_marker_missing:websocket_upgrade_parity_status$'; then
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
if lane_payload.get("request_validation_status") != "verified":
    raise SystemExit("expected request_validation_status=verified")
if lane_payload.get("error_envelope_field_status") != "verified":
    raise SystemExit("expected error_envelope_field_status=verified")
if lane_payload.get("method_path_classification_status") != "verified":
    raise SystemExit("expected method_path_classification_status=verified")
if lane_payload.get("ingress_resilience_gate_status") != "verified":
    raise SystemExit("expected ingress_resilience_gate_status=verified")
if lane_payload.get("websocket_upgrade_parity_status") != "verified":
    raise SystemExit("expected websocket_upgrade_parity_status=verified")
if lane_payload.get("ci_local_promotion_budget_boundary_status") != "verified":
    raise SystemExit("expected ci_local_promotion_budget_boundary_status=verified")
if lane_payload.get("protocol_compliance_reason_taxonomy_version") != "kamn.runtime.service-api-protocol-compliance-reason-taxonomy.v1":
    raise SystemExit("expected deterministic protocol_compliance_reason_taxonomy_version marker")
if lane_payload.get("protocol_compliance_reason_codes_csv") != "method_path_contract_mismatch,payload_shape_contract_mismatch,route_contract_bypass_detected":
    raise SystemExit("expected deterministic protocol_compliance_reason_codes_csv marker")
if lane_payload.get("request_validation_reason_taxonomy_version") != "kamn.runtime.service-api-request-validation-reason-taxonomy.v1":
    raise SystemExit("expected deterministic request_validation_reason_taxonomy_version marker")
if lane_payload.get("request_validation_reason_registry_status") != "verified":
    raise SystemExit("expected request_validation_reason_registry_status=verified")
if lane_payload.get("error_envelope_source_contract_status") != "verified":
    raise SystemExit("expected error_envelope_source_contract_status=verified")
if lane_payload.get("request_validation_reason_codes_csv") != "service_api_ws_upgrade_header_missing,service_api_ws_version_header_invalid,service_api_method_not_allowed,service_api_route_not_found,service_api_payload_json_syntax_invalid,service_api_payload_structure_invalid":
    raise SystemExit("expected deterministic request_validation_reason_codes_csv marker")
if lane_payload.get("error_envelope_reason_taxonomy_version") != "kamn.runtime.service-api-error-envelope-reason-taxonomy.v1":
    raise SystemExit("expected deterministic error_envelope_reason_taxonomy_version marker")
if lane_payload.get("error_envelope_reason_codes_csv") != "service_api_ws_upgrade_header_missing,service_api_method_not_allowed,service_api_route_not_found":
    raise SystemExit("expected deterministic error_envelope_reason_codes_csv marker")
if lane_payload.get("ingress_resilience_reason_taxonomy_version") != "kamn.runtime.service-api-ingress-resilience-reason-taxonomy.v1":
    raise SystemExit("expected deterministic ingress_resilience_reason_taxonomy_version marker")
if lane_payload.get("ingress_resilience_reason_codes_csv") != "ingress_readiness_progress_stalled,websocket_upgrade_parity_mismatch,ci_local_promotion_budget_boundary_exceeded":
    raise SystemExit("expected deterministic ingress_resilience_reason_codes_csv marker")
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
if policy_payload.get("request_validation_reason_taxonomy_version") != "kamn.runtime.service-api-request-validation-reason-taxonomy.v1":
    raise SystemExit("expected deterministic request_validation_reason_taxonomy_version marker in policy report")
if policy_payload.get("request_validation_reason_registry_status") != "verified":
    raise SystemExit("expected request_validation_reason_registry_status=verified in policy report")
if policy_payload.get("error_envelope_source_contract_status") != "verified":
    raise SystemExit("expected error_envelope_source_contract_status=verified in policy report")
if policy_payload.get("request_validation_reason_codes_csv") != "service_api_ws_upgrade_header_missing,service_api_ws_version_header_invalid,service_api_method_not_allowed,service_api_route_not_found,service_api_payload_json_syntax_invalid,service_api_payload_structure_invalid":
    raise SystemExit("expected deterministic request_validation_reason_codes_csv marker in policy report")
if policy_payload.get("error_envelope_reason_taxonomy_version") != "kamn.runtime.service-api-error-envelope-reason-taxonomy.v1":
    raise SystemExit("expected deterministic error_envelope_reason_taxonomy_version marker in policy report")
if policy_payload.get("error_envelope_reason_codes_csv") != "service_api_ws_upgrade_header_missing,service_api_method_not_allowed,service_api_route_not_found":
    raise SystemExit("expected deterministic error_envelope_reason_codes_csv marker in policy report")
if policy_payload.get("ingress_resilience_reason_taxonomy_version") != "kamn.runtime.service-api-ingress-resilience-reason-taxonomy.v1":
    raise SystemExit("expected deterministic ingress_resilience_reason_taxonomy_version marker in policy report")
if policy_payload.get("ingress_resilience_reason_codes_csv") != "ingress_readiness_progress_stalled,websocket_upgrade_parity_mismatch,ci_local_promotion_budget_boundary_exceeded":
    raise SystemExit("expected deterministic ingress_resilience_reason_codes_csv marker in policy report")
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
