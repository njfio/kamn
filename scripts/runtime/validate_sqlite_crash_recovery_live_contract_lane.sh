#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_sqlite_crash_recovery_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_sqlite_crash_recovery_live_policy.sh"
EVIDENCE_CHECKER="$ROOT_DIR/scripts/runtime/check_sqlite_crash_recovery_live_evidence_convergence.sh"
STRATEGY_DOC="$ROOT_DIR/docs/ci/strategy.md"
RUNBOOK_DOC="$ROOT_DIR/docs/deploy/kolme_devnet_ops.md"

EVIDENCE_REPORT_SCHEMA="kamn.runtime.sqlite-crash-recovery-live-evidence-convergence-report.v1"
EVIDENCE_REASON_TAXONOMY_VERSION="kamn.runtime.sqlite-crash-replay-evidence-convergence-reason-taxonomy.v1"
EVIDENCE_REASON_CODES_CSV="sqlite_crash_replay_evidence_link_missing,sqlite_crash_replay_evidence_payload_tamper_detected,sqlite_crash_replay_promotion_decision_reason_mapping_mismatch"
PROMOTION_DECISION_REASON_TAXONOMY_VERSION="kamn.runtime.sqlite-crash-recovery-promotion-decision-reason-taxonomy.v1"
PROMOTION_DECISION_REASON_CODES_CSV="sqlite_crash_recovery_policy_required_field_missing,sqlite_crash_recovery_policy_marker_missing,sqlite_crash_recovery_policy_reason_taxonomy_mismatch,sqlite_crash_recovery_policy_runtime_mode_contract_mismatch,replay_idempotency_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch,ci_fast_gate_failed,sqlite_crash_recovery_policy_expected_decision_mismatch,sqlite_crash_recovery_policy_violation"
EVIDENCE_TAMPER_REASON_CODE="sqlite_crash_replay_promotion_decision_reason_mapping_mismatch"

output_json=""
policy_output_json=""
convergence_output_json=""
summary_output_json=""
max_seconds="${KAMN_SQLITE_CRASH_RECOVERY_LIVE_CONTRACT_MAX_SECONDS:-240}"
ci_fast_gate="PASS"
mode="dry-run"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-json)
      output_json="${2:-}"
      shift 2
      ;;
    --policy-output-json)
      policy_output_json="${2:-}"
      shift 2
      ;;
    --convergence-output-json)
      convergence_output_json="${2:-}"
      shift 2
      ;;
    --summary-output-json)
      summary_output_json="${2:-}"
      shift 2
      ;;
    --max-seconds)
      max_seconds="${2:-}"
      shift 2
      ;;
    --ci-fast-gate)
      ci_fast_gate="${2:-}"
      shift 2
      ;;
    --mode)
      mode="${2:-}"
      shift 2
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if ! [[ "$max_seconds" =~ ^[0-9]+$ ]]; then
  echo "max-seconds must be an integer" >&2
  exit 1
fi
if [ "$max_seconds" -le 0 ]; then
  echo "max-seconds must be greater than zero" >&2
  exit 1
fi
if [[ "$ci_fast_gate" != "PASS" && "$ci_fast_gate" != "FAIL" ]]; then
  echo "ci-fast-gate must be PASS or FAIL" >&2
  exit 1
fi
if [[ "$mode" != "dry-run" && "$mode" != "run" ]]; then
  echo "mode must be dry-run or run" >&2
  exit 1
fi
for required_exec in "$VALIDATION_SCRIPT" "$POLICY_CHECKER" "$EVIDENCE_CHECKER"; do
  if [ ! -x "$required_exec" ]; then
    echo "expected required executable script '$required_exec'" >&2
    exit 1
  fi
done
if [ ! -f "$STRATEGY_DOC" ]; then
  echo "expected required documentation file '$STRATEGY_DOC'" >&2
  exit 1
fi
if [ ! -f "$RUNBOOK_DOC" ]; then
  echo "expected required runbook file '$RUNBOOK_DOC'" >&2
  exit 1
fi

start_epoch="$(date +%s)"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

