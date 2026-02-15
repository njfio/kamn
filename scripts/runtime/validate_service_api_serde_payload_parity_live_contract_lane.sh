#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
# Shared runner for service-api contract lanes.
source "$ROOT_DIR/scripts/runtime/service_api_contract_lane_runner.sh"

VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_service_api_serde_payload_parity_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_service_api_serde_payload_parity_live_policy.sh"
STRATEGY_DOC="$ROOT_DIR/docs/ci/strategy.md"
ROADMAP_DOC="$ROOT_DIR/docs/plans/2026-02-08-production-service-roadmap.md"

LANE_LABEL="service api serde payload parity"
LANE_SLUG="service-api-serde-payload-parity-live"
MAX_SECONDS_ENV="KAMN_SERVICE_API_SERDE_PAYLOAD_CONTRACT_MAX_SECONDS"
MAX_SECONDS_DEFAULT="240"
CONTRACT_STATUS_KEY="service_api_serde_payload_contract_status"
POLICY_STATUS_KEY="service_api_serde_payload_policy_status"
SUMMARY_SCHEMA="kamn.runtime.service-api-serde-payload-parity-live-validation.v1"
POLICY_SCHEMA="kamn.runtime.service-api-serde-payload-parity-live-policy-report.v1"
LANE_REPORT_SCHEMA="kamn.runtime.service-api-serde-payload-parity-live-contract-lane-report.v1"
TAMPER_FIELD="route_payload_parity_status"
TAMPER_REASON_CODE="service_api_serde_payload_policy_marker_missing:route_payload_parity_status"
ROADMAP_TASK_MARKER="Task #3267"
ROADMAP_CONTRACT_SCRIPT_REF="scripts/runtime/validate_service_api_serde_payload_parity_live_contract_lane.sh"
ROADMAP_POLICY_SCRIPT_REF="scripts/runtime/check_service_api_serde_payload_parity_live_policy.sh"
ALLOW_MODE="0"

VALIDATION_REQUIRED_MARKERS=(
  "status=pass"
  "final_decision=GO"
  "serde_dto_status=verified"
  "serde_serializer_status=verified"
  "serde_parser_status=verified"
  "route_payload_parity_status=verified"
  "fail_closed_status=verified"
)
VALIDATION_REQUIRED_REGEX_MARKERS=()
POLICY_REQUIRED_MARKERS=(
  "status=ok"
  "final_decision=GO"
  "service_api_serde_payload_policy_status=verified"
)
STRATEGY_REQUIRED_REFS=(
  "validate_service_api_serde_payload_parity_live.sh"
  "check_service_api_serde_payload_parity_live_policy.sh"
  "validate_service_api_serde_payload_parity_live_contract_lane.sh"
  "test_validate_service_api_serde_payload_parity_live_contract_lane.sh"
  "test_check_service_api_serde_payload_parity_live_policy.sh"
)
STRATEGY_REQUIRED_MARKERS=(
  "service api serde payload parity contract-lane commands remain excluded from ci-fast-gate and ci-tools fast mode."
)
LANE_REPORT_SUMMARY_FIELDS=()
OUTPUT_SUMMARY_FIELDS=()

service_api_contract_lane_run "$@"
