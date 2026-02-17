#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
POLICY_SCRIPT="$ROOT_DIR/scripts/runtime/check_full_io_scenario_matrix_live_policy.sh"
TMP_REPORT="$(mktemp)"
TMP_POLICY="$(mktemp)"
TMP_TAMPERED="$(mktemp)"
trap 'rm -f "$TMP_REPORT" "$TMP_POLICY" "$TMP_TAMPERED"' EXIT

if [ ! -x "$POLICY_SCRIPT" ]; then
  echo "expected full I/O scenario matrix policy script to be executable" >&2
  exit 1
fi

cat >"$TMP_REPORT" <<'JSON'
{
  "schema_version": "kamn.runtime.full-io-scenario-matrix-live-report.v1",
  "status": "pass",
  "final_decision": "GO",
  "lane_mode": "dry-run",
  "ci_fast_gate": "PASS",
  "ci_fast_gate_eligibility": "eligible",
  "fast_gate_exclusion_status": "verified",
  "fast_gate_exclusion_reason_code": "full_io_scenario_matrix_run_mode_excluded_from_fast_gate",
  "process_harness_contract_status": "verified",
  "api_route_matrix_status": "verified",
  "auth_failure_matrix_status": "verified",
  "websocket_matrix_status": "verified",
  "multinode_propagation_status": "verified",
  "run_mode_command_status": "dry_run_no_commands_executed",
  "run_mode_command_count": 0,
  "reason_code": "dry_run_no_commands_executed",
  "elapsed_seconds": 1,
  "max_seconds": 120,
  "command_max_seconds": 60,
  "scenario_artifact_paths": {}
}
JSON

policy_output="$(
  bash "$POLICY_SCRIPT" \
    --report-file "$TMP_REPORT" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_POLICY"
)"
if ! printf '%s\n' "$policy_output" | grep -q '^status=ok$'; then
  echo "expected full I/O scenario matrix policy status=ok marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^final_decision=GO$'; then
  echo "expected full I/O scenario matrix policy final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^full_io_scenario_matrix_policy_status=verified$'; then
  echo "expected full I/O scenario matrix policy status marker" >&2
  exit 1
fi

python3 - "$TMP_POLICY" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.full-io-scenario-matrix-live-policy-report.v1":
    raise SystemExit("unexpected full I/O scenario matrix policy schema")
if payload.get("status") != "ok":
    raise SystemExit("expected full I/O scenario matrix policy status=ok")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected full I/O scenario matrix policy final_decision=GO")
if payload.get("full_io_scenario_matrix_policy_status") != "verified":
    raise SystemExit("expected full_io_scenario_matrix_policy_status=verified")
PY

cp "$TMP_REPORT" "$TMP_TAMPERED"
python3 - "$TMP_TAMPERED" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["auth_failure_matrix_status"] = "missing"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_output="$(
  bash "$POLICY_SCRIPT" \
    --report-file "$TMP_TAMPERED" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_POLICY" 2>&1
)"
tampered_code=$?
set -e
if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered full I/O scenario matrix report to fail policy check" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q 'full_io_scenario_matrix_policy_auth_failure_matrix_mismatch'; then
  echo "expected deterministic reason marker for tampered auth matrix status" >&2
  exit 1
fi

tampered_missing_harness_report="$(mktemp)"
cp "$TMP_REPORT" "$tampered_missing_harness_report"
python3 - "$tampered_missing_harness_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload.pop("process_harness_contract_status", None)
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_missing_harness_output="$(
  bash "$POLICY_SCRIPT" \
    --report-file "$tampered_missing_harness_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_POLICY" 2>&1
)"
tampered_missing_harness_code=$?
set -e
rm -f "$tampered_missing_harness_report"
if [ "$tampered_missing_harness_code" -eq 0 ]; then
  echo "expected missing process harness marker to fail policy check" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_missing_harness_output" | grep -q 'full_io_scenario_matrix_policy_process_harness_mismatch'; then
  echo "expected deterministic process harness mismatch reason marker" >&2
  exit 1
fi

tampered_dry_run_count_report="$(mktemp)"
cp "$TMP_REPORT" "$tampered_dry_run_count_report"
python3 - "$tampered_dry_run_count_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["run_mode_command_count"] = 2
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_dry_run_count_output="$(
  bash "$POLICY_SCRIPT" \
    --report-file "$tampered_dry_run_count_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_POLICY" 2>&1
)"
tampered_dry_run_count_code=$?
set -e
rm -f "$tampered_dry_run_count_report"
if [ "$tampered_dry_run_count_code" -eq 0 ]; then
  echo "expected dry-run command-count parity mismatch to fail policy check" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_dry_run_count_output" | grep -q 'full_io_scenario_matrix_policy_dry_run_command_count_mismatch'; then
  echo "expected deterministic dry-run command-count mismatch reason marker" >&2
  exit 1
fi

tampered_dry_run_status_report="$(mktemp)"
cp "$TMP_REPORT" "$tampered_dry_run_status_report"
python3 - "$tampered_dry_run_status_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["run_mode_command_status"] = "executed"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_dry_run_status_output="$(
  bash "$POLICY_SCRIPT" \
    --report-file "$tampered_dry_run_status_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_POLICY" 2>&1
)"
tampered_dry_run_status_code=$?
set -e
rm -f "$tampered_dry_run_status_report"
if [ "$tampered_dry_run_status_code" -eq 0 ]; then
  echo "expected dry-run command-status parity mismatch to fail policy check" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_dry_run_status_output" | grep -q 'full_io_scenario_matrix_policy_dry_run_command_status_mismatch'; then
  echo "expected deterministic dry-run command-status mismatch reason marker" >&2
  exit 1
fi

echo "full I/O scenario matrix policy checker tests passed."