summary_report="$TMP_DIR/sqlite-crash-recovery-live-summary.json"
policy_report="$TMP_DIR/sqlite-crash-recovery-live-policy.json"
tampered_report="$TMP_DIR/sqlite-crash-recovery-live-summary.tampered.json"
convergence_report="$TMP_DIR/sqlite-crash-recovery-live-convergence.json"

validation_output="$(
  bash "$VALIDATION_SCRIPT" \
    --mode "$mode" \
    --max-seconds "$max_seconds" \
    --ci-fast-gate "$ci_fast_gate" \
    --output-json "$summary_report"
)"
if ! grep -q '^status=pass$' <<<"$validation_output"; then
  echo "expected sqlite crash-recovery live validation status=pass marker" >&2
  exit 1
fi
if ! grep -q '^final_decision=GO$' <<<"$validation_output"; then
  echo "expected sqlite crash-recovery live validation final_decision=GO marker" >&2
  exit 1
fi
if ! grep -q '^fast_gate_exclusion_status=verified$' <<<"$validation_output"; then
  echo "expected sqlite crash-recovery live validation fast-gate exclusion marker" >&2
  exit 1
fi
if ! grep -q '^sqlite_crash_recovery_state_replay_status=verified$' <<<"$validation_output"; then
  echo "expected sqlite crash-recovery live validation replay marker" >&2
  exit 1
fi
if ! grep -q '^sqlite_crash_recovery_abrupt_kill_status=verified$' <<<"$validation_output"; then
  echo "expected sqlite crash-recovery live validation abrupt-kill marker" >&2
  exit 1
fi
if ! grep -q '^wal_append_status=verified$' <<<"$validation_output"; then
  echo "expected sqlite crash-recovery live validation wal-append marker" >&2
  exit 1
fi
if ! grep -q '^wal_checkpoint_status=verified$' <<<"$validation_output"; then
  echo "expected sqlite crash-recovery live validation wal-checkpoint marker" >&2
  exit 1
fi
if ! grep -q '^append_checkpoint_integrity_status=verified$' <<<"$validation_output"; then
  echo "expected sqlite crash-recovery live validation append-checkpoint integrity marker" >&2
  exit 1
fi
if ! grep -q '^append_checkpoint_reason_taxonomy_version=kamn.runtime.append-checkpoint-integrity-reason-taxonomy.v1$' <<<"$validation_output"; then
  echo "expected sqlite crash-recovery live validation append-checkpoint taxonomy marker" >&2
  exit 1
fi
if ! grep -q '^append_checkpoint_reason_codes_csv=wal_append_marker_missing,wal_checkpoint_marker_missing,append_checkpoint_marker_parity_mismatch$' <<<"$validation_output"; then
  echo "expected sqlite crash-recovery live validation append-checkpoint taxonomy csv marker" >&2
  exit 1
fi
if ! grep -q '^wal_durability_reason_taxonomy_version=kamn.runtime.wal-durability-reason-taxonomy.v1$' <<<"$validation_output"; then
  echo "expected sqlite crash-recovery live validation wal-durability taxonomy marker" >&2
  exit 1
fi
if ! grep -q '^wal_durability_reason_codes_csv=wal_append_rejected,wal_checkpoint_skipped,wal_replay_incomplete$' <<<"$validation_output"; then
  echo "expected sqlite crash-recovery live validation wal-durability taxonomy csv marker" >&2
  exit 1
fi
if ! grep -q '^historical_query_index_status=verified$' <<<"$validation_output"; then
  echo "expected sqlite crash-recovery live validation historical-query index marker" >&2
  exit 1
fi
if ! grep -q '^historical_query_latency_budget_status=verified$' <<<"$validation_output"; then
  echo "expected sqlite crash-recovery live validation historical-query latency budget marker" >&2
  exit 1
fi
if ! grep -q '^historical_query_reason_taxonomy_version=kamn.runtime.historical-query-reason-taxonomy.v1$' <<<"$validation_output"; then
  echo "expected sqlite crash-recovery live validation historical-query taxonomy marker" >&2
  exit 1
