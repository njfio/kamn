#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_sqlite_crash_recovery_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_sqlite_crash_recovery_live_policy.sh"
TMP_REPORT="$(mktemp)"
TMP_POLICY="$(mktemp)"
TMP_TAMPERED="$(mktemp)"
tmp_tampered_wal_checkpoint=""
tmp_tampered_wal_taxonomy=""
tmp_tampered_historical_query_index=""
tmp_tampered_historical_query_taxonomy=""
tmp_tampered_historical_query_latency=""
cleanup() {
  rm -f "$TMP_REPORT" "$TMP_POLICY" "$TMP_TAMPERED"
  if [[ -n "$tmp_tampered_wal_checkpoint" ]]; then
    rm -f "$tmp_tampered_wal_checkpoint"
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
if payload.get("wal_durability_reason_taxonomy_version") != "kamn.runtime.wal-durability-reason-taxonomy.v1":
    raise SystemExit("expected deterministic wal_durability_reason_taxonomy_version marker")
if payload.get("wal_durability_reason_codes_csv") != "wal_append_rejected,wal_checkpoint_skipped,wal_replay_incomplete":
    raise SystemExit("expected deterministic wal_durability_reason_codes_csv marker")
if payload.get("historical_query_reason_taxonomy_version") != "kamn.runtime.historical-query-reason-taxonomy.v1":
    raise SystemExit("expected deterministic historical_query_reason_taxonomy_version marker")
if payload.get("historical_query_reason_codes_csv") != "historical_query_index_drift,historical_query_latency_budget_exceeded,historical_query_consistency_mismatch":
    raise SystemExit("expected deterministic historical_query_reason_codes_csv marker")
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
