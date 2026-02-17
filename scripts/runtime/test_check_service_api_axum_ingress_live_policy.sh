#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_service_api_axum_ingress_live_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected service api axum ingress policy checker script to be executable" >&2
  exit 1
fi

report_file="$TMP_DIR/service-api-axum-ingress-live-summary.json"
cat >"$report_file" <<'JSON'
{
  "schema_version": "kamn.runtime.service-api-axum-ingress-live-validation.v1",
  "status": "pass",
  "final_decision": "GO",
  "keep_alive_status": "verified",
  "body_size_guard_status": "verified",
  "concurrency_status": "verified",
  "websocket_status": "verified",
  "ingress_limit_config_status": "verified",
  "docs_ingress_limit_matrix_status": "verified",
  "protocol_compliance_status": "verified",
  "route_contract_parity_status": "verified",
  "request_validation_status": "verified",
  "error_envelope_field_status": "verified",
  "method_path_classification_status": "verified",
  "ingress_resilience_gate_status": "verified",
  "websocket_upgrade_parity_status": "verified",
  "ci_local_promotion_budget_boundary_status": "verified",
  "admission_saturation_status": "verified",
  "admission_queue_cap_enforcement_status": "verified",
  "admission_inflight_budget_status": "verified",
  "admission_queue_budget_status": "verified",
  "overload_evidence_normalization_status": "verified",
  "async_lifecycle_backpressure_projection_status": "verified",
  "protocol_compliance_reason_taxonomy_version": "kamn.runtime.service-api-protocol-compliance-reason-taxonomy.v1",
  "protocol_compliance_reason_codes_csv": "method_path_contract_mismatch,payload_shape_contract_mismatch,route_contract_bypass_detected",
  "ingress_resilience_reason_taxonomy_version": "kamn.runtime.service-api-ingress-resilience-reason-taxonomy.v1",
  "ingress_resilience_reason_codes_csv": "ingress_readiness_progress_stalled,websocket_upgrade_parity_mismatch,ci_local_promotion_budget_boundary_exceeded",
  "admission_reason_taxonomy_version": "kamn.runtime.service-api-admission-reason-taxonomy.v1",
  "admission_reason_codes_csv": "admission_queue_saturation_detected,admission_queue_cap_bypass_detected,admission_evidence_normalization_drift",
  "admission_budget_reason_taxonomy_version": "kamn.runtime.service-api-admission-budget-reason-taxonomy.v1",
  "admission_budget_reason_codes_csv": "admission_inflight_budget_mismatch,admission_queue_budget_mismatch",
  "admission_decision_taxonomy_status": "verified",
  "admission_decision_accept_status": "verified",
  "admission_decision_defer_status": "verified",
  "admission_decision_reject_status": "verified",
  "admission_decision_reason_taxonomy_version": "kamn.runtime.service-api-admission-decision-reason-taxonomy.v1",
  "admission_decision_reason_codes_csv": "admission_decision_accept,admission_decision_defer,admission_decision_reject",
  "service_api_lifecycle_rejection_reason_taxonomy_version": "kamn.runtime.service-api.lifecycle-rejection-reason-taxonomy.v1",
  "service_api_lifecycle_rejection_reason_codes_csv": "service_api_ingress_concurrency_limit_exceeded,service_api_ingress_rate_limit_exceeded,service_api_ingress_sender_rate_limit_exceeded,service_api_ingress_sender_suspended,service_api_ingress_sender_duplicate_message_id,service_api_ingress_sender_insufficient_deposit,service_api_ingress_anti_spam_engine_invalid",
  "request_validation_reason_registry_status": "verified",
  "error_envelope_source_contract_status": "verified",
  "request_validation_reason_taxonomy_version": "kamn.runtime.service-api-request-validation-reason-taxonomy.v1",
  "request_validation_reason_codes_csv": "service_api_ws_upgrade_header_missing,service_api_ws_version_header_invalid,service_api_method_not_allowed,service_api_route_not_found,service_api_payload_json_syntax_invalid,service_api_payload_structure_invalid",
  "error_envelope_reason_taxonomy_version": "kamn.runtime.service-api-error-envelope-reason-taxonomy.v1",
  "error_envelope_reason_codes_csv": "service_api_ws_upgrade_header_missing,service_api_method_not_allowed,service_api_route_not_found",
  "api_max_requests_default": 1,
  "api_idle_timeout_default_ms": 5000,
  "admission_inflight_budget_limit": 32,
  "admission_queue_budget_limit": 1,
  "body_size_limit_bytes": 65536,
  "api_concurrency_limit_default": 32,
  "api_rate_limit_per_second_default": 120,
  "fail_closed_status": "verified",
  "ci_fast_gate_exclusion_status": "verified",
  "performance_budget_status": "verified",
  "fail_closed_reason_code": "service_api_axum_oversized_body_rejected",
  "elapsed_seconds": 3
}
JSON