fi
if ! grep -q '^historical_query_reason_codes_csv=historical_query_index_drift,historical_query_latency_budget_exceeded,historical_query_consistency_mismatch$' <<<"$validation_output"; then
  echo "expected sqlite crash-recovery live validation historical-query taxonomy csv marker" >&2
  exit 1
fi
if ! grep -q '^journal_replay_drift_detection_status=verified$' <<<"$validation_output"; then
  echo "expected sqlite crash-recovery live validation journal replay drift detection marker" >&2
  exit 1
fi
if ! grep -q '^checkpoint_divergence_bypass_rejection_status=verified$' <<<"$validation_output"; then
  echo "expected sqlite crash-recovery live validation checkpoint divergence bypass rejection marker" >&2
  exit 1
fi
if ! grep -q '^journal_replay_reason_taxonomy_version=kamn.runtime.journal-replay-reason-taxonomy.v1$' <<<"$validation_output"; then
  echo "expected sqlite crash-recovery live validation journal replay taxonomy marker" >&2
  exit 1
fi
if ! grep -q '^journal_replay_reason_codes_csv=journal_replay_drift_detected,checkpoint_divergence_bypass_detected$' <<<"$validation_output"; then
  echo "expected sqlite crash-recovery live validation journal replay taxonomy csv marker" >&2
  exit 1
fi
if ! grep -q '^replay_idempotency_taxonomy_mapping_status=verified$' <<<"$validation_output"; then
  echo "expected sqlite crash-recovery live validation replay idempotency taxonomy mapping status marker" >&2
  exit 1
fi
if ! grep -q '^runbook_marker_parity_status=verified$' <<<"$validation_output"; then
  echo "expected sqlite crash-recovery live validation runbook marker parity status marker" >&2
  exit 1
fi
if ! grep -q '^replay_idempotency_runbook_reason_taxonomy_version=kamn.runtime.sqlite-crash-recovery-replay-idempotency-runbook-reason-taxonomy.v1$' <<<"$validation_output"; then
  echo "expected sqlite crash-recovery live validation replay idempotency runbook reason taxonomy marker" >&2
  exit 1
fi
if ! grep -q '^replay_idempotency_runbook_reason_codes_csv=replay_idempotency_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch$' <<<"$validation_output"; then
  echo "expected sqlite crash-recovery live validation replay idempotency runbook reason csv marker" >&2
  exit 1
fi
if ! grep -q '^replay_idempotency_runbook_reason_code=none$' <<<"$validation_output"; then
  echo "expected sqlite crash-recovery live validation replay idempotency runbook reason code marker" >&2
  exit 1
fi
if ! grep -q '^crash_recovery_readiness_progress_status=verified$' <<<"$validation_output"; then
  echo "expected sqlite crash-recovery live validation readiness-progress marker" >&2
  exit 1
fi
if ! grep -q '^snapshot_parity_status=verified$' <<<"$validation_output"; then
  echo "expected sqlite crash-recovery live validation snapshot-parity marker" >&2
  exit 1
fi
if ! grep -q '^ci_local_recovery_budget_boundary_status=verified$' <<<"$validation_output"; then
  echo "expected sqlite crash-recovery live validation ci-local recovery budget boundary marker" >&2
  exit 1
fi
if ! grep -q '^state_consistency_reason_taxonomy_version=kamn.runtime.crash-recovery-state-consistency-reason-taxonomy.v1$' <<<"$validation_output"; then
  echo "expected sqlite crash-recovery live validation state-consistency taxonomy marker" >&2
  exit 1
fi
if ! grep -q '^state_consistency_reason_codes_csv=crash_recovery_readiness_progress_stalled,snapshot_parity_drift_detected,ci_local_recovery_budget_boundary_exceeded$' <<<"$validation_output"; then
  echo "expected sqlite crash-recovery live validation state-consistency taxonomy csv marker" >&2
  exit 1
fi
if ! grep -q '^crash_recovery_promotion_gate_status=verified$' <<<"$validation_output"; then
  echo "expected sqlite crash-recovery live validation promotion-gate marker" >&2
  exit 1
fi
if ! grep -q '^audit_trail_parity_status=verified$' <<<"$validation_output"; then
  echo "expected sqlite crash-recovery live validation audit-trail parity marker" >&2
  exit 1
