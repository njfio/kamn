#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/runtime/run_sqlite_crash_restart_local_heavy_lane.sh"
CHECKER="$ROOT_DIR/scripts/runtime/check_sqlite_crash_restart_local_heavy_policy.sh"
RUNBOOK_DOC="$ROOT_DIR/docs/deploy/kolme_devnet_ops.md"
STRATEGY_DOC="$ROOT_DIR/docs/ci/strategy.md"

for required_exec in "$RUNNER" "$CHECKER"; do
  if [ ! -x "$required_exec" ]; then
    echo "expected sqlite crash-restart local-heavy policy script to be executable: $required_exec" >&2
    exit 1
  fi
done
for required_file in "$RUNBOOK_DOC" "$STRATEGY_DOC"; do
  if [ ! -f "$required_file" ]; then
    echo "expected sqlite crash-restart local-heavy policy source to exist: $required_file" >&2
    exit 1
  fi
done

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT
lane_report="$TMP_DIR/sqlite-crash-restart-local-heavy-lane-report.json"
policy_report="$TMP_DIR/sqlite-crash-restart-local-heavy-policy-report.json"

lane_output="$(
  bash "$RUNNER" \
    --profile combined \
    --mode dry-run \
    --ci-fast-gate PASS \
    --max-seconds 240 \
    --output-json "$lane_report"
)"
if ! printf '%s\n' "$lane_output" | grep -q '^status=pass$'; then
  echo "expected sqlite crash-restart local-heavy lane status=pass marker" >&2
  exit 1
fi

policy_output="$(
  bash "$CHECKER" \
    --report-file "$lane_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --runbook-file "$RUNBOOK_DOC" \
    --strategy-doc "$STRATEGY_DOC" \
    --output-json "$policy_report"
)"
for marker in \
  "status=pass" \
  "final_decision=GO" \
  "sqlite_crash_restart_policy_status=verified" \
  "sqlite_crash_restart_runbook_marker_parity_status=verified" \
  "sqlite_crash_restart_strategy_marker_parity_status=verified" \
  "promotion_decision_reason_mapping_status=verified" \
  "reason_codes_value=none"; do
  if ! printf '%s\n' "$policy_output" | grep -q "^${marker}$"; then
    echo "expected sqlite crash-restart local-heavy policy marker ${marker}" >&2
    exit 1
  fi
done

python3 - "$policy_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.sqlite-crash-restart-local-heavy-policy-report.v1":
    raise SystemExit("unexpected sqlite crash-restart policy report schema")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected sqlite crash-restart policy final_decision=GO")
if payload.get("reason_taxonomy_version") != "kamn.runtime.sqlite-crash-restart-local-heavy-policy-reason-taxonomy.v1":
    raise SystemExit("expected sqlite crash-restart policy reason taxonomy marker")
if payload.get("reason_codes_csv") != "sqlite_crash_restart_policy_required_field_missing,sqlite_crash_restart_policy_marker_mismatch,sqlite_crash_restart_policy_reason_taxonomy_mismatch,sqlite_crash_restart_policy_profile_contract_mismatch,sqlite_crash_restart_policy_runbook_marker_parity_mismatch,sqlite_crash_restart_policy_strategy_marker_parity_mismatch,ci_fast_gate_failed,sqlite_crash_restart_policy_expected_decision_mismatch,sqlite_crash_restart_policy_violation":
    raise SystemExit("expected sqlite crash-restart policy reason codes csv marker")
PY

tampered_report="$TMP_DIR/sqlite-crash-restart-local-heavy-lane-report.tampered.json"
cp "$lane_report" "$tampered_report"
python3 - "$tampered_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["restart_drill_status"] = "failed"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_output="$(
  bash "$CHECKER" \
    --report-file "$tampered_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --runbook-file "$RUNBOOK_DOC" \
    --strategy-doc "$STRATEGY_DOC" \
    --output-json "$TMP_DIR/sqlite-crash-restart-local-heavy-policy-report.tampered.json" 2>&1
)"
tampered_code=$?
set -e
if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered sqlite crash-restart report to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q 'sqlite_crash_restart_policy_profile_contract_mismatch'; then
  echo "expected deterministic fail-closed reason for tampered sqlite crash-restart marker" >&2
  exit 1
fi

drifted_runbook="$TMP_DIR/kolme-devnet-ops.drifted.md"
cp "$RUNBOOK_DOC" "$drifted_runbook"
python3 - "$drifted_runbook" <<'PY'
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
updated = text.replace(
    "sqlite_crash_restart_runbook_marker_parity_status=verified",
    "sqlite_crash_restart_runbook_marker_parity_status=drifted",
    1,
)
if text == updated:
    raise SystemExit("failed to drift sqlite crash-restart runbook marker fixture")
path.write_text(updated, encoding="utf-8")
PY

set +e
runbook_drift_output="$(
  bash "$CHECKER" \
    --report-file "$lane_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --runbook-file "$drifted_runbook" \
    --strategy-doc "$STRATEGY_DOC" \
    --output-json "$TMP_DIR/sqlite-crash-restart-local-heavy-policy-report.runbook-drift.json" 2>&1
)"
runbook_drift_code=$?
set -e
if [ "$runbook_drift_code" -eq 0 ]; then
  echo "expected runbook drift fixture to fail sqlite crash-restart policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$runbook_drift_output" | grep -q 'sqlite_crash_restart_policy_runbook_marker_parity_mismatch'; then
  echo "expected deterministic runbook parity drift reason marker" >&2
  exit 1
fi

drifted_strategy="$TMP_DIR/ci-strategy.drifted.md"
cp "$STRATEGY_DOC" "$drifted_strategy"
python3 - "$drifted_strategy" <<'PY'
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
updated = text.replace(
    "sqlite_crash_restart_local_heavy_policy_reason_taxonomy_version=kamn.runtime.sqlite-crash-restart-local-heavy-policy-reason-taxonomy.v1",
    "sqlite_crash_restart_local_heavy_policy_reason_taxonomy_version=drifted",
    1,
)
if text == updated:
    raise SystemExit("failed to drift sqlite crash-restart strategy marker fixture")
path.write_text(updated, encoding="utf-8")
PY

set +e
strategy_drift_output="$(
  bash "$CHECKER" \
    --report-file "$lane_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --runbook-file "$RUNBOOK_DOC" \
    --strategy-doc "$drifted_strategy" \
    --output-json "$TMP_DIR/sqlite-crash-restart-local-heavy-policy-report.strategy-drift.json" 2>&1
)"
strategy_drift_code=$?
set -e
if [ "$strategy_drift_code" -eq 0 ]; then
  echo "expected strategy drift fixture to fail sqlite crash-restart policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$strategy_drift_output" | grep -q 'sqlite_crash_restart_policy_strategy_marker_parity_mismatch'; then
  echo "expected deterministic strategy parity drift reason marker" >&2
  exit 1
fi

echo "sqlite crash-restart local-heavy policy checker tests passed."
