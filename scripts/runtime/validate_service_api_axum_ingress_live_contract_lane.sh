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
TAMPER_FIELD="concurrency_status"
TAMPER_REASON_CODE="service_api_axum_policy_marker_missing:concurrency_status"
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
)
LANE_REPORT_SUMMARY_FIELDS=(
  ingress_limit_config_status
  docs_ingress_limit_matrix_status
  api_max_requests_default
  api_idle_timeout_default_ms
  body_size_limit_bytes
  api_concurrency_limit_default
  api_rate_limit_per_second_default
)
OUTPUT_SUMMARY_FIELDS=(
  ingress_limit_config_status
  docs_ingress_limit_matrix_status
  api_max_requests_default
  api_idle_timeout_default_ms
  body_size_limit_bytes
  api_concurrency_limit_default
  api_rate_limit_per_second_default
)

service_api_contract_lane_run "$@"
