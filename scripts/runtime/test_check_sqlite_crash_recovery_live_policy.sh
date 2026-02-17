#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_sqlite_crash_recovery_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_sqlite_crash_recovery_live_policy.sh"
TMP_REPORT="$(mktemp)"
TMP_POLICY="$(mktemp)"
TMP_TAMPERED="$(mktemp)"
tmp_tampered_wal_checkpoint=""
tmp_tampered_wal_append=""
tmp_tampered_wal_taxonomy=""
tmp_tampered_historical_query_index=""
tmp_tampered_historical_query_taxonomy=""
tmp_tampered_historical_query_latency=""
tmp_tampered_journal_replay_drift=""
tmp_tampered_journal_replay_taxonomy=""
tmp_tampered_checkpoint_divergence_bypass=""
tmp_tampered_recovery_readiness_progress=""
tmp_tampered_snapshot_parity=""
tmp_tampered_state_consistency_taxonomy=""
tmp_tampered_promotion_gate=""
tmp_tampered_audit_parity=""
tmp_tampered_durability_taxonomy=""
tmp_tampered_ci_local_budget=""
cleanup() {
  rm -f "$TMP_REPORT" "$TMP_POLICY" "$TMP_TAMPERED"
  if [[ -n "$tmp_tampered_wal_checkpoint" ]]; then
    rm -f "$tmp_tampered_wal_checkpoint"
  fi
  if [[ -n "$tmp_tampered_wal_append" ]]; then
    rm -f "$tmp_tampered_wal_append"
  fi
  if [[ -n "$tmp_tampered_wal_taxonomy" ]]; then
    rm -f "$tmp_tampered_wal_taxonomy"
  fi
  if [[ -n "$tmp_tampered_historical_query_index" ]]; then
    rm -f "$tmp_tampered_historical_query_index"
  fi
  if [[ -n "$tmp_tampered_historical_query_taxonomy" ]]; then
    rm -f "$tmp_tampered_historical_query_taxonomy"
  fi
  if [[ -n "$tmp_tampered_historical_query_latency" ]]; then
    rm -f "$tmp_tampered_historical_query_latency"
  fi
  if [[ -n "$tmp_tampered_journal_replay_drift" ]]; then
    rm -f "$tmp_tampered_journal_replay_drift"
  fi
  if [[ -n "$tmp_tampered_journal_replay_taxonomy" ]]; then
    rm -f "$tmp_tampered_journal_replay_taxonomy"
  fi
  if [[ -n "$tmp_tampered_checkpoint_divergence_bypass" ]]; then
    rm -f "$tmp_tampered_checkpoint_divergence_bypass"
  fi
  if [[ -n "$tmp_tampered_recovery_readiness_progress" ]]; then
    rm -f "$tmp_tampered_recovery_readiness_progress"
  fi
  if [[ -n "$tmp_tampered_snapshot_parity" ]]; then
    rm -f "$tmp_tampered_snapshot_parity"
  fi
  if [[ -n "$tmp_tampered_state_consistency_taxonomy" ]]; then
    rm -f "$tmp_tampered_state_consistency_taxonomy"
  fi
  if [[ -n "$tmp_tampered_promotion_gate" ]]; then
    rm -f "$tmp_tampered_promotion_gate"
  fi
  if [[ -n "$tmp_tampered_audit_parity" ]]; then
    rm -f "$tmp_tampered_audit_parity"
  fi
  if [[ -n "$tmp_tampered_durability_taxonomy" ]]; then
    rm -f "$tmp_tampered_durability_taxonomy"
  fi
  if [[ -n "$tmp_tampered_ci_local_budget" ]]; then
    rm -f "$tmp_tampered_ci_local_budget"
  fi
}
trap cleanup EXIT

if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected sqlite crash-recovery validation script to be executable" >&2
  exit 1
fi
if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected sqlite crash-recovery policy checker script to be executable" >&2
  exit 1
fi

bash "$VALIDATION_SCRIPT" --mode dry-run --ci-fast-gate PASS --output-json "$TMP_REPORT" >/dev/null