policy_report="$TMP_DIR/service-api-axum-ingress-live-policy.json"
policy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$report_file" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$policy_report"
)"
if ! printf '%s\n' "$policy_output" | grep -q '^status=ok$'; then
  echo "expected service api axum ingress policy checker status=ok marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^final_decision=GO$'; then
  echo "expected service api axum ingress policy checker GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^service_api_axum_ingress_policy_status=verified$'; then
  echo "expected service api axum ingress policy checker status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^reason_codes_value=none$'; then
  echo "expected service api axum ingress policy checker normalized reason_codes_value marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^service_api_lifecycle_rejection_reason_taxonomy_version=kamn.runtime.service-api.lifecycle-rejection-reason-taxonomy.v1$'; then
  echo "expected service api axum ingress policy checker lifecycle rejection reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^service_api_lifecycle_rejection_reason_codes_csv=service_api_ingress_concurrency_limit_exceeded,service_api_ingress_rate_limit_exceeded,service_api_ingress_sender_rate_limit_exceeded,service_api_ingress_sender_suspended,service_api_ingress_sender_duplicate_message_id,service_api_ingress_sender_insufficient_deposit,service_api_ingress_anti_spam_engine_invalid$'; then
  echo "expected service api axum ingress policy checker lifecycle rejection reason taxonomy csv marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^admission_inflight_budget_status=verified$'; then
  echo "expected service api axum ingress policy checker in-flight budget status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^admission_queue_budget_status=verified$'; then
  echo "expected service api axum ingress policy checker queue budget status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^admission_inflight_budget_limit=32$'; then
  echo "expected service api axum ingress policy checker in-flight budget limit marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^admission_queue_budget_limit=1$'; then
  echo "expected service api axum ingress policy checker queue budget limit marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^admission_budget_reason_taxonomy_version=kamn.runtime.service-api-admission-budget-reason-taxonomy.v1$'; then
  echo "expected service api axum ingress policy checker admission budget reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^admission_budget_reason_codes_csv=admission_inflight_budget_mismatch,admission_queue_budget_mismatch$'; then
  echo "expected service api axum ingress policy checker admission budget reason codes marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^admission_decision_taxonomy_status=verified$'; then
  echo "expected service api axum ingress policy checker admission decision taxonomy status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^admission_decision_accept_status=verified$'; then
  echo "expected service api axum ingress policy checker admission decision accept status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^admission_decision_defer_status=verified$'; then
  echo "expected service api axum ingress policy checker admission decision defer status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^admission_decision_reject_status=verified$'; then
  echo "expected service api axum ingress policy checker admission decision reject status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^admission_decision_reason_taxonomy_version=kamn.runtime.service-api-admission-decision-reason-taxonomy.v1$'; then
  echo "expected service api axum ingress policy checker admission decision reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^admission_decision_reason_codes_csv=admission_decision_accept,admission_decision_defer,admission_decision_reject$'; then
  echo "expected service api axum ingress policy checker admission decision reason codes marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^service_api_axum_protocol_mismatch_reason_mapping_status=verified$'; then
  echo "expected service api axum ingress policy checker protocol mismatch reason mapping status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^service_api_axum_protocol_mismatch_reason_taxonomy_version=kamn.runtime.service-api-axum-protocol-mismatch-reason-taxonomy.v1$'; then
  echo "expected service api axum ingress policy checker protocol mismatch reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^service_api_axum_protocol_mismatch_reason_codes_csv=service_api_axum_policy_required_field_missing,service_api_axum_policy_marker_missing,service_api_axum_policy_protocol_taxonomy_mismatch,service_api_axum_policy_limit_contract_mismatch,ci_fast_gate_failed,service_api_axum_policy_expected_decision_mismatch,service_api_axum_policy_violation$'; then
  echo "expected service api axum ingress policy checker protocol mismatch reason codes marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^service_api_axum_protocol_mismatch_reason_code=none$'; then
  echo "expected service api axum ingress policy checker protocol mismatch reason code marker" >&2
  exit 1
fi

python3 - "$policy_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.service-api-axum-ingress-live-policy-report.v1":
    raise SystemExit("unexpected service api axum ingress policy report schema")
if payload.get("status") != "pass":
    raise SystemExit("expected status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected final_decision=GO")
if payload.get("service_api_axum_ingress_policy_status") != "verified":
    raise SystemExit("expected service_api_axum_ingress_policy_status=verified")
if payload.get("reason_codes") != ["none"]:
    raise SystemExit("expected policy checker success reason code ['none']")
if payload.get("reason_codes_value") != "none":
    raise SystemExit("expected policy checker success normalized reason_codes_value marker")
if payload.get("protocol_compliance_reason_taxonomy_version") != "kamn.runtime.service-api-protocol-compliance-reason-taxonomy.v1":
    raise SystemExit("expected deterministic protocol_compliance_reason_taxonomy_version marker")
if payload.get("protocol_compliance_reason_codes_csv") != "method_path_contract_mismatch,payload_shape_contract_mismatch,route_contract_bypass_detected":
    raise SystemExit("expected deterministic protocol_compliance_reason_codes_csv marker")
if payload.get("ingress_resilience_reason_taxonomy_version") != "kamn.runtime.service-api-ingress-resilience-reason-taxonomy.v1":
    raise SystemExit("expected deterministic ingress_resilience_reason_taxonomy_version marker")
if payload.get("ingress_resilience_reason_codes_csv") != "ingress_readiness_progress_stalled,websocket_upgrade_parity_mismatch,ci_local_promotion_budget_boundary_exceeded":
    raise SystemExit("expected deterministic ingress_resilience_reason_codes_csv marker")
