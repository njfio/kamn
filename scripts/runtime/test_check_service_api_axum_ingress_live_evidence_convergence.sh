#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/runtime/validate_service_api_axum_ingress_live_contract_lane.sh"
EVIDENCE_CHECKER="$ROOT_DIR/scripts/runtime/check_service_api_axum_ingress_live_evidence_convergence.sh"

EVIDENCE_REPORT_SCHEMA="kamn.runtime.service-api-axum-ingress-live-convergence-report.v1"
EVIDENCE_REASON_TAXONOMY_VERSION="kamn.runtime.service-api-axum-evidence-convergence-reason-taxonomy.v1"
EVIDENCE_REASON_CODES_CSV="service_api_axum_evidence_link_missing,service_api_axum_evidence_payload_tamper_detected,service_api_axum_promotion_decision_reason_mapping_mismatch"
PROMOTION_DECISION_REASON_TAXONOMY_VERSION="kamn.runtime.service-api-axum-protocol-mismatch-reason-taxonomy.v1"
PROMOTION_DECISION_REASON_CODES_CSV="service_api_axum_policy_required_field_missing,service_api_axum_policy_marker_missing,service_api_axum_policy_protocol_taxonomy_mismatch,service_api_axum_policy_limit_contract_mismatch,ci_fast_gate_failed,service_api_axum_policy_expected_decision_mismatch,service_api_axum_policy_violation"

for required_exec in "$CONTRACT_LANE" "$EVIDENCE_CHECKER"; do
  if [ ! -x "$required_exec" ]; then
    echo "expected service api axum ingress convergence script to be executable: $required_exec" >&2
    exit 1
  fi
done

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT
lane_report="$TMP_DIR/service-api-axum-ingress-contract-lane-report.json"
policy_report="$TMP_DIR/service-api-axum-ingress-policy-report.json"
convergence_report="$TMP_DIR/service-api-axum-ingress-convergence-report.json"

lane_output="$(
  bash "$CONTRACT_LANE" \
    --output-json "$lane_report" \
    --policy-output-json "$policy_report"
)"
if ! printf '%s\n' "$lane_output" | grep -q '^status=pass$'; then
  echo "expected service api axum ingress contract lane status=pass marker" >&2
  exit 1
fi

convergence_output="$(
  bash "$EVIDENCE_CHECKER" \
    --report-file "$lane_report" \
    --policy-file "$policy_report" \
    --output-json "$convergence_report"
)"
for marker in \
  "status=ok" \
  "final_decision=GO" \
  "evidence_convergence_status=verified" \
  "promotion_decision_reason_mapping_status=verified" \
  "reason_taxonomy_version=${EVIDENCE_REASON_TAXONOMY_VERSION}" \
  "reason_codes_csv=${EVIDENCE_REASON_CODES_CSV}" \
  "reason_codes_value=none" \
  "promotion_decision_reason_taxonomy_version=${PROMOTION_DECISION_REASON_TAXONOMY_VERSION}" \
  "promotion_decision_reason_codes_csv=${PROMOTION_DECISION_REASON_CODES_CSV}" \
  "promotion_decision_reason_code=none"; do
  if ! printf '%s\n' "$convergence_output" | grep -q "^${marker}$"; then
    echo "expected service api axum ingress convergence marker ${marker}" >&2
    exit 1
  fi
done

python3 - "$convergence_report" "$EVIDENCE_REPORT_SCHEMA" "$EVIDENCE_REASON_TAXONOMY_VERSION" "$EVIDENCE_REASON_CODES_CSV" "$PROMOTION_DECISION_REASON_TAXONOMY_VERSION" "$PROMOTION_DECISION_REASON_CODES_CSV" <<'PY'
import json
import pathlib
import sys

convergence_report_file = pathlib.Path(sys.argv[1])
expected_schema = sys.argv[2]
expected_reason_taxonomy_version = sys.argv[3]
expected_reason_codes_csv = sys.argv[4]
expected_promotion_reason_taxonomy_version = sys.argv[5]
expected_promotion_reason_codes_csv = sys.argv[6]