policy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$TMP_REPORT" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_POLICY"
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
if ! printf '%s\n' "$policy_output" | grep -q '^append_checkpoint_integrity_status=verified$'; then
  echo "expected sqlite crash-recovery policy checker append-checkpoint integrity marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^append_checkpoint_reason_taxonomy_version=kamn.runtime.append-checkpoint-integrity-reason-taxonomy.v1$'; then
  echo "expected sqlite crash-recovery policy checker append-checkpoint reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^append_checkpoint_reason_codes_csv=wal_append_marker_missing,wal_checkpoint_marker_missing,append_checkpoint_marker_parity_mismatch$'; then
  echo "expected sqlite crash-recovery policy checker append-checkpoint reason taxonomy csv marker" >&2
  exit 1
fi

python3 - "$TMP_POLICY" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.sqlite-crash-recovery-live-policy-report.v1":
    raise SystemExit("unexpected sqlite crash-recovery policy schema")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected sqlite crash-recovery policy final_decision=GO")
if payload.get("sqlite_crash_recovery_policy_status") != "verified":
    raise SystemExit("expected sqlite_crash_recovery_policy_status=verified")
if payload.get("append_checkpoint_integrity_status") != "verified":
    raise SystemExit("expected deterministic append_checkpoint_integrity_status marker")
if payload.get("append_checkpoint_reason_taxonomy_version") != "kamn.runtime.append-checkpoint-integrity-reason-taxonomy.v1":
    raise SystemExit("expected deterministic append_checkpoint_reason_taxonomy_version marker")
if payload.get("append_checkpoint_reason_codes_csv") != "wal_append_marker_missing,wal_checkpoint_marker_missing,append_checkpoint_marker_parity_mismatch":
    raise SystemExit("expected deterministic append_checkpoint_reason_codes_csv marker")
if payload.get("wal_durability_reason_taxonomy_version") != "kamn.runtime.wal-durability-reason-taxonomy.v1":
    raise SystemExit("expected deterministic wal_durability_reason_taxonomy_version marker")
if payload.get("wal_durability_reason_codes_csv") != "wal_append_rejected,wal_checkpoint_skipped,wal_replay_incomplete":
    raise SystemExit("expected deterministic wal_durability_reason_codes_csv marker")
if payload.get("historical_query_reason_taxonomy_version") != "kamn.runtime.historical-query-reason-taxonomy.v1":
    raise SystemExit("expected deterministic historical_query_reason_taxonomy_version marker")
if payload.get("historical_query_reason_codes_csv") != "historical_query_index_drift,historical_query_latency_budget_exceeded,historical_query_consistency_mismatch":
    raise SystemExit("expected deterministic historical_query_reason_codes_csv marker")
if payload.get("journal_replay_drift_detection_status") != "verified":
    raise SystemExit("expected deterministic journal_replay_drift_detection_status marker")
if payload.get("checkpoint_divergence_bypass_rejection_status") != "verified":
    raise SystemExit("expected deterministic checkpoint_divergence_bypass_rejection_status marker")
if payload.get("journal_replay_reason_taxonomy_version") != "kamn.runtime.journal-replay-reason-taxonomy.v1":
    raise SystemExit("expected deterministic journal_replay_reason_taxonomy_version marker")
if payload.get("journal_replay_reason_codes_csv") != "journal_replay_drift_detected,checkpoint_divergence_bypass_detected":
    raise SystemExit("expected deterministic journal_replay_reason_codes_csv marker")
if payload.get("crash_recovery_readiness_progress_status") != "verified":
    raise SystemExit("expected deterministic crash_recovery_readiness_progress_status marker")
if payload.get("snapshot_parity_status") != "verified":
    raise SystemExit("expected deterministic snapshot_parity_status marker")
if payload.get("ci_local_recovery_budget_boundary_status") != "verified":
    raise SystemExit("expected deterministic ci_local_recovery_budget_boundary_status marker")
if payload.get("state_consistency_reason_taxonomy_version") != "kamn.runtime.crash-recovery-state-consistency-reason-taxonomy.v1":
    raise SystemExit("expected deterministic state_consistency_reason_taxonomy_version marker")
if payload.get("state_consistency_reason_codes_csv") != "crash_recovery_readiness_progress_stalled,snapshot_parity_drift_detected,ci_local_recovery_budget_boundary_exceeded":
    raise SystemExit("expected deterministic state_consistency_reason_codes_csv marker")