if payload.get("admission_saturation_status") != "verified":
    raise SystemExit("expected deterministic admission_saturation_status marker")
if payload.get("admission_queue_cap_enforcement_status") != "verified":
    raise SystemExit("expected deterministic admission_queue_cap_enforcement_status marker")
if payload.get("admission_inflight_budget_status") != "verified":
    raise SystemExit("expected deterministic admission_inflight_budget_status marker")
if payload.get("admission_queue_budget_status") != "verified":
    raise SystemExit("expected deterministic admission_queue_budget_status marker")
if payload.get("overload_evidence_normalization_status") != "verified":
    raise SystemExit("expected deterministic overload_evidence_normalization_status marker")
if payload.get("service_api_lifecycle_rejection_reason_taxonomy_version") != "kamn.runtime.service-api.lifecycle-rejection-reason-taxonomy.v1":
    raise SystemExit("expected deterministic service_api_lifecycle_rejection_reason_taxonomy_version marker")
if payload.get("service_api_lifecycle_rejection_reason_codes_csv") != "service_api_ingress_concurrency_limit_exceeded,service_api_ingress_rate_limit_exceeded,service_api_ingress_sender_rate_limit_exceeded,service_api_ingress_sender_suspended,service_api_ingress_sender_duplicate_message_id,service_api_ingress_sender_insufficient_deposit,service_api_ingress_anti_spam_engine_invalid":
    raise SystemExit("expected deterministic service_api_lifecycle_rejection_reason_codes_csv marker")
if payload.get("admission_reason_taxonomy_version") != "kamn.runtime.service-api-admission-reason-taxonomy.v1":
    raise SystemExit("expected deterministic admission_reason_taxonomy_version marker")
if payload.get("admission_reason_codes_csv") != "admission_queue_saturation_detected,admission_queue_cap_bypass_detected,admission_evidence_normalization_drift":
    raise SystemExit("expected deterministic admission_reason_codes_csv marker")
if payload.get("admission_budget_reason_taxonomy_version") != "kamn.runtime.service-api-admission-budget-reason-taxonomy.v1":
    raise SystemExit("expected deterministic admission_budget_reason_taxonomy_version marker")
if payload.get("admission_budget_reason_codes_csv") != "admission_inflight_budget_mismatch,admission_queue_budget_mismatch":
    raise SystemExit("expected deterministic admission_budget_reason_codes_csv marker")
if payload.get("admission_decision_taxonomy_status") != "verified":
    raise SystemExit("expected deterministic admission_decision_taxonomy_status marker")
if payload.get("admission_decision_accept_status") != "verified":
    raise SystemExit("expected deterministic admission_decision_accept_status marker")
if payload.get("admission_decision_defer_status") != "verified":
    raise SystemExit("expected deterministic admission_decision_defer_status marker")
if payload.get("admission_decision_reject_status") != "verified":
    raise SystemExit("expected deterministic admission_decision_reject_status marker")
if payload.get("admission_decision_reason_taxonomy_version") != "kamn.runtime.service-api-admission-decision-reason-taxonomy.v1":
    raise SystemExit("expected deterministic admission_decision_reason_taxonomy_version marker")
if payload.get("admission_decision_reason_codes_csv") != "admission_decision_accept,admission_decision_defer,admission_decision_reject":
    raise SystemExit("expected deterministic admission_decision_reason_codes_csv marker")
if payload.get("admission_inflight_budget_limit") != 32:
    raise SystemExit("expected deterministic admission_inflight_budget_limit marker")
if payload.get("admission_queue_budget_limit") != 1:
    raise SystemExit("expected deterministic admission_queue_budget_limit marker")
if payload.get("request_validation_reason_registry_status") != "verified":
    raise SystemExit("expected deterministic request_validation_reason_registry_status marker")
if payload.get("error_envelope_source_contract_status") != "verified":
    raise SystemExit("expected deterministic error_envelope_source_contract_status marker")
if payload.get("request_validation_reason_taxonomy_version") != "kamn.runtime.service-api-request-validation-reason-taxonomy.v1":
    raise SystemExit("expected deterministic request_validation_reason_taxonomy_version marker")
if payload.get("request_validation_reason_codes_csv") != "service_api_ws_upgrade_header_missing,service_api_ws_version_header_invalid,service_api_method_not_allowed,service_api_route_not_found,service_api_payload_json_syntax_invalid,service_api_payload_structure_invalid":
    raise SystemExit("expected deterministic request_validation_reason_codes_csv marker")
if payload.get("error_envelope_reason_taxonomy_version") != "kamn.runtime.service-api-error-envelope-reason-taxonomy.v1":
    raise SystemExit("expected deterministic error_envelope_reason_taxonomy_version marker")
if payload.get("error_envelope_reason_codes_csv") != "service_api_ws_upgrade_header_missing,service_api_method_not_allowed,service_api_route_not_found":
    raise SystemExit("expected deterministic error_envelope_reason_codes_csv marker")
if payload.get("service_api_axum_protocol_mismatch_reason_mapping_status") != "verified":
    raise SystemExit("expected service_api_axum_protocol_mismatch_reason_mapping_status=verified")
