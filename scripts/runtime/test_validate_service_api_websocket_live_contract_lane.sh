#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/runtime/validate_service_api_websocket_live_contract_lane.sh"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_service_api_websocket_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_service_api_websocket_live_policy.sh"
EVIDENCE_CHECKER="$ROOT_DIR/scripts/runtime/check_service_api_websocket_live_evidence_convergence.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected websocket live contract lane script to be executable" >&2
  exit 1
fi
if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected websocket live validation script to be executable" >&2
  exit 1
fi
if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected websocket live policy checker script to be executable" >&2
  exit 1
fi
if [ ! -x "$EVIDENCE_CHECKER" ]; then
  echo "expected websocket live evidence convergence checker script to be executable" >&2
  exit 1
fi

lane_report="$TMP_DIR/service-api-websocket-live-contract-lane-report.json"
policy_report="$TMP_DIR/service-api-websocket-live-policy-report.json"

lane_output="$(
  bash "$CONTRACT_LANE" \
    --output-json "$lane_report" \
    --policy-output-json "$policy_report"
)"
if ! printf '%s\n' "$lane_output" | grep -q '^status=pass$'; then
  echo "expected websocket live contract lane status=pass marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^final_decision=GO$'; then
  echo "expected websocket live contract lane final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^service_api_websocket_contract_status=verified$'; then
  echo "expected websocket live contract lane status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^service_api_websocket_policy_status=verified$'; then
  echo "expected websocket live contract lane policy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^websocket_session_lifecycle_status=verified$'; then
  echo "expected websocket session lifecycle status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^websocket_heartbeat_timeout_status=verified$'; then
  echo "expected websocket heartbeat-timeout status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^websocket_idle_timeout_contract_status=verified$'; then
  echo "expected websocket idle-timeout contract status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^websocket_reason_registry_status=verified$'; then
  echo "expected websocket reason registry status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^protocol_session_docs_contract_status=verified$'; then
  echo "expected protocol/session docs contract status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^service_api_protocol_session_reason_taxonomy_version=kamn.runtime.service-api.protocol-session-reason-taxonomy.v1$'; then
  echo "expected protocol/session reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^service_api_protocol_session_reason_codes_csv=service_api_ws_upgrade_header_missing,service_api_ws_connection_header_missing,service_api_ws_key_header_missing,service_api_ws_version_header_missing,service_api_ws_upgrade_header_invalid,service_api_ws_connection_header_invalid,service_api_ws_key_header_empty,service_api_ws_version_header_invalid,service_api_payload_json_syntax_invalid,service_api_payload_structure_invalid,service_api_payload_io_error,service_api_auth_replay_nonce_detected,service_api_websocket_upgrade_required,service_api_protocol_session_docs_marker_missing$'; then
  echo "expected protocol/session reason taxonomy csv marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^websocket_lifecycle_reason_taxonomy_version=kamn.runtime.service-api-websocket-lifecycle-reason-taxonomy.v1$'; then
  echo "expected websocket lifecycle reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^websocket_lifecycle_reason_codes_csv=service_api_ws_upgrade_header_missing,service_api_ws_version_header_invalid,service_api_auth_sender_did_header_missing,service_api_ws_connection_header_missing,service_api_ws_key_header_missing$'; then
  echo "expected websocket lifecycle reason taxonomy csv marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^fail_closed_reason_code=service_api_websocket_policy_marker_missing:websocket_session_lifecycle_status$'; then
  echo "expected websocket live contract lane fail-closed reason marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^service_api_websocket_evidence_convergence_status=verified$'; then
  echo "expected websocket live contract lane evidence convergence status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^promotion_decision_reason_mapping_status=verified$'; then
  echo "expected websocket live contract lane promotion decision reason mapping status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^service_api_websocket_evidence_reason_taxonomy_version=kamn.runtime.service-api-websocket-evidence-convergence-reason-taxonomy.v1$'; then
  echo "expected websocket live contract lane evidence reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^service_api_websocket_evidence_reason_codes_csv=service_api_websocket_evidence_link_missing,service_api_websocket_evidence_payload_tamper_detected,service_api_websocket_promotion_decision_reason_mapping_mismatch$'; then
  echo "expected websocket live contract lane evidence reason codes marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^promotion_decision_reason_taxonomy_version=kamn.runtime.service-api-websocket-promotion-decision-reason-taxonomy.v1$'; then
  echo "expected websocket live contract lane promotion decision reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^promotion_decision_reason_codes_csv=service_api_websocket_policy_required_field_missing,service_api_websocket_policy_marker_missing,service_api_websocket_policy_reason_taxonomy_mismatch,service_api_websocket_policy_idle_timeout_contract_mismatch,ci_fast_gate_failed,service_api_websocket_policy_expected_decision_mismatch,service_api_websocket_policy_violation$'; then
  echo "expected websocket live contract lane promotion decision reason codes marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^promotion_decision_reason_code=none$'; then
  echo "expected websocket live contract lane promotion decision reason code marker" >&2
  exit 1