if payload.get("durability_governance_reason_taxonomy_version") != "kamn.runtime.durability-governance-reason-taxonomy.v1":
    raise SystemExit("expected deterministic durability_governance_reason_taxonomy_version marker")
if payload.get("durability_governance_reason_codes_csv") != "crash_recovery_promotion_stalled,audit_trail_parity_mismatch,ci_local_promotion_budget_boundary_exceeded":
    raise SystemExit("expected deterministic durability_governance_reason_codes_csv marker")
PY

cp "$TMP_REPORT" "$TMP_TAMPERED"
python3 - "$TMP_TAMPERED" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["fast_gate_exclusion_status"] = "mismatch-marker"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$TMP_TAMPERED" \
    --expected-final-decision GO \
    --ci-fast-gate PASS 2>&1
)"
tampered_code=$?
set -e
if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered sqlite crash-recovery report to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q 'sqlite_crash_recovery_policy_fast_gate_exclusion_mismatch'; then
  echo "expected deterministic fail-closed reason for tampered sqlite crash-recovery report" >&2
  exit 1
fi

tmp_tampered_wal_checkpoint="$(mktemp)"
cp "$TMP_REPORT" "$tmp_tampered_wal_checkpoint"
python3 - "$tmp_tampered_wal_checkpoint" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["wal_checkpoint_status"] = "drifted"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
wal_checkpoint_tampered_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tmp_tampered_wal_checkpoint" \
    --expected-final-decision GO \
    --ci-fast-gate PASS 2>&1
)"
wal_checkpoint_tampered_code=$?
set -e
if [ "$wal_checkpoint_tampered_code" -eq 0 ]; then
  echo "expected tampered wal-checkpoint sqlite crash-recovery report to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$wal_checkpoint_tampered_output" | grep -q 'sqlite_crash_recovery_policy_wal_checkpoint_status_mismatch'; then
  echo "expected deterministic wal-checkpoint mismatch reason for tampered sqlite crash-recovery report" >&2
  exit 1
fi
if ! printf '%s\n' "$wal_checkpoint_tampered_output" | grep -q 'sqlite_crash_recovery_policy_append_checkpoint_parity_mismatch'; then
  echo "expected deterministic append-checkpoint parity mismatch reason for tampered sqlite crash-recovery report" >&2
  exit 1
fi

tmp_tampered_wal_append="$(mktemp)"
cp "$TMP_REPORT" "$tmp_tampered_wal_append"
python3 - "$tmp_tampered_wal_append" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["wal_append_status"] = "drifted"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
wal_append_tampered_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tmp_tampered_wal_append" \
    --expected-final-decision GO \
    --ci-fast-gate PASS 2>&1
)"
wal_append_tampered_code=$?
set -e
if [ "$wal_append_tampered_code" -eq 0 ]; then
  echo "expected tampered wal-append sqlite crash-recovery report to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$wal_append_tampered_output" | grep -q 'sqlite_crash_recovery_policy_wal_append_status_mismatch'; then
  echo "expected deterministic wal-append mismatch reason for tampered sqlite crash-recovery report" >&2
  exit 1
fi

tmp_tampered_wal_taxonomy="$(mktemp)"
cp "$TMP_REPORT" "$tmp_tampered_wal_taxonomy"
python3 - "$tmp_tampered_wal_taxonomy" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["wal_durability_reason_taxonomy_version"] = "tampered-taxonomy"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
wal_taxonomy_tampered_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tmp_tampered_wal_taxonomy" \
    --expected-final-decision GO \
    --ci-fast-gate PASS 2>&1
)"
wal_taxonomy_tampered_code=$?
set -e
if [ "$wal_taxonomy_tampered_code" -eq 0 ]; then
  echo "expected tampered wal-durability taxonomy sqlite crash-recovery report to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$wal_taxonomy_tampered_output" | grep -q 'sqlite_crash_recovery_policy_wal_durability_reason_taxonomy_version_mismatch'; then
  echo "expected deterministic wal-durability reason taxonomy mismatch for tampered sqlite crash-recovery report" >&2
  exit 1
fi