if payload.get("service_api_axum_protocol_mismatch_reason_taxonomy_version") != "kamn.runtime.service-api-axum-protocol-mismatch-reason-taxonomy.v1":
    raise SystemExit("expected deterministic service_api_axum_protocol_mismatch_reason_taxonomy_version marker")
if payload.get("service_api_axum_protocol_mismatch_reason_codes_csv") != "service_api_axum_policy_required_field_missing,service_api_axum_policy_marker_missing,service_api_axum_policy_protocol_taxonomy_mismatch,service_api_axum_policy_limit_contract_mismatch,ci_fast_gate_failed,service_api_axum_policy_expected_decision_mismatch,service_api_axum_policy_violation":
    raise SystemExit("expected deterministic service_api_axum_protocol_mismatch_reason_codes_csv marker")
if payload.get("service_api_axum_protocol_mismatch_reason_code") != "none":
    raise SystemExit("expected deterministic service_api_axum_protocol_mismatch_reason_code marker")
PY

tampered_report="$TMP_DIR/service-api-axum-ingress-live-summary.tampered.json"
cp "$report_file" "$tampered_report"
python3 - "$tampered_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["concurrency_status"] = "missing"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/service-api-axum-ingress-live-policy.tampered.json" 2>&1
)"
tampered_code=$?
set -e

if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered service api axum ingress report to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q 'service_api_axum_policy_marker_missing:concurrency_status'; then
  echo "expected deterministic mismatch reason code for tampered policy validation" >&2
  exit 1
fi

tampered_route_parity_report="$TMP_DIR/service-api-axum-ingress-live-summary.route-parity.tampered.json"
cp "$report_file" "$tampered_route_parity_report"
python3 - "$tampered_route_parity_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["route_contract_parity_status"] = "missing"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_route_parity_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_route_parity_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/service-api-axum-ingress-live-policy.route-parity.tampered.json" 2>&1
)"
tampered_route_parity_code=$?
set -e

if [ "$tampered_route_parity_code" -eq 0 ]; then
  echo "expected tampered service api route-contract parity to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_route_parity_output" | grep -q 'service_api_axum_policy_marker_missing:route_contract_parity_status'; then
  echo "expected deterministic mismatch reason code for tampered route-contract parity" >&2
  exit 1
fi

tampered_method_path_classification_report="$TMP_DIR/service-api-axum-ingress-live-summary.method-path-classification.tampered.json"
cp "$report_file" "$tampered_method_path_classification_report"
python3 - "$tampered_method_path_classification_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["method_path_classification_status"] = "missing"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_method_path_classification_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_method_path_classification_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/service-api-axum-ingress-live-policy.method-path-classification.tampered.json" 2>&1
)"
tampered_method_path_classification_code=$?
set -e

if [ "$tampered_method_path_classification_code" -eq 0 ]; then
  echo "expected tampered method/path classification status to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_method_path_classification_output" | grep -q 'service_api_axum_policy_marker_missing:method_path_classification_status'; then
  echo "expected deterministic mismatch reason code for tampered method/path classification status" >&2
  exit 1
fi

tampered_websocket_upgrade_parity_report="$TMP_DIR/service-api-axum-ingress-live-summary.websocket-upgrade-parity.tampered.json"
cp "$report_file" "$tampered_websocket_upgrade_parity_report"
python3 - "$tampered_websocket_upgrade_parity_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["websocket_upgrade_parity_status"] = "missing"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_websocket_upgrade_parity_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_websocket_upgrade_parity_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/service-api-axum-ingress-live-policy.websocket-upgrade-parity.tampered.json" 2>&1
)"
tampered_websocket_upgrade_parity_code=$?
set -e

if [ "$tampered_websocket_upgrade_parity_code" -eq 0 ]; then
  echo "expected tampered websocket-upgrade parity status to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_websocket_upgrade_parity_output" | grep -q 'service_api_axum_policy_marker_missing:websocket_upgrade_parity_status'; then
  echo "expected deterministic mismatch reason code for websocket-upgrade parity status tamper" >&2
  exit 1
fi

tampered_protocol_taxonomy_report="$TMP_DIR/service-api-axum-ingress-live-summary.protocol-taxonomy.tampered.json"
cp "$report_file" "$tampered_protocol_taxonomy_report"
python3 - "$tampered_protocol_taxonomy_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["protocol_compliance_reason_taxonomy_version"] = "tampered-taxonomy"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_protocol_taxonomy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_protocol_taxonomy_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/service-api-axum-ingress-live-policy.protocol-taxonomy.tampered.json" 2>&1
)"
tampered_protocol_taxonomy_code=$?
set -e

if [ "$tampered_protocol_taxonomy_code" -eq 0 ]; then
  echo "expected tampered service api protocol taxonomy to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_protocol_taxonomy_output" | grep -q 'service_api_axum_policy_protocol_compliance_reason_taxonomy_version_mismatch'; then
  echo "expected deterministic mismatch reason code for tampered protocol taxonomy" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_protocol_taxonomy_output" | grep -q '^service_api_axum_protocol_mismatch_reason_code=service_api_axum_policy_protocol_taxonomy_mismatch$'; then
  echo "expected deterministic protocol mismatch reason mapping code for protocol taxonomy tamper" >&2
  exit 1
fi

