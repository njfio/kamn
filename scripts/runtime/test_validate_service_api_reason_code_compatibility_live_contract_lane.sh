#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/runtime/validate_service_api_reason_code_compatibility_live_contract_lane.sh"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_service_api_reason_code_compatibility_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_service_api_reason_code_compatibility_live_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected service api reason-code compatibility contract lane script to be executable" >&2
  exit 1
fi
if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected service api reason-code compatibility validation script to be executable" >&2
  exit 1
fi
if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected service api reason-code compatibility policy checker script to be executable" >&2
  exit 1
fi

lane_report="$TMP_DIR/service-api-reason-code-compatibility-contract-lane-report.json"
policy_report="$TMP_DIR/service-api-reason-code-compatibility-policy-report.json"

lane_output="$(
  bash "$CONTRACT_LANE" \
    --output-json "$lane_report" \
    --policy-output-json "$policy_report"
)"
if ! printf '%s\n' "$lane_output" | grep -q '^status=pass$'; then
  echo "expected service api reason-code compatibility contract lane status=pass marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^final_decision=GO$'; then
  echo "expected service api reason-code compatibility contract lane final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^service_api_reason_code_contract_status=verified$'; then
  echo "expected service api reason-code compatibility contract lane status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^service_api_reason_code_policy_status=verified$'; then
  echo "expected service api reason-code compatibility contract lane policy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^error_envelope_field_status=verified$'; then
  echo "expected service api reason-code compatibility contract lane envelope marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^rust_sdk_reason_code_status=verified$'; then
  echo "expected service api reason-code compatibility contract lane rust sdk marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^python_sdk_reason_code_status=verified$'; then
  echo "expected service api reason-code compatibility contract lane python sdk marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^regression_corpus_status=verified$'; then
  echo "expected service api reason-code compatibility contract lane regression corpus marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^regression_drift_diagnostics_status=verified$'; then
  echo "expected service api reason-code compatibility contract lane regression drift marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^api_error_reason_taxonomy_status=verified$'; then
  echo "expected service api reason-code compatibility contract lane api-error taxonomy status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^timeout_classification_status=verified$'; then
  echo "expected service api reason-code compatibility contract lane timeout-classification marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^endpoint_parity_gate_normalization_status=verified$'; then
  echo "expected service api reason-code compatibility contract lane endpoint parity normalization marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^api_error_reason_taxonomy_version=kamn.runtime.service-api-error-reason-taxonomy.v1$'; then
  echo "expected service api reason-code compatibility contract lane api-error reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^api_error_reason_codes_csv=service_api_auth_sender_did_header_missing,service_api_auth_replay_nonce_detected,service_api_ws_upgrade_header_missing,service_api_ws_version_header_invalid,service_api_payload_json_syntax_invalid,service_api_payload_structure_invalid,service_api_payload_io_error$'; then
  echo "expected service api reason-code compatibility contract lane api-error reason taxonomy csv marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^timeout_reason_taxonomy_version=kamn.runtime.service-api-timeout-reason-taxonomy.v1$'; then
  echo "expected service api reason-code compatibility contract lane timeout reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^timeout_reason_codes_csv=service_api_request_read_failed$'; then
  echo "expected service api reason-code compatibility contract lane timeout reason taxonomy csv marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -Eq '^regression_corpus_scenario_count=[1-9][0-9]*$'; then
  echo "expected service api reason-code compatibility contract lane regression corpus count marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^fail_closed_reason_code=service_api_reason_code_policy_marker_missing:route_error_mapping_status$'; then
  echo "expected service api reason-code compatibility contract lane fail-closed reason marker" >&2
  exit 1
fi

python3 - "$lane_report" "$policy_report" <<'PY'
import json
import pathlib
import sys

lane_payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if lane_payload.get("schema_version") != "kamn.runtime.service-api-reason-code-compatibility-live-contract-lane-report.v1":
    raise SystemExit("unexpected service api reason-code compatibility contract lane report schema")
if lane_payload.get("status") != "pass":
    raise SystemExit("expected contract lane status=pass")
if lane_payload.get("final_decision") != "GO":
    raise SystemExit("expected contract lane final_decision=GO")
if lane_payload.get("service_api_reason_code_contract_status") != "verified":
    raise SystemExit("expected service_api_reason_code_contract_status=verified")
if lane_payload.get("service_api_reason_code_policy_status") != "verified":
    raise SystemExit("expected service_api_reason_code_policy_status=verified")
if lane_payload.get("error_envelope_field_status") != "verified":
    raise SystemExit("expected error_envelope_field_status=verified")
if lane_payload.get("rust_sdk_reason_code_status") != "verified":
    raise SystemExit("expected rust_sdk_reason_code_status=verified")
if lane_payload.get("python_sdk_reason_code_status") != "verified":
    raise SystemExit("expected python_sdk_reason_code_status=verified")
