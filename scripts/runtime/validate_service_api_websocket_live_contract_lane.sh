#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
# Shared runner for service-api contract lanes.
source "$ROOT_DIR/scripts/runtime/service_api_contract_lane_runner.sh"

VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_service_api_websocket_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_service_api_websocket_live_policy.sh"
EVIDENCE_CHECKER="$ROOT_DIR/scripts/runtime/check_service_api_websocket_live_evidence_convergence.sh"
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
EVIDENCE_REPORT_SCHEMA="kamn.runtime.service-api-websocket-live-convergence-report.v1"
EVIDENCE_CONVERGENCE_STATUS_KEY="service_api_websocket_evidence_convergence_status"
EVIDENCE_REASON_TAXONOMY_VERSION="kamn.runtime.service-api-websocket-evidence-convergence-reason-taxonomy.v1"
EVIDENCE_REASON_CODES_CSV="service_api_websocket_evidence_link_missing,service_api_websocket_evidence_payload_tamper_detected,service_api_websocket_promotion_decision_reason_mapping_mismatch"
PROMOTION_DECISION_REASON_TAXONOMY_VERSION="kamn.runtime.service-api-websocket-promotion-decision-reason-taxonomy.v1"
PROMOTION_DECISION_REASON_CODES_CSV="service_api_websocket_policy_required_field_missing,service_api_websocket_policy_marker_missing,service_api_websocket_policy_reason_taxonomy_mismatch,service_api_websocket_policy_idle_timeout_contract_mismatch,ci_fast_gate_failed,service_api_websocket_policy_expected_decision_mismatch,service_api_websocket_policy_violation"
EVIDENCE_TAMPER_FIELD="promotion_decision_reason_code"
EVIDENCE_TAMPER_REASON_CODE="service_api_websocket_promotion_decision_reason_mapping_mismatch"

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
  "check_service_api_websocket_live_evidence_convergence.sh"
  "validate_service_api_websocket_live_contract_lane.sh"
  "test_validate_service_api_websocket_live.sh"
  "test_check_service_api_websocket_live_policy.sh"
  "test_validate_service_api_websocket_live_contract_lane.sh"
)
STRATEGY_REQUIRED_MARKERS=(
  "websocket lifecycle governance remains deterministic via:"
  "websocket evidence convergence governance remains deterministic via:"
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

EVIDENCE_REQUIRED_MARKERS=(
  "status=ok"
  "final_decision=GO"
  "evidence_convergence_status=verified"
  "promotion_decision_reason_mapping_status=verified"
  "reason_taxonomy_version=${EVIDENCE_REASON_TAXONOMY_VERSION}"
  "reason_codes_csv=${EVIDENCE_REASON_CODES_CSV}"
  "reason_codes_value=none"
  "promotion_decision_reason_taxonomy_version=${PROMOTION_DECISION_REASON_TAXONOMY_VERSION}"
  "promotion_decision_reason_codes_csv=${PROMOTION_DECISION_REASON_CODES_CSV}"
  "promotion_decision_reason_code=none"
)

if [[ ! -x "$EVIDENCE_CHECKER" ]]; then
  echo "expected required executable script '$EVIDENCE_CHECKER'" >&2
  exit 1
fi

runner_output_json=""
runner_policy_output_json=""
convergence_output_json=""
runner_args=()
while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-json)
      runner_output_json="${2:-}"
      runner_args+=("$1" "${2:-}")
      shift 2
      ;;
    --policy-output-json)
      runner_policy_output_json="${2:-}"
      runner_args+=("$1" "${2:-}")
      shift 2
      ;;
    --convergence-output-json)
      convergence_output_json="${2:-}"
      shift 2
      ;;
    *)
      runner_args+=("$1")
      shift
      ;;
  esac
done

tmp_dir="$(mktemp -d)"
trap "rm -rf '$tmp_dir'" EXIT

if [[ -z "$runner_output_json" ]]; then
  runner_output_json="$tmp_dir/${LANE_SLUG}-contract-lane-report.json"
  runner_args+=("--output-json" "$runner_output_json")
fi
if [[ -z "$runner_policy_output_json" ]]; then
  runner_policy_output_json="$tmp_dir/${LANE_SLUG}-policy-report.json"
  runner_args+=("--policy-output-json" "$runner_policy_output_json")
fi
if [[ -z "$convergence_output_json" ]]; then
  convergence_output_json="$tmp_dir/${LANE_SLUG}-convergence-report.json"
fi

lane_output="$(service_api_contract_lane_run "${runner_args[@]}")"

convergence_output="$(
  bash "$EVIDENCE_CHECKER" \
    --report-file "$runner_output_json" \
    --policy-file "$runner_policy_output_json" \
    --output-json "$convergence_output_json"
)"

for marker in "${EVIDENCE_REQUIRED_MARKERS[@]}"; do
  if ! printf '%s\n' "$convergence_output" | grep -q "^${marker}$"; then
    echo "expected ${LANE_LABEL} evidence convergence marker ${marker}" >&2
    exit 1
  fi
done