tampered_request_validation_taxonomy_report="$TMP_DIR/service-api-axum-ingress-live-summary.request-validation-taxonomy.tampered.json"
cp "$report_file" "$tampered_request_validation_taxonomy_report"
python3 - "$tampered_request_validation_taxonomy_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["request_validation_reason_taxonomy_version"] = "tampered-taxonomy"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_request_validation_taxonomy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_request_validation_taxonomy_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/service-api-axum-ingress-live-policy.request-validation-taxonomy.tampered.json" 2>&1
)"
tampered_request_validation_taxonomy_code=$?
set -e

if [ "$tampered_request_validation_taxonomy_code" -eq 0 ]; then
  echo "expected tampered request-validation taxonomy to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_request_validation_taxonomy_output" | grep -q 'service_api_axum_policy_request_validation_reason_taxonomy_version_mismatch'; then
  echo "expected deterministic mismatch reason code for tampered request-validation taxonomy" >&2
  exit 1
fi

tampered_ingress_resilience_taxonomy_report="$TMP_DIR/service-api-axum-ingress-live-summary.ingress-resilience-taxonomy.tampered.json"
cp "$report_file" "$tampered_ingress_resilience_taxonomy_report"
python3 - "$tampered_ingress_resilience_taxonomy_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["ingress_resilience_reason_taxonomy_version"] = "tampered-taxonomy"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_ingress_resilience_taxonomy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_ingress_resilience_taxonomy_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/service-api-axum-ingress-live-policy.ingress-resilience-taxonomy.tampered.json" 2>&1
)"
tampered_ingress_resilience_taxonomy_code=$?
set -e

if [ "$tampered_ingress_resilience_taxonomy_code" -eq 0 ]; then
  echo "expected tampered ingress-resilience taxonomy to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_ingress_resilience_taxonomy_output" | grep -q 'service_api_axum_policy_ingress_resilience_reason_taxonomy_version_mismatch'; then
  echo "expected deterministic mismatch reason code for ingress-resilience taxonomy tamper" >&2
  exit 1
fi

tampered_admission_taxonomy_report="$TMP_DIR/service-api-axum-ingress-live-summary.admission-taxonomy.tampered.json"
cp "$report_file" "$tampered_admission_taxonomy_report"
python3 - "$tampered_admission_taxonomy_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["admission_reason_taxonomy_version"] = "tampered-taxonomy"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_admission_taxonomy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_admission_taxonomy_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/service-api-axum-ingress-live-policy.admission-taxonomy.tampered.json" 2>&1
)"
tampered_admission_taxonomy_code=$?
set -e

if [ "$tampered_admission_taxonomy_code" -eq 0 ]; then
  echo "expected tampered admission taxonomy to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_admission_taxonomy_output" | grep -q 'service_api_axum_policy_admission_reason_taxonomy_version_mismatch'; then
  echo "expected deterministic mismatch reason code for admission taxonomy tamper" >&2
  exit 1
fi

tampered_admission_decision_taxonomy_report="$TMP_DIR/service-api-axum-ingress-live-summary.admission-decision-taxonomy.tampered.json"
cp "$report_file" "$tampered_admission_decision_taxonomy_report"
python3 - "$tampered_admission_decision_taxonomy_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["admission_decision_reason_taxonomy_version"] = "tampered-taxonomy"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_admission_decision_taxonomy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_admission_decision_taxonomy_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/service-api-axum-ingress-live-policy.admission-decision-taxonomy.tampered.json" 2>&1
)"
tampered_admission_decision_taxonomy_code=$?
set -e

if [ "$tampered_admission_decision_taxonomy_code" -eq 0 ]; then
  echo "expected tampered admission decision taxonomy to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_admission_decision_taxonomy_output" | grep -q 'service_api_axum_policy_admission_decision_reason_taxonomy_version_mismatch'; then
  echo "expected deterministic mismatch reason code for admission decision taxonomy tamper" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_admission_decision_taxonomy_output" | grep -q '^service_api_axum_protocol_mismatch_reason_code=service_api_axum_policy_protocol_taxonomy_mismatch$'; then
  echo "expected deterministic protocol mismatch reason mapping code for admission decision taxonomy tamper" >&2
  exit 1
fi

tampered_admission_decision_reason_codes_report="$TMP_DIR/service-api-axum-ingress-live-summary.admission-decision-reason-codes.tampered.json"
cp "$report_file" "$tampered_admission_decision_reason_codes_report"
python3 - "$tampered_admission_decision_reason_codes_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["admission_decision_reason_codes_csv"] = "admission_decision_accept,admission_decision_reject"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_admission_decision_reason_codes_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_admission_decision_reason_codes_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/service-api-axum-ingress-live-policy.admission-decision-reason-codes.tampered.json" 2>&1
)"
tampered_admission_decision_reason_codes_code=$?
set -e

if [ "$tampered_admission_decision_reason_codes_code" -eq 0 ]; then
  echo "expected tampered admission decision reason codes to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_admission_decision_reason_codes_output" | grep -q 'service_api_axum_policy_admission_decision_reason_codes_csv_mismatch'; then
  echo "expected deterministic mismatch reason code for admission decision reason-codes tamper" >&2
  exit 1
fi