payload = json.loads(convergence_report_file.read_text(encoding="utf-8"))
if payload.get("schema_version") != expected_schema:
    raise SystemExit("unexpected service api axum convergence report schema")
if payload.get("reason_taxonomy_version") != expected_reason_taxonomy_version:
    raise SystemExit("unexpected service api axum convergence reason taxonomy marker")
if payload.get("reason_codes_csv") != expected_reason_codes_csv:
    raise SystemExit("unexpected service api axum convergence reason codes marker")
if (
    payload.get("promotion_decision_reason_taxonomy_version")
    != expected_promotion_reason_taxonomy_version
):
    raise SystemExit("unexpected service api axum promotion reason taxonomy marker")
if (
    payload.get("promotion_decision_reason_codes_csv")
    != expected_promotion_reason_codes_csv
):
    raise SystemExit("unexpected service api axum promotion reason codes marker")
if payload.get("promotion_decision_reason_code") != "none":
    raise SystemExit("expected deterministic promotion_decision_reason_code=none marker")
PY

tampered_payload_policy_report="$TMP_DIR/service-api-axum-ingress-policy-report.tampered-payload.json"
cp "$policy_report" "$tampered_payload_policy_report"
python3 - "$tampered_payload_policy_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["reason_codes_value"] = "tampered"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_payload_output="$(
  bash "$EVIDENCE_CHECKER" \
    --report-file "$lane_report" \
    --policy-file "$tampered_payload_policy_report" \
    --output-json "$TMP_DIR/service-api-axum-ingress-convergence-report.tampered-payload.json" 2>&1
)"
tampered_payload_code=$?
set -e
if [ "$tampered_payload_code" -eq 0 ]; then
  echo "expected tampered service api axum payload evidence to fail convergence checker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_payload_output" | grep -q 'service_api_axum_evidence_payload_tamper_detected:reason_codes_value'; then
  echo "expected deterministic payload tamper reason marker for service api axum convergence checker" >&2
  exit 1
fi

tampered_mapping_policy_report="$TMP_DIR/service-api-axum-ingress-policy-report.tampered-mapping.json"
cp "$policy_report" "$tampered_mapping_policy_report"
python3 - "$tampered_mapping_policy_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["service_api_axum_protocol_mismatch_reason_code"] = "tampered"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_mapping_output="$(
  bash "$EVIDENCE_CHECKER" \
    --report-file "$lane_report" \
    --policy-file "$tampered_mapping_policy_report" \
    --output-json "$TMP_DIR/service-api-axum-ingress-convergence-report.tampered-mapping.json" 2>&1
)"
tampered_mapping_code=$?
set -e
if [ "$tampered_mapping_code" -eq 0 ]; then
  echo "expected tampered service api axum promotion mapping to fail evidence convergence checker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_mapping_output" | grep -q 'service_api_axum_promotion_decision_reason_mapping_mismatch'; then
  echo "expected deterministic promotion decision reason mapping mismatch marker" >&2
  exit 1
fi

missing_link_policy_report="$TMP_DIR/service-api-axum-ingress-policy-report.missing-link.json"
cp "$policy_report" "$missing_link_policy_report"
python3 - "$missing_link_policy_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["source_report_file"] = ""
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
missing_link_output="$(
  bash "$EVIDENCE_CHECKER" \
    --report-file "$lane_report" \
    --policy-file "$missing_link_policy_report" \
    --output-json "$TMP_DIR/service-api-axum-ingress-convergence-report.missing-link.json" 2>&1
)"
missing_link_code=$?
set -e
if [ "$missing_link_code" -eq 0 ]; then
  echo "expected missing source report link to fail service api axum evidence convergence checker" >&2
  exit 1
fi
if ! printf '%s\n' "$missing_link_output" | grep -q 'service_api_axum_evidence_link_missing:source_report_file'; then
  echo "expected deterministic missing evidence link marker for source_report_file" >&2
  exit 1
fi

echo "service api axum ingress evidence convergence checker tests passed."