if lane_payload.get("regression_corpus_status") != "verified":
    raise SystemExit("expected regression_corpus_status=verified")
if lane_payload.get("regression_drift_diagnostics_status") != "verified":
    raise SystemExit("expected regression_drift_diagnostics_status=verified")
if lane_payload.get("api_error_reason_taxonomy_status") != "verified":
    raise SystemExit("expected api_error_reason_taxonomy_status=verified")
if lane_payload.get("timeout_classification_status") != "verified":
    raise SystemExit("expected timeout_classification_status=verified")
if lane_payload.get("endpoint_parity_gate_normalization_status") != "verified":
    raise SystemExit("expected endpoint_parity_gate_normalization_status=verified")
if lane_payload.get("api_error_reason_taxonomy_version") != "kamn.runtime.service-api-error-reason-taxonomy.v1":
    raise SystemExit("expected deterministic api_error_reason_taxonomy_version marker")
if lane_payload.get("api_error_reason_codes_csv") != "service_api_auth_sender_did_header_missing,service_api_auth_replay_nonce_detected,service_api_ws_upgrade_header_missing,service_api_ws_version_header_invalid,service_api_payload_json_syntax_invalid,service_api_payload_structure_invalid,service_api_payload_io_error":
    raise SystemExit("expected deterministic api_error_reason_codes_csv marker")
if lane_payload.get("timeout_reason_taxonomy_version") != "kamn.runtime.service-api-timeout-reason-taxonomy.v1":
    raise SystemExit("expected deterministic timeout_reason_taxonomy_version marker")
if lane_payload.get("timeout_reason_codes_csv") != "service_api_request_read_failed":
    raise SystemExit("expected deterministic timeout_reason_codes_csv marker")
scenario_count = lane_payload.get("regression_corpus_scenario_count")
if not isinstance(scenario_count, int) or scenario_count <= 0:
    raise SystemExit("expected regression_corpus_scenario_count to be positive integer")
if lane_payload.get("docs_contract_status") != "verified":
    raise SystemExit("expected docs_contract_status=verified")
if lane_payload.get("performance_budget_status") != "verified":
    raise SystemExit("expected performance_budget_status=verified")

policy_payload = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
if policy_payload.get("schema_version") != "kamn.runtime.service-api-reason-code-compatibility-live-policy-report.v1":
    raise SystemExit("unexpected service api reason-code compatibility policy report schema")
if policy_payload.get("final_decision") != "GO":
    raise SystemExit("expected policy final_decision=GO")
if policy_payload.get("service_api_reason_code_policy_status") != "verified":
    raise SystemExit("expected service_api_reason_code_policy_status=verified in policy report")
if policy_payload.get("api_error_reason_taxonomy_version") != "kamn.runtime.service-api-error-reason-taxonomy.v1":
    raise SystemExit("expected deterministic api_error_reason_taxonomy_version marker in policy report")
if policy_payload.get("api_error_reason_codes_csv") != "service_api_auth_sender_did_header_missing,service_api_auth_replay_nonce_detected,service_api_ws_upgrade_header_missing,service_api_ws_version_header_invalid,service_api_payload_json_syntax_invalid,service_api_payload_structure_invalid,service_api_payload_io_error":
    raise SystemExit("expected deterministic api_error_reason_codes_csv marker in policy report")
if policy_payload.get("timeout_reason_taxonomy_version") != "kamn.runtime.service-api-timeout-reason-taxonomy.v1":
    raise SystemExit("expected deterministic timeout_reason_taxonomy_version marker in policy report")
if policy_payload.get("timeout_reason_codes_csv") != "service_api_request_read_failed":
    raise SystemExit("expected deterministic timeout_reason_codes_csv marker in policy report")
PY

if ! grep -q "check_service_api_reason_code_compatibility_live_policy.sh" "$CONTRACT_LANE"; then
  echo "expected service api reason-code compatibility contract lane to compose policy checker" >&2
  exit 1
fi
if ! grep -q "validate_service_api_reason_code_compatibility_live.sh" "$CONTRACT_LANE"; then
  echo "expected service api reason-code compatibility contract lane to compose validation lane" >&2
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
  echo "expected service api reason-code compatibility contract lane to reject invalid ci-fast-gate value" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_ci_fast_gate_output" | grep -q 'ci-fast-gate must be PASS or FAIL'; then
  echo "expected deterministic invalid ci-fast-gate marker for service api reason-code compatibility contract lane" >&2
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
  echo "expected service api reason-code compatibility contract lane to fail closed when ci-fast-gate=FAIL" >&2
  exit 1
fi
if ! printf '%s\n' "$blocked_fast_gate_output" | grep -q 'ci_fast_gate_failed'; then
  echo "expected deterministic ci_fast_gate_failed marker for service api reason-code compatibility contract lane" >&2
  exit 1
fi

echo "service api reason-code compatibility contract lane tests passed."