tampered_admission_decision_defer_status_report="$TMP_DIR/service-api-axum-ingress-live-summary.admission-decision-defer-status.tampered.json"
cp "$report_file" "$tampered_admission_decision_defer_status_report"
python3 - "$tampered_admission_decision_defer_status_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["admission_decision_defer_status"] = "missing"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_admission_decision_defer_status_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_admission_decision_defer_status_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/service-api-axum-ingress-live-policy.admission-decision-defer-status.tampered.json" 2>&1
)"
tampered_admission_decision_defer_status_code=$?
set -e

if [ "$tampered_admission_decision_defer_status_code" -eq 0 ]; then
  echo "expected tampered admission decision defer status to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_admission_decision_defer_status_output" | grep -q 'service_api_axum_policy_marker_missing:admission_decision_defer_status'; then
  echo "expected deterministic mismatch reason code for admission decision defer status tamper" >&2
  exit 1
fi

tampered_lifecycle_taxonomy_report="$TMP_DIR/service-api-axum-ingress-live-summary.lifecycle-taxonomy.tampered.json"
cp "$report_file" "$tampered_lifecycle_taxonomy_report"
python3 - "$tampered_lifecycle_taxonomy_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["service_api_lifecycle_rejection_reason_taxonomy_version"] = "tampered-taxonomy"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_lifecycle_taxonomy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_lifecycle_taxonomy_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/service-api-axum-ingress-live-policy.lifecycle-taxonomy.tampered.json" 2>&1
)"
tampered_lifecycle_taxonomy_code=$?
set -e

if [ "$tampered_lifecycle_taxonomy_code" -eq 0 ]; then
  echo "expected tampered lifecycle taxonomy to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_lifecycle_taxonomy_output" | grep -q 'service_api_axum_policy_lifecycle_rejection_reason_taxonomy_version_mismatch'; then
  echo "expected deterministic mismatch reason code for lifecycle taxonomy tamper" >&2
  exit 1
fi

missing_lifecycle_reason_codes_csv_report="$TMP_DIR/service-api-axum-ingress-live-summary.lifecycle-reason-csv.missing.json"
cp "$report_file" "$missing_lifecycle_reason_codes_csv_report"
python3 - "$missing_lifecycle_reason_codes_csv_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload.pop("service_api_lifecycle_rejection_reason_codes_csv", None)
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
missing_lifecycle_reason_codes_csv_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$missing_lifecycle_reason_codes_csv_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/service-api-axum-ingress-live-policy.lifecycle-reason-csv.missing.json" 2>&1
)"
missing_lifecycle_reason_codes_csv_code=$?
set -e

if [ "$missing_lifecycle_reason_codes_csv_code" -eq 0 ]; then
  echo "expected missing lifecycle reason taxonomy csv field to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$missing_lifecycle_reason_codes_csv_output" | grep -q 'service_api_axum_policy_required_field_missing:service_api_lifecycle_rejection_reason_codes_csv'; then
  echo "expected deterministic required-field reason code for missing lifecycle reason taxonomy csv field" >&2
  exit 1
fi
if ! printf '%s\n' "$missing_lifecycle_reason_codes_csv_output" | grep -q '^reason_codes_value=.*service_api_axum_policy_required_field_missing:service_api_lifecycle_rejection_reason_codes_csv'; then
  echo "expected normalized reason_codes_value output to include missing lifecycle taxonomy csv reason code" >&2
  exit 1
fi

tampered_async_backpressure_projection_report="$TMP_DIR/service-api-axum-ingress-live-summary.async-backpressure-projection.tampered.json"
cp "$report_file" "$tampered_async_backpressure_projection_report"
python3 - "$tampered_async_backpressure_projection_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["async_lifecycle_backpressure_projection_status"] = "missing"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_async_backpressure_projection_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_async_backpressure_projection_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/service-api-axum-ingress-live-policy.async-backpressure-projection.tampered.json" 2>&1
)"
tampered_async_backpressure_projection_code=$?
set -e

if [ "$tampered_async_backpressure_projection_code" -eq 0 ]; then
  echo "expected tampered async lifecycle backpressure projection status to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_async_backpressure_projection_output" | grep -q 'service_api_axum_policy_marker_missing:async_lifecycle_backpressure_projection_status'; then
  echo "expected deterministic mismatch reason code for async lifecycle backpressure projection status tamper" >&2
  exit 1
fi

tampered_admission_inflight_budget_limit_report="$TMP_DIR/service-api-axum-ingress-live-summary.admission-inflight-budget-limit.tampered.json"
cp "$report_file" "$tampered_admission_inflight_budget_limit_report"
python3 - "$tampered_admission_inflight_budget_limit_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["admission_inflight_budget_limit"] = 31
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_admission_inflight_budget_limit_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_admission_inflight_budget_limit_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/service-api-axum-ingress-live-policy.admission-inflight-budget-limit.tampered.json" 2>&1
)"
tampered_admission_inflight_budget_limit_code=$?
set -e

if [ "$tampered_admission_inflight_budget_limit_code" -eq 0 ]; then
  echo "expected tampered service api in-flight budget limit to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_admission_inflight_budget_limit_output" | grep -q 'service_api_axum_policy_admission_inflight_budget_limit_mismatch'; then
  echo "expected deterministic mismatch reason code for tampered in-flight budget limit" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_admission_inflight_budget_limit_output" | grep -q '^service_api_axum_protocol_mismatch_reason_code=service_api_axum_policy_limit_contract_mismatch$'; then
  echo "expected deterministic protocol mismatch reason mapping code for in-flight budget limit tamper" >&2
  exit 1