fi
if ! grep -q '^ci_local_promotion_budget_boundary_status=verified$' <<<"$validation_output"; then
  echo "expected sqlite crash-recovery live validation ci-local promotion budget boundary marker" >&2
  exit 1
fi
if ! grep -q '^durability_governance_reason_taxonomy_version=kamn.runtime.durability-governance-reason-taxonomy.v1$' <<<"$validation_output"; then
  echo "expected sqlite crash-recovery live validation durability-governance taxonomy marker" >&2
  exit 1
fi
if ! grep -q '^durability_governance_reason_codes_csv=crash_recovery_promotion_stalled,audit_trail_parity_mismatch,ci_local_promotion_budget_boundary_exceeded$' <<<"$validation_output"; then
  echo "expected sqlite crash-recovery live validation durability-governance taxonomy csv marker" >&2
  exit 1
fi

policy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$summary_report" \
    --expected-final-decision GO \
    --ci-fast-gate "$ci_fast_gate" \
    --runbook-file "$RUNBOOK_DOC" \
    --output-json "$policy_report"
)"
if ! grep -q '^status=ok$' <<<"$policy_output"; then
  echo "expected sqlite crash-recovery policy checker status=ok marker" >&2
  exit 1
fi
if ! grep -q '^final_decision=GO$' <<<"$policy_output"; then
  echo "expected sqlite crash-recovery policy checker final_decision=GO marker" >&2
  exit 1
fi
if ! grep -q '^sqlite_crash_recovery_policy_status=verified$' <<<"$policy_output"; then
  echo "expected sqlite crash-recovery policy checker status marker" >&2
  exit 1
fi
if ! grep -q '^promotion_decision_reason_mapping_status=verified$' <<<"$policy_output"; then
  echo "expected sqlite crash-recovery policy checker promotion decision reason mapping status marker" >&2
  exit 1
fi
if ! grep -q "^promotion_decision_reason_taxonomy_version=${PROMOTION_DECISION_REASON_TAXONOMY_VERSION}$" <<<"$policy_output"; then
  echo "expected sqlite crash-recovery policy checker promotion decision reason taxonomy version marker" >&2
  exit 1
fi
if ! grep -q "^promotion_decision_reason_codes_csv=${PROMOTION_DECISION_REASON_CODES_CSV}$" <<<"$policy_output"; then
  echo "expected sqlite crash-recovery policy checker promotion decision reason taxonomy csv marker" >&2
  exit 1
fi
if ! grep -q '^promotion_decision_reason_code=none$' <<<"$policy_output"; then
  echo "expected sqlite crash-recovery policy checker promotion decision reason code marker on GO path" >&2
  exit 1
fi

cp "$summary_report" "$tampered_report"
python3 - "$tampered_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["fast_gate_exclusion_status"] = "tampered"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_policy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_report" \
    --expected-final-decision GO \
    --ci-fast-gate "$ci_fast_gate" \
    --runbook-file "$RUNBOOK_DOC" \
    --output-json "$TMP_DIR/sqlite-crash-recovery-live-policy.tampered.json" 2>&1
)"
tampered_policy_code=$?
set -e
if [ "$tampered_policy_code" -eq 0 ]; then
  echo "expected tampered sqlite crash-recovery report to fail policy validation" >&2
  exit 1
fi
if ! grep -q 'sqlite_crash_recovery_policy_fast_gate_exclusion_mismatch' <<<"$tampered_policy_output"; then
  echo "expected deterministic fail-closed reason for tampered sqlite crash-recovery report" >&2
  exit 1
fi

for required_ref in \
  "validate_sqlite_crash_recovery_live.sh" \
  "check_sqlite_crash_recovery_live_policy.sh" \
  "check_sqlite_crash_recovery_live_evidence_convergence.sh" \
  "validate_sqlite_crash_recovery_live_contract_lane.sh" \
  "test_validate_sqlite_crash_recovery_live.sh" \
  "test_check_sqlite_crash_recovery_live_policy.sh" \
  "test_check_sqlite_crash_recovery_live_evidence_convergence.sh" \
  "test_validate_sqlite_crash_recovery_live_contract_lane.sh"; do
  if ! grep -q "$required_ref" "$STRATEGY_DOC"; then
    echo "expected CI strategy docs to reference $required_ref" >&2
    exit 1
  fi