fi

python3 - "$lane_report" "$policy_report" <<'PY'
import json
import pathlib
import sys

lane_payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if lane_payload.get("schema_version") != "kamn.runtime.service-api-websocket-live-contract-lane-report.v1":
    raise SystemExit("unexpected websocket live contract lane report schema")
if lane_payload.get("status") != "pass":
    raise SystemExit("expected websocket contract lane status=pass")
if lane_payload.get("final_decision") != "GO":
    raise SystemExit("expected websocket contract lane final_decision=GO")
if lane_payload.get("service_api_websocket_contract_status") != "verified":
    raise SystemExit("expected service_api_websocket_contract_status=verified")
if lane_payload.get("service_api_websocket_policy_status") != "verified":
    raise SystemExit("expected service_api_websocket_policy_status=verified")
if lane_payload.get("websocket_session_lifecycle_status") != "verified":
    raise SystemExit("expected websocket_session_lifecycle_status=verified")
if lane_payload.get("websocket_heartbeat_timeout_status") != "verified":
    raise SystemExit("expected websocket_heartbeat_timeout_status=verified")
if lane_payload.get("websocket_idle_timeout_contract_status") != "verified":
    raise SystemExit("expected websocket_idle_timeout_contract_status=verified")
if lane_payload.get("websocket_reason_registry_status") != "verified":
    raise SystemExit("expected websocket_reason_registry_status=verified")
if lane_payload.get("protocol_session_docs_contract_status") != "verified":
    raise SystemExit("expected protocol_session_docs_contract_status=verified")
if lane_payload.get("service_api_protocol_session_reason_taxonomy_version") != "kamn.runtime.service-api.protocol-session-reason-taxonomy.v1":
    raise SystemExit("expected deterministic service_api_protocol_session_reason_taxonomy_version marker")
if lane_payload.get("service_api_protocol_session_reason_codes_csv") != "service_api_ws_upgrade_header_missing,service_api_ws_connection_header_missing,service_api_ws_key_header_missing,service_api_ws_version_header_missing,service_api_ws_upgrade_header_invalid,service_api_ws_connection_header_invalid,service_api_ws_key_header_empty,service_api_ws_version_header_invalid,service_api_payload_json_syntax_invalid,service_api_payload_structure_invalid,service_api_payload_io_error,service_api_auth_replay_nonce_detected,service_api_websocket_upgrade_required,service_api_protocol_session_docs_marker_missing":
    raise SystemExit("expected deterministic service_api_protocol_session_reason_codes_csv marker")
if lane_payload.get("websocket_lifecycle_reason_taxonomy_version") != "kamn.runtime.service-api-websocket-lifecycle-reason-taxonomy.v1":
    raise SystemExit("expected deterministic websocket_lifecycle_reason_taxonomy_version marker")
if lane_payload.get("websocket_lifecycle_reason_codes_csv") != "service_api_ws_upgrade_header_missing,service_api_ws_version_header_invalid,service_api_auth_sender_did_header_missing,service_api_ws_connection_header_missing,service_api_ws_key_header_missing":
    raise SystemExit("expected deterministic websocket_lifecycle_reason_codes_csv marker")
if lane_payload.get("docs_contract_status") != "verified":
    raise SystemExit("expected docs_contract_status=verified")
if lane_payload.get("performance_budget_status") != "verified":
    raise SystemExit("expected performance_budget_status=verified")
if lane_payload.get("service_api_websocket_evidence_convergence_status") != "verified":
    raise SystemExit("expected service_api_websocket_evidence_convergence_status=verified")