fi

tampered_admission_queue_budget_limit_report="$TMP_DIR/service-api-axum-ingress-live-summary.admission-queue-budget-limit.tampered.json"
cp "$report_file" "$tampered_admission_queue_budget_limit_report"
python3 - "$tampered_admission_queue_budget_limit_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["admission_queue_budget_limit"] = 2
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_admission_queue_budget_limit_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_admission_queue_budget_limit_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/service-api-axum-ingress-live-policy.admission-queue-budget-limit.tampered.json" 2>&1
)"
tampered_admission_queue_budget_limit_code=$?
set -e

if [ "$tampered_admission_queue_budget_limit_code" -eq 0 ]; then
  echo "expected tampered service api queue budget limit to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_admission_queue_budget_limit_output" | grep -q 'service_api_axum_policy_admission_queue_budget_limit_mismatch'; then
  echo "expected deterministic mismatch reason code for tampered queue budget limit" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_admission_queue_budget_limit_output" | grep -q '^service_api_axum_protocol_mismatch_reason_code=service_api_axum_policy_limit_contract_mismatch$'; then
  echo "expected deterministic protocol mismatch reason mapping code for queue budget limit tamper" >&2
  exit 1
fi

tampered_concurrency_limit_report="$TMP_DIR/service-api-axum-ingress-live-summary.concurrency-limit.tampered.json"
cp "$report_file" "$tampered_concurrency_limit_report"
python3 - "$tampered_concurrency_limit_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["api_concurrency_limit_default"] = 31
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_concurrency_limit_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_concurrency_limit_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/service-api-axum-ingress-live-policy.concurrency-limit.tampered.json" 2>&1
)"
tampered_concurrency_limit_code=$?
set -e

if [ "$tampered_concurrency_limit_code" -eq 0 ]; then
  echo "expected tampered service api concurrency-limit default to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_concurrency_limit_output" | grep -q 'service_api_axum_policy_api_concurrency_limit_default_mismatch'; then
  echo "expected deterministic mismatch reason code for tampered concurrency-limit default" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_concurrency_limit_output" | grep -q '^service_api_axum_protocol_mismatch_reason_code=service_api_axum_policy_limit_contract_mismatch$'; then
  echo "expected deterministic protocol mismatch reason mapping code for concurrency-limit default tamper" >&2
  exit 1
fi

tampered_rate_limit_report="$TMP_DIR/service-api-axum-ingress-live-summary.rate-limit.tampered.json"
cp "$report_file" "$tampered_rate_limit_report"
python3 - "$tampered_rate_limit_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["api_rate_limit_per_second_default"] = 119
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_rate_limit_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_rate_limit_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/service-api-axum-ingress-live-policy.rate-limit.tampered.json" 2>&1
)"
tampered_rate_limit_code=$?
set -e

if [ "$tampered_rate_limit_code" -eq 0 ]; then
  echo "expected tampered service api rate-limit default to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_rate_limit_output" | grep -q 'service_api_axum_policy_api_rate_limit_per_second_default_mismatch'; then
  echo "expected deterministic mismatch reason code for tampered rate-limit default" >&2
  exit 1
fi

tampered_threshold_report="$TMP_DIR/service-api-axum-ingress-live-summary.threshold.tampered.json"
cp "$report_file" "$tampered_threshold_report"
python3 - "$tampered_threshold_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["body_size_limit_bytes"] = 65535
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_threshold_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_threshold_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/service-api-axum-ingress-live-policy.threshold.tampered.json" 2>&1
)"
tampered_threshold_code=$?
set -e

if [ "$tampered_threshold_code" -eq 0 ]; then
  echo "expected tampered service api body-size threshold to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_threshold_output" | grep -q 'service_api_axum_policy_body_size_limit_mismatch'; then
  echo "expected deterministic mismatch reason code for tampered body-size threshold" >&2
  exit 1
fi

budget_multi_mismatch_report="$TMP_DIR/service-api-axum-ingress-live-summary.budget-multi-mismatch.json"
cp "$report_file" "$budget_multi_mismatch_report"
python3 - "$budget_multi_mismatch_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["admission_inflight_budget_limit"] = 31
payload["admission_queue_budget_limit"] = 2
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
budget_multi_mismatch_output_first="$(
  bash "$POLICY_CHECKER" \
    --report-file "$budget_multi_mismatch_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/service-api-axum-ingress-live-policy.budget-multi-mismatch.first.json" 2>&1
)"
budget_multi_mismatch_code_first=$?
budget_multi_mismatch_output_second="$(
  bash "$POLICY_CHECKER" \
    --report-file "$budget_multi_mismatch_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/service-api-axum-ingress-live-policy.budget-multi-mismatch.second.json" 2>&1
)"
budget_multi_mismatch_code_second=$?
set -e

if [ "$budget_multi_mismatch_code_first" -eq 0 ] || [ "$budget_multi_mismatch_code_second" -eq 0 ]; then
  echo "expected admission budget multi-mismatch report to fail policy checker deterministically" >&2
  exit 1
