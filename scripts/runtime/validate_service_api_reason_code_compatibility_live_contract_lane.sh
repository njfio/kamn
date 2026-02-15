#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
# Shared runner for service-api contract lanes.
source "$ROOT_DIR/scripts/runtime/service_api_contract_lane_runner.sh"

VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_service_api_reason_code_compatibility_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_service_api_reason_code_compatibility_live_policy.sh"
STRATEGY_DOC="$ROOT_DIR/docs/ci/strategy.md"
ROADMAP_DOC="$ROOT_DIR/docs/plans/2026-02-08-production-service-roadmap.md"

LANE_LABEL="service api reason-code compatibility"
LANE_SLUG="service-api-reason-code-compatibility-live"
MAX_SECONDS_ENV="KAMN_SERVICE_API_REASON_CODE_CONTRACT_MAX_SECONDS"
MAX_SECONDS_DEFAULT="240"
CONTRACT_STATUS_KEY="service_api_reason_code_contract_status"
POLICY_STATUS_KEY="service_api_reason_code_policy_status"
SUMMARY_SCHEMA="kamn.runtime.service-api-reason-code-compatibility-live-validation.v1"
POLICY_SCHEMA="kamn.runtime.service-api-reason-code-compatibility-live-policy-report.v1"
LANE_REPORT_SCHEMA="kamn.runtime.service-api-reason-code-compatibility-live-contract-lane-report.v1"
TAMPER_FIELD="route_error_mapping_status"
TAMPER_REASON_CODE="service_api_reason_code_policy_marker_missing:route_error_mapping_status"
ROADMAP_TASK_MARKER="Task #3278"
ROADMAP_CONTRACT_SCRIPT_REF="scripts/runtime/validate_service_api_reason_code_compatibility_live_contract_lane.sh"
ROADMAP_POLICY_SCRIPT_REF="scripts/runtime/check_service_api_reason_code_compatibility_live_policy.sh"
ALLOW_MODE="0"

VALIDATION_REQUIRED_MARKERS=(
  "status=pass"
  "final_decision=GO"
  "reason_registry_status=verified"
  "error_envelope_field_status=verified"
  "rust_sdk_reason_code_status=verified"
  "python_sdk_reason_code_status=verified"
  "regression_corpus_status=verified"
  "regression_drift_diagnostics_status=verified"
  "route_error_mapping_status=verified"
  "replay_error_mapping_status=verified"
  "websocket_error_mapping_status=verified"
  "fail_closed_status=verified"
)
VALIDATION_REQUIRED_REGEX_MARKERS=(
  '^regression_corpus_scenario_count=[1-9][0-9]*$'
)
POLICY_REQUIRED_MARKERS=(
  "status=ok"
  "final_decision=GO"
  "service_api_reason_code_policy_status=verified"
)
STRATEGY_REQUIRED_REFS=(
  "validate_service_api_reason_code_compatibility_live.sh"
  "check_service_api_reason_code_compatibility_live_policy.sh"
  "validate_service_api_reason_code_compatibility_live_contract_lane.sh"
  "test_validate_service_api_reason_code_compatibility_live_contract_lane.sh"
  "test_check_service_api_reason_code_compatibility_live_policy.sh"
)
STRATEGY_REQUIRED_MARKERS=(
  "service api reason-code compatibility contract-lane commands remain excluded from ci-fast-gate and ci-tools fast mode."
)
LANE_REPORT_SUMMARY_FIELDS=(
  error_envelope_field_status
  rust_sdk_reason_code_status
  python_sdk_reason_code_status
  regression_corpus_status
  regression_drift_diagnostics_status
  regression_corpus_scenario_count
)
OUTPUT_SUMMARY_FIELDS=(
  error_envelope_field_status
  rust_sdk_reason_code_status
  python_sdk_reason_code_status
  regression_corpus_status
  regression_drift_diagnostics_status
  regression_corpus_scenario_count
)

service_api_contract_lane_run "$@"
