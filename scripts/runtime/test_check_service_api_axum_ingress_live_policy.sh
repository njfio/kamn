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
  "overload_evidence_normalization_status": "verified",
  "protocol_compliance_reason_taxonomy_version": "kamn.runtime.service-api-protocol-compliance-reason-taxonomy.v1",
  "protocol_compliance_reason_codes_csv": "method_path_contract_mismatch,payload_shape_contract_mismatch,route_contract_bypass_detected",
  "ingress_resilience_reason_taxonomy_version": "kamn.runtime.service-api-ingress-resilience-reason-taxonomy.v1",
  "ingress_resilience_reason_codes_csv": "ingress_readiness_progress_stalled,websocket_upgrade_parity_mismatch,ci_local_promotion_budget_boundary_exceeded",
  "admission_reason_taxonomy_version": "kamn.runtime.service-api-admission-reason-taxonomy.v1",
  "admission_reason_codes_csv": "admission_queue_saturation_detected,admission_queue_cap_bypass_detected,admission_evidence_normalization_drift",
  "request_validation_reason_registry_status": "verified",
  "error_envelope_source_contract_status": "verified",
  "request_validation_reason_taxonomy_version": "kamn.runtime.service-api-request-validation-reason-taxonomy.v1",
  "request_validation_reason_codes_csv": "service_api_ws_upgrade_header_missing,service_api_ws_version_header_invalid,service_api_method_not_allowed,service_api_route_not_found,service_api_payload_json_syntax_invalid,service_api_payload_structure_invalid",
  "error_envelope_reason_taxonomy_version": "kamn.runtime.service-api-error-envelope-reason-taxonomy.v1",
  "error_envelope_reason_codes_csv": "service_api_ws_upgrade_header_missing,service_api_method_not_allowed,service_api_route_not_found",
  "api_max_requests_default": 1,
  "api_idle_timeout_default_ms": 5000,
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
if payload.get("overload_evidence_normalization_status") != "verified":
    raise SystemExit("expected deterministic overload_evidence_normalization_status marker")
if payload.get("admission_reason_taxonomy_version") != "kamn.runtime.service-api-admission-reason-taxonomy.v1":
    raise SystemExit("expected deterministic admission_reason_taxonomy_version marker")
if payload.get("admission_reason_codes_csv") != "admission_queue_saturation_detected,admission_queue_cap_bypass_detected,admission_evidence_normalization_drift":
    raise SystemExit("expected deterministic admission_reason_codes_csv marker")
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

echo "service api axum ingress live policy checker tests passed."
