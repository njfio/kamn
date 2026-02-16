#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
# Shared runner for service-api contract lanes.
source "$ROOT_DIR/scripts/runtime/service_api_contract_lane_runner.sh"

VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_service_api_axum_ingress_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_service_api_axum_ingress_live_policy.sh"
STRATEGY_DOC="$ROOT_DIR/docs/ci/strategy.md"
ROADMAP_DOC="$ROOT_DIR/docs/plans/2026-02-08-production-service-roadmap.md"

LANE_LABEL="service api axum ingress"
LANE_SLUG="service-api-axum-ingress-live"
MAX_SECONDS_ENV="KAMN_SERVICE_API_AXUM_INGRESS_CONTRACT_MAX_SECONDS"
MAX_SECONDS_DEFAULT="180"
CONTRACT_STATUS_KEY="service_api_axum_ingress_contract_status"
POLICY_STATUS_KEY="service_api_axum_ingress_policy_status"
SUMMARY_SCHEMA="kamn.runtime.service-api-axum-ingress-live-validation.v1"
POLICY_SCHEMA="kamn.runtime.service-api-axum-ingress-live-policy-report.v1"
LANE_REPORT_SCHEMA="kamn.runtime.service-api-axum-ingress-live-contract-lane-report.v1"
TAMPER_FIELD="method_path_classification_status"
TAMPER_REASON_CODE="service_api_axum_policy_marker_missing:method_path_classification_status"
ROADMAP_TASK_MARKER="Task #3308"
ROADMAP_CONTRACT_SCRIPT_REF="scripts/runtime/validate_service_api_axum_ingress_live_contract_lane.sh"
ROADMAP_POLICY_SCRIPT_REF="scripts/runtime/check_service_api_axum_ingress_live_policy.sh"
ALLOW_MODE="0"

VALIDATION_REQUIRED_MARKERS=(
  "status=pass"
  "final_decision=GO"
  "keep_alive_status=verified"
  "body_size_guard_status=verified"
  "concurrency_status=verified"
  "websocket_status=verified"
  "ingress_limit_config_status=verified"
  "docs_ingress_limit_matrix_status=verified"
  "request_validation_status=verified"
  "error_envelope_field_status=verified"
  "method_path_classification_status=verified"
  "protocol_compliance_status=verified"
  "route_contract_parity_status=verified"
  "protocol_compliance_reason_taxonomy_version=kamn.runtime.service-api-protocol-compliance-reason-taxonomy.v1"
  "protocol_compliance_reason_codes_csv=method_path_contract_mismatch,payload_shape_contract_mismatch,route_contract_bypass_detected"
  "request_validation_reason_registry_status=verified"
  "error_envelope_source_contract_status=verified"
  "request_validation_reason_taxonomy_version=kamn.runtime.service-api-request-validation-reason-taxonomy.v1"
  "request_validation_reason_codes_csv=service_api_ws_upgrade_header_missing,service_api_ws_version_header_invalid,service_api_method_not_allowed,service_api_route_not_found,service_api_payload_json_syntax_invalid,service_api_payload_structure_invalid"
  "error_envelope_reason_taxonomy_version=kamn.runtime.service-api-error-envelope-reason-taxonomy.v1"
  "error_envelope_reason_codes_csv=service_api_ws_upgrade_header_missing,service_api_method_not_allowed,service_api_route_not_found"
)
VALIDATION_REQUIRED_REGEX_MARKERS=(
  '^api_max_requests_default=[1-9][0-9]*$'
  '^api_idle_timeout_default_ms=[1-9][0-9]*$'
  '^body_size_limit_bytes=[1-9][0-9]*$'
  '^api_concurrency_limit_default=[1-9][0-9]*$'
  '^api_rate_limit_per_second_default=[1-9][0-9]*$'
)
POLICY_REQUIRED_MARKERS=(
  "status=ok"
  "final_decision=GO"
  "service_api_axum_ingress_policy_status=verified"
)
STRATEGY_REQUIRED_REFS=(
  "validate_service_api_axum_ingress_live.sh"
  "check_service_api_axum_ingress_live_policy.sh"
  "validate_service_api_axum_ingress_live_contract_lane.sh"
  "test_validate_service_api_axum_ingress_live_contract_lane.sh"
  "test_check_service_api_axum_ingress_live_policy.sh"
)
STRATEGY_REQUIRED_MARKERS=(
  "service api axum ingress run-mode commands remain excluded from ci-fast-gate and ci-tools fast mode."
  "ingress limit config matrix defaults remain parity-checked against source constants and API docs"
  "request-validation and error-envelope taxonomy parity remains deterministic via:"
)
LANE_REPORT_SUMMARY_FIELDS=(
  ingress_limit_config_status
  docs_ingress_limit_matrix_status
  request_validation_status
  error_envelope_field_status
  method_path_classification_status
  protocol_compliance_status
  route_contract_parity_status
  protocol_compliance_reason_taxonomy_version
  protocol_compliance_reason_codes_csv
  request_validation_reason_registry_status
  error_envelope_source_contract_status
  request_validation_reason_taxonomy_version
  request_validation_reason_codes_csv
  error_envelope_reason_taxonomy_version
  error_envelope_reason_codes_csv
  api_max_requests_default
  api_idle_timeout_default_ms
  body_size_limit_bytes
  api_concurrency_limit_default
  api_rate_limit_per_second_default
)
OUTPUT_SUMMARY_FIELDS=(
  ingress_limit_config_status
  docs_ingress_limit_matrix_status
  request_validation_status
  error_envelope_field_status
  method_path_classification_status
  protocol_compliance_status
  route_contract_parity_status
  protocol_compliance_reason_taxonomy_version
  protocol_compliance_reason_codes_csv
  request_validation_reason_registry_status
  error_envelope_source_contract_status
  request_validation_reason_taxonomy_version
  request_validation_reason_codes_csv
  error_envelope_reason_taxonomy_version
  error_envelope_reason_codes_csv
  api_max_requests_default
  api_idle_timeout_default_ms
  body_size_limit_bytes
  api_concurrency_limit_default
  api_rate_limit_per_second_default
)

service_api_contract_lane_run "$@"