done
if ! grep -q "sqlite crash-recovery run-mode commands remain excluded from ci-fast-gate and ci-tools fast mode." "$STRATEGY_DOC"; then
  echo "expected CI strategy docs to include sqlite crash-recovery run-mode exclusion marker" >&2
  exit 1
fi
if ! grep -q "sqlite crash-replay evidence convergence remains deterministic via:" "$STRATEGY_DOC"; then
  echo "expected CI strategy docs to include sqlite crash-replay evidence convergence heading" >&2
  exit 1
fi
if ! grep -q "sqlite_crash_replay_evidence_reason_taxonomy_version=${EVIDENCE_REASON_TAXONOMY_VERSION}" "$STRATEGY_DOC"; then
  echo "expected CI strategy docs to include sqlite crash-replay evidence convergence reason taxonomy marker" >&2
  exit 1
fi
if ! grep -q "promotion_decision_reason_taxonomy_version=${PROMOTION_DECISION_REASON_TAXONOMY_VERSION}" "$STRATEGY_DOC"; then
  echo "expected CI strategy docs to include sqlite crash-replay promotion decision reason taxonomy marker" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "sqlite crash-recovery contract lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

lane_report="$TMP_DIR/sqlite-crash-recovery-live-contract-lane-report.json"
python3 - "$summary_report" "$policy_report" "$lane_report" "$elapsed_seconds" "$max_seconds" "$mode" <<'PY'
import json
import pathlib
import sys

summary_report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
policy_report = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
lane_report_file = pathlib.Path(sys.argv[3])
elapsed_seconds = int(sys.argv[4])
max_seconds = int(sys.argv[5])
mode = sys.argv[6]

if summary_report.get("schema_version") != "kamn.runtime.sqlite-crash-recovery-live-report.v1":
    raise SystemExit("unexpected sqlite crash-recovery summary schema")
if policy_report.get("schema_version") != "kamn.runtime.sqlite-crash-recovery-live-policy-report.v1":
    raise SystemExit("unexpected sqlite crash-recovery policy schema")
if summary_report.get("final_decision") != "GO":
    raise SystemExit("expected sqlite crash-recovery summary final_decision=GO")
if policy_report.get("final_decision") != "GO":
    raise SystemExit("expected sqlite crash-recovery policy final_decision=GO")

