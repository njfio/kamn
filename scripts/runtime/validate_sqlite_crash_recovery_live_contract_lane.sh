#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_sqlite_crash_recovery_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_sqlite_crash_recovery_live_policy.sh"
STRATEGY_DOC="$ROOT_DIR/docs/ci/strategy.md"

output_json=""
policy_output_json=""
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
for required_exec in "$VALIDATION_SCRIPT" "$POLICY_CHECKER"; do
  if [ ! -x "$required_exec" ]; then
    echo "expected required executable script '$required_exec'" >&2
    exit 1
  fi
done
if [ ! -f "$STRATEGY_DOC" ]; then
  echo "expected required documentation file '$STRATEGY_DOC'" >&2
  exit 1
fi

start_epoch="$(date +%s)"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

summary_report="$TMP_DIR/sqlite-crash-recovery-live-summary.json"
policy_report="$TMP_DIR/sqlite-crash-recovery-live-policy.json"
tampered_report="$TMP_DIR/sqlite-crash-recovery-live-summary.tampered.json"

validation_output="$(
  bash "$VALIDATION_SCRIPT" \
    --mode "$mode" \
    --max-seconds "$max_seconds" \
    --ci-fast-gate "$ci_fast_gate" \
    --output-json "$summary_report"
)"
if ! printf '%s\n' "$validation_output" | grep -q '^status=pass$'; then
  echo "expected sqlite crash-recovery live validation status=pass marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^final_decision=GO$'; then
  echo "expected sqlite crash-recovery live validation final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^fast_gate_exclusion_status=verified$'; then
  echo "expected sqlite crash-recovery live validation fast-gate exclusion marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^sqlite_crash_recovery_state_replay_status=verified$'; then
  echo "expected sqlite crash-recovery live validation replay marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^sqlite_crash_recovery_abrupt_kill_status=verified$'; then
  echo "expected sqlite crash-recovery live validation abrupt-kill marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^wal_append_status=verified$'; then
  echo "expected sqlite crash-recovery live validation wal-append marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^wal_checkpoint_status=verified$'; then
  echo "expected sqlite crash-recovery live validation wal-checkpoint marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^wal_durability_reason_taxonomy_version=kamn.runtime.wal-durability-reason-taxonomy.v1$'; then
  echo "expected sqlite crash-recovery live validation wal-durability taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^wal_durability_reason_codes_csv=wal_append_rejected,wal_checkpoint_skipped,wal_replay_incomplete$'; then
  echo "expected sqlite crash-recovery live validation wal-durability taxonomy csv marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^historical_query_index_status=verified$'; then
  echo "expected sqlite crash-recovery live validation historical-query index marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^historical_query_latency_budget_status=verified$'; then
  echo "expected sqlite crash-recovery live validation historical-query latency budget marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^historical_query_reason_taxonomy_version=kamn.runtime.historical-query-reason-taxonomy.v1$'; then
  echo "expected sqlite crash-recovery live validation historical-query taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^historical_query_reason_codes_csv=historical_query_index_drift,historical_query_latency_budget_exceeded,historical_query_consistency_mismatch$'; then
  echo "expected sqlite crash-recovery live validation historical-query taxonomy csv marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^journal_replay_drift_detection_status=verified$'; then
  echo "expected sqlite crash-recovery live validation journal replay drift detection marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^checkpoint_divergence_bypass_rejection_status=verified$'; then
  echo "expected sqlite crash-recovery live validation checkpoint divergence bypass rejection marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^journal_replay_reason_taxonomy_version=kamn.runtime.journal-replay-reason-taxonomy.v1$'; then
  echo "expected sqlite crash-recovery live validation journal replay taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^journal_replay_reason_codes_csv=journal_replay_drift_detected,checkpoint_divergence_bypass_detected$'; then
  echo "expected sqlite crash-recovery live validation journal replay taxonomy csv marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^crash_recovery_promotion_gate_status=verified$'; then
  echo "expected sqlite crash-recovery live validation promotion-gate marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^audit_trail_parity_status=verified$'; then
  echo "expected sqlite crash-recovery live validation audit-trail parity marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^ci_local_promotion_budget_boundary_status=verified$'; then
  echo "expected sqlite crash-recovery live validation ci-local promotion budget boundary marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^durability_governance_reason_taxonomy_version=kamn.runtime.durability-governance-reason-taxonomy.v1$'; then
  echo "expected sqlite crash-recovery live validation durability-governance taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^durability_governance_reason_codes_csv=crash_recovery_promotion_stalled,audit_trail_parity_mismatch,ci_local_promotion_budget_boundary_exceeded$'; then
  echo "expected sqlite crash-recovery live validation durability-governance taxonomy csv marker" >&2
  exit 1
fi

policy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$summary_report" \
    --expected-final-decision GO \
    --ci-fast-gate "$ci_fast_gate" \
    --output-json "$policy_report"
)"
if ! printf '%s\n' "$policy_output" | grep -q '^status=ok$'; then
  echo "expected sqlite crash-recovery policy checker status=ok marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^final_decision=GO$'; then
  echo "expected sqlite crash-recovery policy checker final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^sqlite_crash_recovery_policy_status=verified$'; then
  echo "expected sqlite crash-recovery policy checker status marker" >&2
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
    --output-json "$TMP_DIR/sqlite-crash-recovery-live-policy.tampered.json" 2>&1
)"
tampered_policy_code=$?
set -e
if [ "$tampered_policy_code" -eq 0 ]; then
  echo "expected tampered sqlite crash-recovery report to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_policy_output" | grep -q 'sqlite_crash_recovery_policy_fast_gate_exclusion_mismatch'; then
  echo "expected deterministic fail-closed reason for tampered sqlite crash-recovery report" >&2
  exit 1
fi

for required_ref in \
  "validate_sqlite_crash_recovery_live.sh" \
  "check_sqlite_crash_recovery_live_policy.sh" \
  "validate_sqlite_crash_recovery_live_contract_lane.sh" \
  "test_validate_sqlite_crash_recovery_live.sh" \
  "test_check_sqlite_crash_recovery_live_policy.sh" \
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

if [[ -n "$output_json" ]]; then
  cp "$lane_report" "$output_json"
fi
if [[ -n "$policy_output_json" ]]; then
  cp "$policy_report" "$policy_output_json"
fi

echo "status=pass"
echo "final_decision=GO"
echo "lane_mode=$mode"
echo "wal_append_status=verified"
echo "wal_checkpoint_status=verified"
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
echo "crash_recovery_promotion_gate_status=verified"
echo "audit_trail_parity_status=verified"
echo "ci_local_promotion_budget_boundary_status=verified"
echo "durability_governance_reason_taxonomy_version=kamn.runtime.durability-governance-reason-taxonomy.v1"
echo "durability_governance_reason_codes_csv=crash_recovery_promotion_stalled,audit_trail_parity_mismatch,ci_local_promotion_budget_boundary_exceeded"
echo "sqlite_crash_recovery_contract_status=verified"
echo "sqlite_crash_recovery_policy_status=verified"
echo "docs_contract_status=verified"
echo "fail_closed_status=verified"
echo "fail_closed_reason_code=sqlite_crash_recovery_policy_fast_gate_exclusion_mismatch"
echo "performance_budget_status=verified"