tmp_tampered_historical_query_index="$(mktemp)"
cp "$TMP_REPORT" "$tmp_tampered_historical_query_index"
python3 - "$tmp_tampered_historical_query_index" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["historical_query_index_status"] = "drifted"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
historical_query_index_tampered_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tmp_tampered_historical_query_index" \
    --expected-final-decision GO \
    --ci-fast-gate PASS 2>&1
)"
historical_query_index_tampered_code=$?
set -e
if [ "$historical_query_index_tampered_code" -eq 0 ]; then
  echo "expected tampered historical-query index sqlite crash-recovery report to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$historical_query_index_tampered_output" | grep -q 'sqlite_crash_recovery_policy_historical_query_index_status_mismatch'; then
  echo "expected deterministic historical-query index mismatch reason for tampered sqlite crash-recovery report" >&2
  exit 1
fi

tmp_tampered_historical_query_taxonomy="$(mktemp)"
cp "$TMP_REPORT" "$tmp_tampered_historical_query_taxonomy"
python3 - "$tmp_tampered_historical_query_taxonomy" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["historical_query_reason_taxonomy_version"] = "tampered-taxonomy"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
historical_query_taxonomy_tampered_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tmp_tampered_historical_query_taxonomy" \
    --expected-final-decision GO \
    --ci-fast-gate PASS 2>&1
)"
historical_query_taxonomy_tampered_code=$?
set -e
if [ "$historical_query_taxonomy_tampered_code" -eq 0 ]; then
  echo "expected tampered historical-query taxonomy sqlite crash-recovery report to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$historical_query_taxonomy_tampered_output" | grep -q 'sqlite_crash_recovery_policy_historical_query_reason_taxonomy_version_mismatch'; then
  echo "expected deterministic historical-query taxonomy mismatch reason for tampered sqlite crash-recovery report" >&2
  exit 1
fi

tmp_tampered_journal_replay_drift="$(mktemp)"
cp "$TMP_REPORT" "$tmp_tampered_journal_replay_drift"
python3 - "$tmp_tampered_journal_replay_drift" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["journal_replay_drift_detection_status"] = "drifted"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
journal_replay_drift_tampered_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tmp_tampered_journal_replay_drift" \
    --expected-final-decision GO \
    --ci-fast-gate PASS 2>&1
)"
journal_replay_drift_tampered_code=$?
set -e
if [ "$journal_replay_drift_tampered_code" -eq 0 ]; then
  echo "expected journal replay drift bypass sqlite crash-recovery report to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$journal_replay_drift_tampered_output" | grep -q 'sqlite_crash_recovery_policy_journal_replay_drift_detection_status_mismatch'; then
  echo "expected deterministic journal replay drift mismatch reason for tampered sqlite crash-recovery report" >&2
  exit 1
fi

tmp_tampered_journal_replay_taxonomy="$(mktemp)"
cp "$TMP_REPORT" "$tmp_tampered_journal_replay_taxonomy"
python3 - "$tmp_tampered_journal_replay_taxonomy" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["journal_replay_reason_taxonomy_version"] = "tampered-taxonomy"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
journal_replay_taxonomy_tampered_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tmp_tampered_journal_replay_taxonomy" \
    --expected-final-decision GO \
    --ci-fast-gate PASS 2>&1
)"
journal_replay_taxonomy_tampered_code=$?
set -e
if [ "$journal_replay_taxonomy_tampered_code" -eq 0 ]; then
  echo "expected journal replay taxonomy drift sqlite crash-recovery report to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$journal_replay_taxonomy_tampered_output" | grep -q 'sqlite_crash_recovery_policy_journal_replay_reason_taxonomy_version_mismatch'; then
  echo "expected deterministic journal replay taxonomy mismatch reason for tampered sqlite crash-recovery report" >&2
  exit 1
fi

tmp_tampered_checkpoint_divergence_bypass="$(mktemp)"
cp "$TMP_REPORT" "$tmp_tampered_checkpoint_divergence_bypass"
python3 - "$tmp_tampered_checkpoint_divergence_bypass" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["checkpoint_divergence_bypass_rejection_status"] = "drifted"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
checkpoint_divergence_bypass_tampered_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tmp_tampered_checkpoint_divergence_bypass" \
    --expected-final-decision GO \
    --ci-fast-gate PASS 2>&1
)"
checkpoint_divergence_bypass_tampered_code=$?
set -e
if [ "$checkpoint_divergence_bypass_tampered_code" -eq 0 ]; then
  echo "expected checkpoint divergence bypass sqlite crash-recovery report to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$checkpoint_divergence_bypass_tampered_output" | grep -q 'sqlite_crash_recovery_policy_checkpoint_divergence_bypass_rejection_status_mismatch'; then
  echo "expected deterministic checkpoint divergence bypass mismatch reason for tampered sqlite crash-recovery report" >&2
  exit 1
