#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
# Shared runner for service-api contract lanes.
source "$ROOT_DIR/scripts/runtime/service_api_contract_lane_runner.sh"

VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_service_api_websocket_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_service_api_websocket_live_policy.sh"
STRATEGY_DOC="$ROOT_DIR/docs/ci/strategy.md"
ROADMAP_DOC="$ROOT_DIR/docs/plans/2026-02-08-production-service-roadmap.md"

LANE_LABEL="service api websocket live"
LANE_SLUG="service-api-websocket-live"
MAX_SECONDS_ENV="KAMN_SERVICE_API_WEBSOCKET_CONTRACT_MAX_SECONDS"
MAX_SECONDS_DEFAULT="180"
CONTRACT_STATUS_KEY="service_api_websocket_contract_status"
POLICY_STATUS_KEY="service_api_websocket_policy_status"
SUMMARY_SCHEMA="kamn.runtime.service-api-websocket-live-validation.v1"
POLICY_SCHEMA="kamn.runtime.service-api-websocket-live-policy-report.v1"
LANE_REPORT_SCHEMA="kamn.runtime.service-api-websocket-live-contract-lane-report.v1"
TAMPER_FIELD="websocket_session_lifecycle_status"
TAMPER_REASON_CODE="service_api_websocket_policy_marker_missing:websocket_session_lifecycle_status"
ROADMAP_TASK_MARKER="Task #2918"
ROADMAP_CONTRACT_SCRIPT_REF="scripts/runtime/validate_service_api_websocket_live_contract_lane.sh"
ROADMAP_POLICY_SCRIPT_REF="scripts/runtime/check_service_api_websocket_live_policy.sh"
ALLOW_MODE="0"

VALIDATION_REQUIRED_MARKERS=(
  "status=pass"
  "final_decision=GO"
  "websocket_upgrade_status=verified"
  "websocket_session_lifecycle_status=verified"
  "websocket_heartbeat_timeout_status=verified"
  "websocket_idle_timeout_contract_status=verified"
  "fail_closed_status=verified"
  "probe_status=verified"
  "websocket_reason_registry_status=verified"
  "protocol_session_docs_contract_status=verified"
  "service_api_protocol_session_reason_taxonomy_version=kamn.runtime.service-api.protocol-session-reason-taxonomy.v1"
  "service_api_protocol_session_reason_codes_csv=service_api_ws_upgrade_header_missing,service_api_ws_connection_header_missing,service_api_ws_key_header_missing,service_api_ws_version_header_missing,service_api_ws_upgrade_header_invalid,service_api_ws_connection_header_invalid,service_api_ws_key_header_empty,service_api_ws_version_header_invalid,service_api_payload_json_syntax_invalid,service_api_payload_structure_invalid,service_api_payload_io_error,service_api_auth_replay_nonce_detected,service_api_websocket_upgrade_required,service_api_protocol_session_docs_marker_missing"
  "websocket_lifecycle_reason_taxonomy_version=kamn.runtime.service-api-websocket-lifecycle-reason-taxonomy.v1"
  "websocket_lifecycle_reason_codes_csv=service_api_ws_upgrade_header_missing,service_api_ws_version_header_invalid,service_api_auth_sender_did_header_missing,service_api_ws_connection_header_missing,service_api_ws_key_header_missing"
)
VALIDATION_REQUIRED_REGEX_MARKERS=(
  '^api_idle_timeout_default_ms=[1-9][0-9]*$'
)
POLICY_REQUIRED_MARKERS=(
  "status=ok"
  "final_decision=GO"
  "service_api_websocket_policy_status=verified"
  "reason_codes_value=none"
)
STRATEGY_REQUIRED_REFS=(
  "validate_service_api_websocket_live.sh"
  "check_service_api_websocket_live_policy.sh"
  "validate_service_api_websocket_live_contract_lane.sh"
  "test_validate_service_api_websocket_live.sh"
  "test_check_service_api_websocket_live_policy.sh"
  "test_validate_service_api_websocket_live_contract_lane.sh"
)
STRATEGY_REQUIRED_MARKERS=(
  "websocket lifecycle governance remains deterministic via:"
  "service api websocket live contract-lane commands remain excluded from ci-fast-gate and ci-tools fast mode."
)
LANE_REPORT_SUMMARY_FIELDS=(
  websocket_upgrade_status
  websocket_session_lifecycle_status
  websocket_heartbeat_timeout_status
  websocket_idle_timeout_contract_status
  websocket_reason_registry_status
  protocol_session_docs_contract_status
  service_api_protocol_session_reason_taxonomy_version
  service_api_protocol_session_reason_codes_csv
  websocket_lifecycle_reason_taxonomy_version
  websocket_lifecycle_reason_codes_csv
  api_idle_timeout_default_ms
)
OUTPUT_SUMMARY_FIELDS=(
  websocket_upgrade_status
  websocket_session_lifecycle_status
  websocket_heartbeat_timeout_status
  websocket_idle_timeout_contract_status
  websocket_reason_registry_status
  protocol_session_docs_contract_status
  service_api_protocol_session_reason_taxonomy_version
  service_api_protocol_session_reason_codes_csv
  websocket_lifecycle_reason_taxonomy_version
  websocket_lifecycle_reason_codes_csv
  api_idle_timeout_default_ms
)

service_api_contract_lane_run "$@"
