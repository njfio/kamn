#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/runtime/validate_sqlite_crash_recovery_live_contract_lane.sh"
EVIDENCE_CHECKER="$ROOT_DIR/scripts/runtime/check_sqlite_crash_recovery_live_evidence_convergence.sh"

EVIDENCE_REPORT_SCHEMA="kamn.runtime.sqlite-crash-recovery-live-evidence-convergence-report.v1"
EVIDENCE_REASON_TAXONOMY_VERSION="kamn.runtime.sqlite-crash-replay-evidence-convergence-reason-taxonomy.v1"
EVIDENCE_REASON_CODES_CSV="sqlite_crash_replay_evidence_link_missing,sqlite_crash_replay_evidence_payload_tamper_detected,sqlite_crash_replay_promotion_decision_reason_mapping_mismatch"
PROMOTION_DECISION_REASON_TAXONOMY_VERSION="kamn.runtime.sqlite-crash-recovery-promotion-decision-reason-taxonomy.v1"
PROMOTION_DECISION_REASON_CODES_CSV="sqlite_crash_recovery_policy_required_field_missing,sqlite_crash_recovery_policy_marker_missing,sqlite_crash_recovery_policy_reason_taxonomy_mismatch,sqlite_crash_recovery_policy_runtime_mode_contract_mismatch,replay_idempotency_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch,ci_fast_gate_failed,sqlite_crash_recovery_policy_expected_decision_mismatch,sqlite_crash_recovery_policy_violation"

for required_exec in "$CONTRACT_LANE" "$EVIDENCE_CHECKER"; do
  if [ ! -x "$required_exec" ]; then
    echo "expected sqlite crash-recovery convergence script to be executable: $required_exec" >&2
    exit 1
  fi
done

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT
lane_report="$TMP_DIR/sqlite-crash-recovery-contract-lane-report.json"
policy_report="$TMP_DIR/sqlite-crash-recovery-policy-report.json"
summary_report="$TMP_DIR/sqlite-crash-recovery-summary-report.json"
convergence_report="$TMP_DIR/sqlite-crash-recovery-convergence-report.json"

lane_output="$(
  bash "$CONTRACT_LANE" \
    --mode dry-run \
    --ci-fast-gate PASS \
    --max-seconds 240 \
    --output-json "$lane_report" \
    --policy-output-json "$policy_report" \
    --summary-output-json "$summary_report"
)"
if ! printf '%s\n' "$lane_output" | grep -q '^status=pass$'; then
  echo "expected sqlite crash-recovery contract lane status=pass marker" >&2
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
    echo "expected sqlite crash-recovery convergence marker ${marker}" >&2
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
    raise SystemExit("unexpected sqlite crash-recovery convergence report schema")
if payload.get("reason_taxonomy_version") != expected_reason_taxonomy_version:
    raise SystemExit("unexpected sqlite crash-recovery convergence reason taxonomy marker")
if payload.get("reason_codes_csv") != expected_reason_codes_csv:
    raise SystemExit("unexpected sqlite crash-recovery convergence reason codes marker")
if (
    payload.get("promotion_decision_reason_taxonomy_version")
    != expected_promotion_reason_taxonomy_version
):
    raise SystemExit("unexpected sqlite crash-recovery promotion reason taxonomy marker")
if (
    payload.get("promotion_decision_reason_codes_csv")
    != expected_promotion_reason_codes_csv
):
    raise SystemExit("unexpected sqlite crash-recovery promotion reason codes marker")
if payload.get("promotion_decision_reason_code") != "none":
    raise SystemExit("expected deterministic promotion_decision_reason_code=none marker")
PY

tampered_policy_report="$TMP_DIR/sqlite-crash-recovery-policy-report.tampered-mapping.json"
cp "$policy_report" "$tampered_policy_report"
python3 - "$tampered_policy_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["promotion_decision_reason_code"] = "tampered"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_mapping_output="$(
  bash "$EVIDENCE_CHECKER" \
    --report-file "$lane_report" \
    --policy-file "$tampered_policy_report" \
    --output-json "$TMP_DIR/sqlite-crash-recovery-convergence-report.tampered-mapping.json" 2>&1
)"
tampered_mapping_code=$?
set -e
if [ "$tampered_mapping_code" -eq 0 ]; then
  echo "expected tampered sqlite crash-recovery promotion mapping to fail evidence convergence checker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_mapping_output" | grep -q 'sqlite_crash_replay_promotion_decision_reason_mapping_mismatch'; then
  echo "expected deterministic promotion decision reason mapping mismatch marker" >&2
  exit 1
fi

missing_link_policy_report="$TMP_DIR/sqlite-crash-recovery-policy-report.missing-link.json"
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
    --output-json "$TMP_DIR/sqlite-crash-recovery-convergence-report.missing-link.json" 2>&1
)"
missing_link_code=$?
set -e
if [ "$missing_link_code" -eq 0 ]; then
  echo "expected missing source report link to fail sqlite crash-recovery evidence convergence checker" >&2
  exit 1
fi
if ! printf '%s\n' "$missing_link_output" | grep -q 'sqlite_crash_replay_evidence_link_missing:source_report_file'; then
  echo "expected deterministic missing evidence link marker for source_report_file" >&2
  exit 1
fi

echo "sqlite crash-recovery evidence convergence checker tests passed."