tampered_policy_report="$tmp_dir/${LANE_SLUG}-policy.tampered-mapping.json"
cp "$runner_policy_output_json" "$tampered_policy_report"
python3 - "$tampered_policy_report" "$EVIDENCE_TAMPER_FIELD" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
field = sys.argv[2]
payload = json.loads(path.read_text(encoding="utf-8"))
payload[field] = "tampered"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_convergence_output="$(
  bash "$EVIDENCE_CHECKER" \
    --report-file "$runner_output_json" \
    --policy-file "$tampered_policy_report" \
    --output-json "$tmp_dir/${LANE_SLUG}-convergence.tampered-mapping.json" 2>&1
)"
tampered_convergence_code=$?
set -e
if [[ "$tampered_convergence_code" -eq 0 ]]; then
  echo "expected tampered ${LANE_LABEL} policy mapping to fail evidence convergence validation" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_convergence_output" | grep -q "$EVIDENCE_TAMPER_REASON_CODE"; then
  echo "expected deterministic fail-closed reason for tampered ${LANE_LABEL} mapping evidence" >&2
  exit 1
fi

python3 - \
  "$runner_output_json" \
  "$convergence_output_json" \
  "$EVIDENCE_REPORT_SCHEMA" \
  "$EVIDENCE_CONVERGENCE_STATUS_KEY" \
  "$EVIDENCE_REASON_TAXONOMY_VERSION" \
  "$EVIDENCE_REASON_CODES_CSV" \
  "$PROMOTION_DECISION_REASON_TAXONOMY_VERSION" \
  "$PROMOTION_DECISION_REASON_CODES_CSV" <<'PY'
import json
import pathlib
import sys

lane_report_file = pathlib.Path(sys.argv[1])
convergence_report_file = pathlib.Path(sys.argv[2])
expected_schema = sys.argv[3]
evidence_status_key = sys.argv[4]
expected_reason_taxonomy_version = sys.argv[5]
expected_reason_codes_csv = sys.argv[6]
expected_promotion_reason_taxonomy_version = sys.argv[7]
expected_promotion_reason_codes_csv = sys.argv[8]

lane_payload = json.loads(lane_report_file.read_text(encoding="utf-8"))
convergence_payload = json.loads(convergence_report_file.read_text(encoding="utf-8"))
if convergence_payload.get("schema_version") != expected_schema:
    raise SystemExit(
        "unexpected websocket evidence convergence report schema: "
        + str(convergence_payload.get("schema_version"))
    )
if convergence_payload.get("reason_taxonomy_version") != expected_reason_taxonomy_version:
    raise SystemExit("unexpected websocket evidence reason taxonomy marker")
if convergence_payload.get("reason_codes_csv") != expected_reason_codes_csv:
    raise SystemExit("unexpected websocket evidence reason codes marker")
if (
    convergence_payload.get("promotion_decision_reason_taxonomy_version")
    != expected_promotion_reason_taxonomy_version
):
    raise SystemExit("unexpected websocket promotion decision reason taxonomy marker")
if (
    convergence_payload.get("promotion_decision_reason_codes_csv")
    != expected_promotion_reason_codes_csv
):
    raise SystemExit("unexpected websocket promotion decision reason codes marker")

lane_payload[evidence_status_key] = convergence_payload.get("evidence_convergence_status")
lane_payload["promotion_decision_reason_mapping_status"] = convergence_payload.get(
    "promotion_decision_reason_mapping_status"
)
lane_payload["service_api_websocket_evidence_reason_taxonomy_version"] = (
    convergence_payload.get("reason_taxonomy_version")
)
lane_payload["service_api_websocket_evidence_reason_codes_csv"] = convergence_payload.get(
    "reason_codes_csv"
)
lane_payload["promotion_decision_reason_taxonomy_version"] = convergence_payload.get(
    "promotion_decision_reason_taxonomy_version"
)
lane_payload["promotion_decision_reason_codes_csv"] = convergence_payload.get(
    "promotion_decision_reason_codes_csv"
)
lane_payload["promotion_decision_reason_code"] = convergence_payload.get(
    "promotion_decision_reason_code"
)
lane_payload["service_api_websocket_evidence_report_file"] = str(convergence_report_file)

lane_report_file.write_text(
    json.dumps(lane_payload, sort_keys=True, indent=2) + "\n",
    encoding="utf-8",
)
PY

printf '%s\n' "$lane_output"

python3 - "$convergence_output_json" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
print(
    "service_api_websocket_evidence_convergence_status="
    + str(payload.get("evidence_convergence_status"))
)
print(
    "promotion_decision_reason_mapping_status="
    + str(payload.get("promotion_decision_reason_mapping_status"))
)
print(
    "service_api_websocket_evidence_reason_taxonomy_version="
    + str(payload.get("reason_taxonomy_version"))
)
print(
    "service_api_websocket_evidence_reason_codes_csv="
    + str(payload.get("reason_codes_csv"))
)
print(
    "promotion_decision_reason_taxonomy_version="
    + str(payload.get("promotion_decision_reason_taxonomy_version"))
)
print(
    "promotion_decision_reason_codes_csv="
    + str(payload.get("promotion_decision_reason_codes_csv"))
)
print(
    "promotion_decision_reason_code="
    + str(payload.get("promotion_decision_reason_code"))
)
PY