if lane_payload.get("promotion_decision_reason_mapping_status") != "verified":
    raise SystemExit("expected promotion_decision_reason_mapping_status=verified in lane report")
if lane_payload.get("service_api_websocket_evidence_reason_taxonomy_version") != "kamn.runtime.service-api-websocket-evidence-convergence-reason-taxonomy.v1":
    raise SystemExit("expected deterministic service_api_websocket_evidence_reason_taxonomy_version marker")
if lane_payload.get("service_api_websocket_evidence_reason_codes_csv") != "service_api_websocket_evidence_link_missing,service_api_websocket_evidence_payload_tamper_detected,service_api_websocket_promotion_decision_reason_mapping_mismatch":
    raise SystemExit("expected deterministic service_api_websocket_evidence_reason_codes_csv marker")
if lane_payload.get("promotion_decision_reason_taxonomy_version") != "kamn.runtime.service-api-websocket-promotion-decision-reason-taxonomy.v1":
    raise SystemExit("expected deterministic promotion_decision_reason_taxonomy_version marker")
if lane_payload.get("promotion_decision_reason_codes_csv") != "service_api_websocket_policy_required_field_missing,service_api_websocket_policy_marker_missing,service_api_websocket_policy_reason_taxonomy_mismatch,service_api_websocket_policy_idle_timeout_contract_mismatch,ci_fast_gate_failed,service_api_websocket_policy_expected_decision_mismatch,service_api_websocket_policy_violation":
    raise SystemExit("expected deterministic promotion_decision_reason_codes_csv marker")
if lane_payload.get("promotion_decision_reason_code") != "none":
    raise SystemExit("expected deterministic promotion_decision_reason_code marker")

policy_payload = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
if policy_payload.get("schema_version") != "kamn.runtime.service-api-websocket-live-policy-report.v1":
    raise SystemExit("unexpected websocket live policy report schema")
if policy_payload.get("final_decision") != "GO":
    raise SystemExit("expected websocket policy final_decision=GO")
if policy_payload.get("service_api_websocket_policy_status") != "verified":
    raise SystemExit("expected service_api_websocket_policy_status=verified in policy report")
if policy_payload.get("reason_codes_value") != "none":
    raise SystemExit("expected policy reason_codes_value=none marker in policy report")
if policy_payload.get("websocket_lifecycle_reason_taxonomy_version") != "kamn.runtime.service-api-websocket-lifecycle-reason-taxonomy.v1":
    raise SystemExit("expected deterministic websocket_lifecycle_reason_taxonomy_version marker in policy report")
if policy_payload.get("websocket_lifecycle_reason_codes_csv") != "service_api_ws_upgrade_header_missing,service_api_ws_version_header_invalid,service_api_auth_sender_did_header_missing,service_api_ws_connection_header_missing,service_api_ws_key_header_missing":
    raise SystemExit("expected deterministic websocket_lifecycle_reason_codes_csv marker in policy report")
if policy_payload.get("promotion_decision_reason_mapping_status") != "verified":
    raise SystemExit("expected promotion_decision_reason_mapping_status=verified in policy report")
if policy_payload.get("promotion_decision_reason_taxonomy_version") != "kamn.runtime.service-api-websocket-promotion-decision-reason-taxonomy.v1":
    raise SystemExit("expected deterministic promotion_decision_reason_taxonomy_version marker in policy report")
if policy_payload.get("promotion_decision_reason_codes_csv") != "service_api_websocket_policy_required_field_missing,service_api_websocket_policy_marker_missing,service_api_websocket_policy_reason_taxonomy_mismatch,service_api_websocket_policy_idle_timeout_contract_mismatch,ci_fast_gate_failed,service_api_websocket_policy_expected_decision_mismatch,service_api_websocket_policy_violation":
    raise SystemExit("expected deterministic promotion_decision_reason_codes_csv marker in policy report")
if policy_payload.get("promotion_decision_reason_code") != "none":
    raise SystemExit("expected deterministic promotion_decision_reason_code marker in policy report")
PY

convergence_report="$TMP_DIR/service-api-websocket-live-convergence-report.json"
convergence_output="$(
  bash "$EVIDENCE_CHECKER" \
    --report-file "$lane_report" \
    --policy-file "$policy_report" \
    --output-json "$convergence_report"
)"
if ! printf '%s\n' "$convergence_output" | grep -q '^status=ok$'; then
  echo "expected websocket evidence convergence checker status=ok marker" >&2
  exit 1