fi

tmp_tampered_recovery_readiness_progress="$(mktemp)"
cp "$TMP_REPORT" "$tmp_tampered_recovery_readiness_progress"
python3 - "$tmp_tampered_recovery_readiness_progress" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["crash_recovery_readiness_progress_status"] = "drifted"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
recovery_readiness_progress_tampered_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tmp_tampered_recovery_readiness_progress" \
    --expected-final-decision GO \
    --ci-fast-gate PASS 2>&1
)"
recovery_readiness_progress_tampered_code=$?
set -e
if [ "$recovery_readiness_progress_tampered_code" -eq 0 ]; then
  echo "expected crash-recovery readiness progress stall bypass sqlite crash-recovery report to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$recovery_readiness_progress_tampered_output" | grep -q 'sqlite_crash_recovery_policy_crash_recovery_readiness_progress_status_mismatch'; then
  echo "expected deterministic crash-recovery readiness progress mismatch reason for tampered sqlite crash-recovery report" >&2
  exit 1
fi

tmp_tampered_snapshot_parity="$(mktemp)"
cp "$TMP_REPORT" "$tmp_tampered_snapshot_parity"
python3 - "$tmp_tampered_snapshot_parity" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["snapshot_parity_status"] = "drifted"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
snapshot_parity_tampered_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tmp_tampered_snapshot_parity" \
    --expected-final-decision GO \
    --ci-fast-gate PASS 2>&1
)"
snapshot_parity_tampered_code=$?
set -e
if [ "$snapshot_parity_tampered_code" -eq 0 ]; then
  echo "expected snapshot parity drift bypass sqlite crash-recovery report to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$snapshot_parity_tampered_output" | grep -q 'sqlite_crash_recovery_policy_snapshot_parity_status_mismatch'; then
  echo "expected deterministic snapshot parity mismatch reason for tampered sqlite crash-recovery report" >&2
  exit 1
fi

tmp_tampered_state_consistency_taxonomy="$(mktemp)"
cp "$TMP_REPORT" "$tmp_tampered_state_consistency_taxonomy"
python3 - "$tmp_tampered_state_consistency_taxonomy" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["state_consistency_reason_taxonomy_version"] = "tampered-taxonomy"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
state_consistency_taxonomy_tampered_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tmp_tampered_state_consistency_taxonomy" \
    --expected-final-decision GO \
    --ci-fast-gate PASS 2>&1
)"
state_consistency_taxonomy_tampered_code=$?
set -e
if [ "$state_consistency_taxonomy_tampered_code" -eq 0 ]; then
  echo "expected state-consistency taxonomy drift sqlite crash-recovery report to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$state_consistency_taxonomy_tampered_output" | grep -q 'sqlite_crash_recovery_policy_state_consistency_reason_taxonomy_version_mismatch'; then
  echo "expected deterministic state-consistency taxonomy mismatch reason for tampered sqlite crash-recovery report" >&2
  exit 1
fi

tmp_tampered_historical_query_latency="$(mktemp)"
cp "$TMP_REPORT" "$tmp_tampered_historical_query_latency"
python3 - "$tmp_tampered_historical_query_latency" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["historical_query_latency_budget_ms"] = 1
payload["max_observed_historical_query_latency_ms"] = 9
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
historical_query_latency_tampered_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tmp_tampered_historical_query_latency" \
    --expected-final-decision GO \
    --ci-fast-gate PASS 2>&1
)"
historical_query_latency_tampered_code=$?
set -e
if [ "$historical_query_latency_tampered_code" -eq 0 ]; then
  echo "expected historical-query latency budget bypass sqlite crash-recovery report to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$historical_query_latency_tampered_output" | grep -q 'sqlite_crash_recovery_policy_historical_query_latency_budget_exceeded'; then
  echo "expected deterministic historical-query latency budget bypass reason for tampered sqlite crash-recovery report" >&2
  exit 1