fi
if ! printf '%s\n' "$budget_multi_mismatch_output_first" | grep -q '^service_api_axum_protocol_mismatch_reason_code=service_api_axum_policy_limit_contract_mismatch$'; then
  echo "expected deterministic limit-contract mapped reason code on first admission budget multi-mismatch run" >&2
  exit 1
fi
if ! printf '%s\n' "$budget_multi_mismatch_output_second" | grep -q '^service_api_axum_protocol_mismatch_reason_code=service_api_axum_policy_limit_contract_mismatch$'; then
  echo "expected deterministic limit-contract mapped reason code on second admission budget multi-mismatch run" >&2
  exit 1
fi

python3 - \
  "$TMP_DIR/service-api-axum-ingress-live-policy.budget-multi-mismatch.first.json" \
  "$TMP_DIR/service-api-axum-ingress-live-policy.budget-multi-mismatch.second.json" <<'PY'
import json
import pathlib
import sys

first = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
second = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
first_reasons = first.get("reason_codes")
second_reasons = second.get("reason_codes")
if not first_reasons:
    raise SystemExit("expected non-empty first reason_codes for admission budget multi-mismatch output")
if first_reasons != second_reasons:
    raise SystemExit("expected deterministic admission budget reason-code ordering across repeated multi-mismatch runs")
if "service_api_axum_policy_admission_inflight_budget_limit_mismatch" not in first_reasons:
    raise SystemExit("expected in-flight budget mismatch reason in admission budget multi-mismatch output")
if "service_api_axum_policy_admission_queue_budget_limit_mismatch" not in first_reasons:
    raise SystemExit("expected queue budget mismatch reason in admission budget multi-mismatch output")
if first.get("service_api_axum_protocol_mismatch_reason_code") != "service_api_axum_policy_limit_contract_mismatch":
    raise SystemExit("expected deterministic mapped reason code for first admission budget multi-mismatch output")
if second.get("service_api_axum_protocol_mismatch_reason_code") != "service_api_axum_policy_limit_contract_mismatch":
    raise SystemExit("expected deterministic mapped reason code for second admission budget multi-mismatch output")
PY

multi_mismatch_report="$TMP_DIR/service-api-axum-ingress-live-summary.multi-mismatch.json"
cp "$report_file" "$multi_mismatch_report"
python3 - "$multi_mismatch_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["route_contract_parity_status"] = "missing"
payload["protocol_compliance_reason_taxonomy_version"] = "tampered-taxonomy"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
multi_mismatch_output_first="$(
  bash "$POLICY_CHECKER" \
    --report-file "$multi_mismatch_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/service-api-axum-ingress-live-policy.multi-mismatch.first.json" 2>&1
)"
multi_mismatch_code_first=$?
multi_mismatch_output_second="$(
  bash "$POLICY_CHECKER" \
    --report-file "$multi_mismatch_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/service-api-axum-ingress-live-policy.multi-mismatch.second.json" 2>&1
)"
multi_mismatch_code_second=$?
set -e

if [ "$multi_mismatch_code_first" -eq 0 ] || [ "$multi_mismatch_code_second" -eq 0 ]; then
  echo "expected multi-mismatch protocol marker drift to fail policy checker deterministically" >&2
  exit 1
fi
if ! printf '%s\n' "$multi_mismatch_output_first" | grep -q '^service_api_axum_protocol_mismatch_reason_code=service_api_axum_policy_marker_missing$'; then
  echo "expected deterministic mapped reason code on first multi-mismatch run" >&2
  exit 1
fi
if ! printf '%s\n' "$multi_mismatch_output_second" | grep -q '^service_api_axum_protocol_mismatch_reason_code=service_api_axum_policy_marker_missing$'; then
  echo "expected deterministic mapped reason code on second multi-mismatch run" >&2
  exit 1
fi

python3 - \
  "$TMP_DIR/service-api-axum-ingress-live-policy.multi-mismatch.first.json" \
  "$TMP_DIR/service-api-axum-ingress-live-policy.multi-mismatch.second.json" <<'PY'
import json
import pathlib
import sys

first = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
second = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
first_reasons = first.get("reason_codes")
second_reasons = second.get("reason_codes")
if not first_reasons:
    raise SystemExit("expected non-empty first reason_codes for multi-mismatch output")
if first_reasons != second_reasons:
    raise SystemExit("expected deterministic reason-code ordering across repeated multi-mismatch runs")
if "service_api_axum_policy_marker_missing:route_contract_parity_status" not in first_reasons:
    raise SystemExit("expected route-contract parity marker mismatch reason in multi-mismatch output")
if "service_api_axum_policy_protocol_compliance_reason_taxonomy_version_mismatch" not in first_reasons:
    raise SystemExit("expected protocol taxonomy mismatch reason in multi-mismatch output")
if first.get("service_api_axum_protocol_mismatch_reason_code") != "service_api_axum_policy_marker_missing":
    raise SystemExit("expected deterministic mapped reason code for first multi-mismatch output")
if second.get("service_api_axum_protocol_mismatch_reason_code") != "service_api_axum_policy_marker_missing":
    raise SystemExit("expected deterministic mapped reason code for second multi-mismatch output")
PY

echo "service api axum ingress live policy checker tests passed."