fi
if ! printf '%s\n' "$convergence_output" | grep -q '^final_decision=GO$'; then
  echo "expected websocket evidence convergence checker final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$convergence_output" | grep -q '^evidence_convergence_status=verified$'; then
  echo "expected websocket evidence convergence checker evidence status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$convergence_output" | grep -q '^promotion_decision_reason_mapping_status=verified$'; then
  echo "expected websocket evidence convergence checker promotion mapping status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$convergence_output" | grep -q '^reason_taxonomy_version=kamn.runtime.service-api-websocket-evidence-convergence-reason-taxonomy.v1$'; then
  echo "expected websocket evidence convergence checker reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$convergence_output" | grep -q '^reason_codes_csv=service_api_websocket_evidence_link_missing,service_api_websocket_evidence_payload_tamper_detected,service_api_websocket_promotion_decision_reason_mapping_mismatch$'; then
  echo "expected websocket evidence convergence checker reason codes marker" >&2
  exit 1
fi
if ! printf '%s\n' "$convergence_output" | grep -q '^reason_codes_value=none$'; then
  echo "expected websocket evidence convergence checker reason_codes_value=none marker" >&2
  exit 1
fi
if ! printf '%s\n' "$convergence_output" | grep -q '^promotion_decision_reason_code=none$'; then
  echo "expected websocket evidence convergence checker promotion decision reason code marker" >&2
  exit 1
fi

python3 - "$convergence_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.service-api-websocket-live-convergence-report.v1":
    raise SystemExit("unexpected websocket evidence convergence report schema")
if payload.get("status") != "pass":
    raise SystemExit("expected websocket evidence convergence report status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected websocket evidence convergence report final_decision=GO")
if payload.get("evidence_convergence_status") != "verified":
    raise SystemExit("expected websocket evidence convergence status marker")
if payload.get("promotion_decision_reason_mapping_status") != "verified":
    raise SystemExit("expected websocket promotion decision reason mapping status marker")
if payload.get("reason_taxonomy_version") != "kamn.runtime.service-api-websocket-evidence-convergence-reason-taxonomy.v1":
    raise SystemExit("expected deterministic websocket evidence reason taxonomy marker")
if payload.get("reason_codes_csv") != "service_api_websocket_evidence_link_missing,service_api_websocket_evidence_payload_tamper_detected,service_api_websocket_promotion_decision_reason_mapping_mismatch":
    raise SystemExit("expected deterministic websocket evidence reason codes marker")
if payload.get("reason_codes") != ["none"]:
    raise SystemExit("expected websocket evidence convergence success reason code ['none']")
if payload.get("promotion_decision_reason_taxonomy_version") != "kamn.runtime.service-api-websocket-promotion-decision-reason-taxonomy.v1":
    raise SystemExit("expected deterministic websocket promotion decision reason taxonomy marker")
if payload.get("promotion_decision_reason_codes_csv") != "service_api_websocket_policy_required_field_missing,service_api_websocket_policy_marker_missing,service_api_websocket_policy_reason_taxonomy_mismatch,service_api_websocket_policy_idle_timeout_contract_mismatch,ci_fast_gate_failed,service_api_websocket_policy_expected_decision_mismatch,service_api_websocket_policy_violation":
    raise SystemExit("expected deterministic websocket promotion decision reason codes marker")
if payload.get("promotion_decision_reason_code") != "none":
    raise SystemExit("expected deterministic websocket promotion decision reason code marker")
PY

missing_link_policy="$TMP_DIR/service-api-websocket-live-policy.missing-link.json"
cp "$policy_report" "$missing_link_policy"
python3 - "$missing_link_policy" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload.pop("source_report_file", None)
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
missing_link_output_first="$(
  bash "$EVIDENCE_CHECKER" \
    --report-file "$lane_report" \
    --policy-file "$missing_link_policy" \
    --output-json "$TMP_DIR/service-api-websocket-live-convergence.missing-link.first.json" 2>&1
)"
missing_link_code_first=$?
missing_link_output_second="$(
  bash "$EVIDENCE_CHECKER" \
    --report-file "$lane_report" \
    --policy-file "$missing_link_policy" \
    --output-json "$TMP_DIR/service-api-websocket-live-convergence.missing-link.second.json" 2>&1
)"
missing_link_code_second=$?
set -e
if [ "$missing_link_code_first" -eq 0 ] || [ "$missing_link_code_second" -eq 0 ]; then
  echo "expected websocket evidence convergence checker to reject missing source report link deterministically" >&2
  exit 1