fi

tmp_tampered_promotion_gate="$(mktemp)"
cp "$TMP_REPORT" "$tmp_tampered_promotion_gate"
python3 - "$tmp_tampered_promotion_gate" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["crash_recovery_promotion_gate_status"] = "drifted"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
promotion_gate_tampered_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tmp_tampered_promotion_gate" \
    --expected-final-decision GO \
    --ci-fast-gate PASS 2>&1
)"
promotion_gate_tampered_code=$?
set -e
if [ "$promotion_gate_tampered_code" -eq 0 ]; then
  echo "expected tampered promotion-gate sqlite crash-recovery report to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$promotion_gate_tampered_output" | grep -q 'sqlite_crash_recovery_policy_crash_recovery_promotion_gate_status_mismatch'; then
  echo "expected deterministic promotion-gate mismatch reason for tampered sqlite crash-recovery report" >&2
  exit 1
fi

tmp_tampered_audit_parity="$(mktemp)"
cp "$TMP_REPORT" "$tmp_tampered_audit_parity"
python3 - "$tmp_tampered_audit_parity" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["audit_trail_parity_status"] = "drifted"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
audit_parity_tampered_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tmp_tampered_audit_parity" \
    --expected-final-decision GO \
    --ci-fast-gate PASS 2>&1
)"
audit_parity_tampered_code=$?
set -e
if [ "$audit_parity_tampered_code" -eq 0 ]; then
  echo "expected tampered audit-parity sqlite crash-recovery report to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_parity_tampered_output" | grep -q 'sqlite_crash_recovery_policy_audit_trail_parity_status_mismatch'; then
  echo "expected deterministic audit-parity mismatch reason for tampered sqlite crash-recovery report" >&2
  exit 1
fi

tmp_tampered_durability_taxonomy="$(mktemp)"
cp "$TMP_REPORT" "$tmp_tampered_durability_taxonomy"
python3 - "$tmp_tampered_durability_taxonomy" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["durability_governance_reason_taxonomy_version"] = "tampered-taxonomy"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
durability_taxonomy_tampered_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tmp_tampered_durability_taxonomy" \
    --expected-final-decision GO \
    --ci-fast-gate PASS 2>&1
)"
durability_taxonomy_tampered_code=$?
set -e
if [ "$durability_taxonomy_tampered_code" -eq 0 ]; then
  echo "expected tampered durability-taxonomy sqlite crash-recovery report to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$durability_taxonomy_tampered_output" | grep -q 'sqlite_crash_recovery_policy_durability_governance_reason_taxonomy_version_mismatch'; then
  echo "expected deterministic durability-taxonomy mismatch reason for tampered sqlite crash-recovery report" >&2
  exit 1
fi

tmp_tampered_ci_local_budget="$(mktemp)"
cp "$TMP_REPORT" "$tmp_tampered_ci_local_budget"
python3 - "$tmp_tampered_ci_local_budget" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["ci_local_promotion_max_seconds"] = 10
payload["max_seconds"] = 120
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
ci_local_budget_tampered_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tmp_tampered_ci_local_budget" \
    --expected-final-decision GO \
    --ci-fast-gate PASS 2>&1
)"
ci_local_budget_tampered_code=$?
set -e
if [ "$ci_local_budget_tampered_code" -eq 0 ]; then
  echo "expected ci-local budget bypass sqlite crash-recovery report to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$ci_local_budget_tampered_output" | grep -q 'sqlite_crash_recovery_policy_ci_local_promotion_budget_boundary_exceeded'; then
  echo "expected deterministic ci-local budget bypass reason for tampered sqlite crash-recovery report" >&2
  exit 1
fi

python3 - "$tampered_output" <<'PY'
import sys

output = sys.argv[1]
failed_checks = ""
for line in output.splitlines():
    if line.startswith("failed_checks="):
        failed_checks = line.split("=", 1)[1]
        break
reason_codes = [entry for entry in failed_checks.split(",") if entry]
if "sqlite_crash_recovery_policy_fast_gate_exclusion_mismatch" not in reason_codes:
    raise SystemExit("expected parser to recover deterministic sqlite crash-recovery reason code")
PY

echo "sqlite crash-recovery live policy tests passed."