lane_report = {
    "schema_version": "kamn.runtime.sqlite-crash-recovery-live-contract-lane-report.v1",
    "status": "pass",
    "final_decision": "GO",
    "lane_mode": mode,
    "wal_append_status": summary_report.get("wal_append_status"),
    "wal_checkpoint_status": summary_report.get("wal_checkpoint_status"),
    "append_checkpoint_integrity_status": summary_report.get(
        "append_checkpoint_integrity_status"
    ),
    "append_checkpoint_reason_taxonomy_version": summary_report.get(
        "append_checkpoint_reason_taxonomy_version"
    ),
    "append_checkpoint_reason_codes_csv": summary_report.get(
        "append_checkpoint_reason_codes_csv"
    ),
    "wal_durability_reason_taxonomy_version": summary_report.get(
        "wal_durability_reason_taxonomy_version"
    ),
    "wal_durability_reason_codes_csv": summary_report.get(
        "wal_durability_reason_codes_csv"
    ),
    "historical_query_index_status": summary_report.get(
        "historical_query_index_status"
    ),
    "historical_query_latency_budget_status": summary_report.get(
        "historical_query_latency_budget_status"
    ),
    "historical_query_reason_taxonomy_version": summary_report.get(
        "historical_query_reason_taxonomy_version"
    ),
    "historical_query_reason_codes_csv": summary_report.get(
        "historical_query_reason_codes_csv"
    ),
    "journal_replay_drift_detection_status": summary_report.get(
        "journal_replay_drift_detection_status"
    ),
    "checkpoint_divergence_bypass_rejection_status": summary_report.get(
        "checkpoint_divergence_bypass_rejection_status"
    ),
    "journal_replay_reason_taxonomy_version": summary_report.get(
        "journal_replay_reason_taxonomy_version"
    ),
    "journal_replay_reason_codes_csv": summary_report.get(
        "journal_replay_reason_codes_csv"
    ),
    "replay_idempotency_taxonomy_mapping_status": summary_report.get(
        "replay_idempotency_taxonomy_mapping_status"
    ),
    "runbook_marker_parity_status": summary_report.get(
        "runbook_marker_parity_status"
    ),
    "replay_idempotency_runbook_reason_taxonomy_version": summary_report.get(
        "replay_idempotency_runbook_reason_taxonomy_version"
    ),
    "replay_idempotency_runbook_reason_codes_csv": summary_report.get(
        "replay_idempotency_runbook_reason_codes_csv"
    ),
    "replay_idempotency_runbook_reason_code": policy_report.get(
        "replay_idempotency_runbook_reason_code"
    ),
    "promotion_decision_reason_mapping_status": policy_report.get(
        "promotion_decision_reason_mapping_status"
    ),
    "promotion_decision_reason_taxonomy_version": policy_report.get(
        "promotion_decision_reason_taxonomy_version"
    ),
    "promotion_decision_reason_codes_csv": policy_report.get(
        "promotion_decision_reason_codes_csv"
    ),
    "promotion_decision_reason_code": policy_report.get(
        "promotion_decision_reason_code"
    ),
    "crash_recovery_readiness_progress_status": summary_report.get(
        "crash_recovery_readiness_progress_status"
    ),
    "snapshot_parity_status": summary_report.get("snapshot_parity_status"),
    "ci_local_recovery_budget_boundary_status": summary_report.get(
        "ci_local_recovery_budget_boundary_status"
    ),
    "state_consistency_reason_taxonomy_version": summary_report.get(
        "state_consistency_reason_taxonomy_version"
    ),
    "state_consistency_reason_codes_csv": summary_report.get(
        "state_consistency_reason_codes_csv"
    ),
    "crash_recovery_promotion_gate_status": summary_report.get(
        "crash_recovery_promotion_gate_status"
    ),
    "audit_trail_parity_status": summary_report.get("audit_trail_parity_status"),
    "ci_local_promotion_budget_boundary_status": summary_report.get(
        "ci_local_promotion_budget_boundary_status"
    ),
    "durability_governance_reason_taxonomy_version": summary_report.get(
        "durability_governance_reason_taxonomy_version"
    ),
    "durability_governance_reason_codes_csv": summary_report.get(
        "durability_governance_reason_codes_csv"
    ),
    "sqlite_crash_recovery_contract_status": "verified",
    "sqlite_crash_recovery_policy_status": policy_report.get("sqlite_crash_recovery_policy_status"),
    "docs_contract_status": "verified",
    "fail_closed_status": "verified",
    "fail_closed_reason_code": "sqlite_crash_recovery_policy_fast_gate_exclusion_mismatch",
    "performance_budget_status": "verified",
    "elapsed_seconds": elapsed_seconds,
    "max_seconds": max_seconds,
}
lane_report_file.write_text(json.dumps(lane_report, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

source_report_file="$summary_report"
if [[ -n "$summary_output_json" ]]; then
  mkdir -p "$(dirname "$summary_output_json")"
  cp "$summary_report" "$summary_output_json"
  source_report_file="$summary_output_json"
fi
python3 - "$policy_report" "$source_report_file" <<'PY'
import json
import pathlib
import sys

policy_report_file = pathlib.Path(sys.argv[1])
source_report_file = sys.argv[2]
payload = json.loads(policy_report_file.read_text(encoding="utf-8"))
payload["source_report_file"] = source_report_file
policy_report_file.write_text(
    json.dumps(payload, sort_keys=True, indent=2) + "\n",
    encoding="utf-8",
)
PY

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
  if ! grep -q "^${marker}$" <<<"$convergence_output"; then
    echo "expected sqlite crash-recovery convergence marker ${marker}" >&2
    exit 1
  fi
done

tampered_policy_report="$TMP_DIR/sqlite-crash-recovery-live-policy.tampered-mapping.json"
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
tampered_convergence_output="$(
  bash "$EVIDENCE_CHECKER" \
    --report-file "$lane_report" \
    --policy-file "$tampered_policy_report" \
    --output-json "$TMP_DIR/sqlite-crash-recovery-live-convergence.tampered-mapping.json" 2>&1
)"
tampered_convergence_code=$?
set -e
if [[ "$tampered_convergence_code" -eq 0 ]]; then
  echo "expected tampered sqlite crash-recovery promotion mapping to fail evidence convergence checker" >&2
  exit 1
fi
if ! grep -q "$EVIDENCE_TAMPER_REASON_CODE" <<<"$tampered_convergence_output"; then
  echo "expected deterministic fail-closed reason for tampered sqlite crash-recovery promotion mapping" >&2
  exit 1
fi

python3 - "$lane_report" "$convergence_report" "$EVIDENCE_REPORT_SCHEMA" "$EVIDENCE_REASON_TAXONOMY_VERSION" "$EVIDENCE_REASON_CODES_CSV" "$PROMOTION_DECISION_REASON_TAXONOMY_VERSION" "$PROMOTION_DECISION_REASON_CODES_CSV" <<'PY'
import json
import pathlib
import sys

lane_report_file = pathlib.Path(sys.argv[1])
convergence_report_file = pathlib.Path(sys.argv[2])
expected_schema = sys.argv[3]
expected_reason_taxonomy_version = sys.argv[4]
expected_reason_codes_csv = sys.argv[5]
expected_promotion_reason_taxonomy_version = sys.argv[6]
expected_promotion_reason_codes_csv = sys.argv[7]

lane_payload = json.loads(lane_report_file.read_text(encoding="utf-8"))
convergence_payload = json.loads(convergence_report_file.read_text(encoding="utf-8"))
if convergence_payload.get("schema_version") != expected_schema:
    raise SystemExit("unexpected sqlite crash-recovery convergence report schema")
if convergence_payload.get("reason_taxonomy_version") != expected_reason_taxonomy_version:
    raise SystemExit("unexpected sqlite crash-recovery convergence reason taxonomy marker")
if convergence_payload.get("reason_codes_csv") != expected_reason_codes_csv:
    raise SystemExit("unexpected sqlite crash-recovery convergence reason codes marker")
if (
    convergence_payload.get("promotion_decision_reason_taxonomy_version")
    != expected_promotion_reason_taxonomy_version
):
    raise SystemExit("unexpected sqlite crash-recovery promotion reason taxonomy marker")
if (
    convergence_payload.get("promotion_decision_reason_codes_csv")
    != expected_promotion_reason_codes_csv
):
    raise SystemExit("unexpected sqlite crash-recovery promotion reason codes marker")

lane_payload["sqlite_crash_replay_evidence_convergence_status"] = convergence_payload.get(
    "evidence_convergence_status"
)
lane_payload["promotion_decision_reason_mapping_status"] = convergence_payload.get(
    "promotion_decision_reason_mapping_status"
)
lane_payload["sqlite_crash_replay_evidence_reason_taxonomy_version"] = (
    convergence_payload.get("reason_taxonomy_version")
)
lane_payload["sqlite_crash_replay_evidence_reason_codes_csv"] = convergence_payload.get(
    "reason_codes_csv"
)
lane_payload["promotion_decision_reason_taxonomy_version"] = (
    convergence_payload.get("promotion_decision_reason_taxonomy_version")
)
lane_payload["promotion_decision_reason_codes_csv"] = convergence_payload.get(
    "promotion_decision_reason_codes_csv"
)
lane_payload["promotion_decision_reason_code"] = convergence_payload.get(
    "promotion_decision_reason_code"
)

lane_report_file.write_text(
    json.dumps(lane_payload, sort_keys=True, indent=2) + "\n",
    encoding="utf-8",
)
PY

if [[ -n "$output_json" ]]; then
  cp "$lane_report" "$output_json"
fi
if [[ -n "$policy_output_json" ]]; then
  cp "$policy_report" "$policy_output_json"
fi
if [[ -n "$convergence_output_json" ]]; then
  cp "$convergence_report" "$convergence_output_json"
fi

echo "status=pass"
echo "final_decision=GO"
echo "lane_mode=$mode"
echo "wal_append_status=verified"
echo "wal_checkpoint_status=verified"
echo "append_checkpoint_integrity_status=verified"
echo "append_checkpoint_reason_taxonomy_version=kamn.runtime.append-checkpoint-integrity-reason-taxonomy.v1"
echo "append_checkpoint_reason_codes_csv=wal_append_marker_missing,wal_checkpoint_marker_missing,append_checkpoint_marker_parity_mismatch"
echo "wal_durability_reason_taxonomy_version=kamn.runtime.wal-durability-reason-taxonomy.v1"
echo "wal_durability_reason_codes_csv=wal_append_rejected,wal_checkpoint_skipped,wal_replay_incomplete"
echo "historical_query_index_status=verified"
echo "historical_query_latency_budget_status=verified"
echo "historical_query_reason_taxonomy_version=kamn.runtime.historical-query-reason-taxonomy.v1"
echo "historical_query_reason_codes_csv=historical_query_index_drift,historical_query_latency_budget_exceeded,historical_query_consistency_mismatch"
echo "journal_replay_drift_detection_status=verified"
echo "checkpoint_divergence_bypass_rejection_status=verified"
echo "journal_replay_reason_taxonomy_version=kamn.runtime.journal-replay-reason-taxonomy.v1"
echo "journal_replay_reason_codes_csv=journal_replay_drift_detected,checkpoint_divergence_bypass_detected"
echo "replay_idempotency_taxonomy_mapping_status=verified"
echo "runbook_marker_parity_status=verified"
echo "replay_idempotency_runbook_reason_taxonomy_version=kamn.runtime.sqlite-crash-recovery-replay-idempotency-runbook-reason-taxonomy.v1"
echo "replay_idempotency_runbook_reason_codes_csv=replay_idempotency_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch"
echo "replay_idempotency_runbook_reason_code=none"
echo "promotion_decision_reason_mapping_status=verified"
echo "promotion_decision_reason_taxonomy_version=${PROMOTION_DECISION_REASON_TAXONOMY_VERSION}"
echo "promotion_decision_reason_codes_csv=${PROMOTION_DECISION_REASON_CODES_CSV}"
echo "promotion_decision_reason_code=none"
echo "crash_recovery_readiness_progress_status=verified"
echo "snapshot_parity_status=verified"
echo "ci_local_recovery_budget_boundary_status=verified"
echo "state_consistency_reason_taxonomy_version=kamn.runtime.crash-recovery-state-consistency-reason-taxonomy.v1"
echo "state_consistency_reason_codes_csv=crash_recovery_readiness_progress_stalled,snapshot_parity_drift_detected,ci_local_recovery_budget_boundary_exceeded"
echo "crash_recovery_promotion_gate_status=verified"
echo "audit_trail_parity_status=verified"
echo "ci_local_promotion_budget_boundary_status=verified"
echo "durability_governance_reason_taxonomy_version=kamn.runtime.durability-governance-reason-taxonomy.v1"
echo "durability_governance_reason_codes_csv=crash_recovery_promotion_stalled,audit_trail_parity_mismatch,ci_local_promotion_budget_boundary_exceeded"
echo "sqlite_crash_recovery_contract_status=verified"
echo "sqlite_crash_recovery_policy_status=verified"
echo "sqlite_crash_replay_evidence_convergence_status=verified"
echo "sqlite_crash_replay_evidence_reason_taxonomy_version=${EVIDENCE_REASON_TAXONOMY_VERSION}"
echo "sqlite_crash_replay_evidence_reason_codes_csv=${EVIDENCE_REASON_CODES_CSV}"
echo "docs_contract_status=verified"
echo "fail_closed_status=verified"
echo "fail_closed_reason_code=sqlite_crash_recovery_policy_fast_gate_exclusion_mismatch"
echo "performance_budget_status=verified"
