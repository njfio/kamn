#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LANE_SCRIPT="$ROOT_DIR/scripts/signer/run_signer_incident_recovery_lane.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/signer/check_signer_incident_recovery_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

extract_value() {
  local output="$1"
  local key="$2"
  printf '%s\n' "$output" | awk -F= -v key="$key" '$1 == key { print $2; exit }'
}

assert_eq() {
  local actual="$1"
  local expected="$2"
  local message="$3"
  if [ "$actual" != "$expected" ]; then
    echo "$message: expected '$expected', got '$actual'" >&2
    exit 1
  fi
}

if [ ! -x "$LANE_SCRIPT" ]; then
  echo "expected signer incident recovery lane script to be executable" >&2
  exit 1
fi

if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected signer incident recovery policy checker to be executable" >&2
  exit 1
fi

go_report="$TMP_DIR/signer-incident-recovery-go.json"
KAMN_SIGNER_INCIDENT_RECOVERY_SKIP_COMMANDS=true bash "$LANE_SCRIPT" --output-json "$go_report" >/dev/null
go_policy_output="$(bash "$POLICY_CHECKER" --report-file "$go_report")"
assert_eq "$(extract_value "$go_policy_output" "status")" "ok" "expected signer incident recovery GO policy check status"
assert_eq "$(extract_value "$go_policy_output" "final_decision")" "GO" "expected signer incident recovery GO policy decision"
assert_eq "$(extract_value "$go_policy_output" "failed_checks")" "none" "expected signer incident recovery GO failed checks marker"

no_go_report="$TMP_DIR/signer-incident-recovery-no-go.json"
set +e
KAMN_SIGNER_INCIDENT_RECOVERY_SKIP_COMMANDS=true \
KAMN_SIGNER_INCIDENT_RECOVERY_FORCE_RUNBOOK_GAP=true \
  bash "$LANE_SCRIPT" --output-json "$no_go_report" >/dev/null 2>&1
lane_no_go_code=$?
set -e

if [ "$lane_no_go_code" -eq 0 ]; then
  echo "expected signer incident recovery runbook-gap lane path to fail closed" >&2
  exit 1
fi

no_go_policy_output="$(bash "$POLICY_CHECKER" --report-file "$no_go_report")"
assert_eq "$(extract_value "$no_go_policy_output" "status")" "ok" "expected signer incident recovery NO-GO policy check status"
assert_eq "$(extract_value "$no_go_policy_output" "final_decision")" "NO-GO" "expected signer incident recovery NO-GO policy decision"
if ! printf '%s\n' "$no_go_policy_output" | grep -q "incident_runbook_step_missing"; then
  echo "expected signer incident recovery NO-GO failed checks to include incident_runbook_step_missing" >&2
  exit 1
fi

tampered_report="$TMP_DIR/signer-incident-recovery-no-go-tampered.json"
cp "$no_go_report" "$tampered_report"
python3 - "$tampered_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["final_decision"] = "GO"
payload["reason_key"] = "signer_incident_recovery_reason_codes:GO:v1"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_output="$(bash "$POLICY_CHECKER" --report-file "$tampered_report" 2>&1)"
tampered_code=$?
set -e

if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered signer incident recovery report to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q "policy decision mismatch"; then
  echo "expected signer incident recovery policy decision mismatch for tampered report" >&2
  exit 1
fi

echo "signer incident recovery policy checker tests passed."