fi
if ! printf '%s\n' "$missing_link_output_first" | grep -q 'service_api_websocket_evidence_link_missing:source_report_file'; then
  echo "expected deterministic websocket missing-link reason output on first run" >&2
  exit 1
fi
if ! printf '%s\n' "$missing_link_output_second" | grep -q 'service_api_websocket_evidence_link_missing:source_report_file'; then
  echo "expected deterministic websocket missing-link reason output on second run" >&2
  exit 1
fi

python3 - \
  "$TMP_DIR/service-api-websocket-live-convergence.missing-link.first.json" \
  "$TMP_DIR/service-api-websocket-live-convergence.missing-link.second.json" <<'PY'
import json
import pathlib
import sys

first_payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
second_payload = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
first_reasons = first_payload.get("reason_codes")
second_reasons = second_payload.get("reason_codes")
if not first_reasons:
    raise SystemExit("expected websocket missing-link convergence report to include non-empty reason codes")
if first_reasons != second_reasons:
    raise SystemExit("expected deterministic websocket convergence reason-code ordering across repeated missing-link checks")
if "service_api_websocket_evidence_link_missing:source_report_file" not in first_reasons:
    raise SystemExit("expected service_api_websocket_evidence_link_missing:source_report_file reason code in missing-link convergence reports")
PY

tampered_mapping_policy="$TMP_DIR/service-api-websocket-live-policy.tampered-mapping.json"
cp "$policy_report" "$tampered_mapping_policy"
python3 - "$tampered_mapping_policy" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["promotion_decision_reason_code"] = "service_api_websocket_policy_marker_missing"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_mapping_output="$(
  bash "$EVIDENCE_CHECKER" \
    --report-file "$lane_report" \
    --policy-file "$tampered_mapping_policy" \
    --output-json "$TMP_DIR/service-api-websocket-live-convergence.tampered-mapping.json" 2>&1
)"
tampered_mapping_code=$?
set -e
if [ "$tampered_mapping_code" -eq 0 ]; then
  echo "expected websocket evidence convergence checker to reject tampered promotion reason mapping" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_mapping_output" | grep -q 'service_api_websocket_promotion_decision_reason_mapping_mismatch'; then
  echo "expected deterministic websocket promotion reason mapping mismatch marker" >&2
  exit 1
fi

tampered_payload_policy="$TMP_DIR/service-api-websocket-live-policy.tampered-payload.json"
cp "$policy_report" "$tampered_payload_policy"
python3 - "$tampered_payload_policy" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["reason_codes"] = "none"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_payload_output="$(
  bash "$EVIDENCE_CHECKER" \
    --report-file "$lane_report" \
    --policy-file "$tampered_payload_policy" \
    --output-json "$TMP_DIR/service-api-websocket-live-convergence.tampered-payload.json" 2>&1
)"
tampered_payload_code=$?
set -e
if [ "$tampered_payload_code" -eq 0 ]; then
  echo "expected websocket evidence convergence checker to reject tampered payload shape" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_payload_output" | grep -q 'service_api_websocket_evidence_payload_tamper_detected:reason_codes'; then
  echo "expected deterministic websocket payload tamper marker" >&2
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
  echo "expected websocket live contract lane to reject invalid ci-fast-gate value" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_ci_fast_gate_output" | grep -q 'ci-fast-gate must be PASS or FAIL'; then
  echo "expected deterministic invalid ci-fast-gate marker for websocket live contract lane" >&2
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
  echo "expected websocket live contract lane to fail closed when ci-fast-gate=FAIL" >&2
  exit 1
fi
if ! printf '%s\n' "$blocked_fast_gate_output" | grep -q 'ci_fast_gate_failed'; then
  echo "expected deterministic ci_fast_gate_failed marker for websocket live contract lane" >&2
  exit 1
fi

echo "service api websocket live contract lane tests passed."
