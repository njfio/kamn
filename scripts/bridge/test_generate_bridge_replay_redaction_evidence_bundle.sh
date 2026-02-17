#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)/scripts/lib/common.sh"
GENERATOR="$KAMN_ROOT/scripts/bridge/generate_bridge_replay_redaction_evidence_bundle.sh"
POLICY_CHECKER="$KAMN_ROOT/scripts/bridge/check_bridge_replay_redaction_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$GENERATOR" ]; then
  echo "expected bridge replay/redaction evidence generator to be executable" >&2
  exit 1
fi

if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected bridge replay/redaction policy checker to be executable" >&2
  exit 1
fi

go_replay_report="$TMP_DIR/replay-go.json"
cat >"$go_replay_report" <<'JSON'
{
  "status": "pass",
  "case_count": 3,
  "failed_count": 0,
  "requested_suites": ["bridge_adapter", "discord_bridge"],
  "failed_case_ids": []
}
JSON

go_redaction_report="$TMP_DIR/redaction-go.json"
cat >"$go_redaction_report" <<'JSON'
{
  "status": "pass",
  "mode": "contract",
  "connectors": [
    {"connector": "telegram", "redacted_credential": "tg...12"},
    {"connector": "discord", "redacted_credential": "dc...34"},
    {"connector": "cross_chain", "redacted_credential": "cc...56"}
  ]
}
JSON

go_bundle="$TMP_DIR/bridge-replay-redaction-go.json"
go_output="$(
  bash "$GENERATOR" \
    --output-file "$go_bundle" \
    --lane contract \
    --replay-report-file "$go_replay_report" \
    --redaction-report-file "$go_redaction_report" \
    --ci-fast-gate PASS
)"

assert_eq "$(extract_value "$go_output" "status")" "generated" "expected GO evidence bundle generation to succeed"
assert_eq "$(extract_value "$go_output" "final_decision")" "GO" "expected GO bridge replay/redaction evidence decision"

go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$go_bundle")"
assert_eq "$(extract_value "$go_policy_output" "status")" "ok" "expected GO bridge replay/redaction policy check to pass"
assert_eq "$(extract_value "$go_policy_output" "final_decision")" "GO" "expected GO bridge replay/redaction policy decision"

no_go_replay_report="$TMP_DIR/replay-no-go.json"
cat >"$no_go_replay_report" <<'JSON'
{
  "status": "fail",
  "case_count": 3,
  "failed_count": 1,
  "requested_suites": ["bridge_adapter", "discord_bridge"],
  "failed_case_ids": ["bridge-replay-failed-case-1"]
}
JSON

no_go_redaction_report="$TMP_DIR/redaction-no-go.json"
cat >"$no_go_redaction_report" <<'JSON'
{
  "status": "pass",
  "mode": "contract",
  "connectors": [
    {"connector": "telegram", "redacted_credential": "tg...12"}
  ],
  "leaked_connectors": ["telegram"]
}
JSON

no_go_bundle="$TMP_DIR/bridge-replay-redaction-no-go.json"
no_go_output="$(
  bash "$GENERATOR" \
    --output-file "$no_go_bundle" \
    --lane contract \
    --replay-report-file "$no_go_replay_report" \
    --redaction-report-file "$no_go_redaction_report" \
    --ci-fast-gate PASS
)"

assert_eq "$(extract_value "$no_go_output" "status")" "generated" "expected NO-GO evidence bundle generation to succeed"
assert_eq "$(extract_value "$no_go_output" "final_decision")" "NO-GO" "expected NO-GO bridge replay/redaction evidence decision"

no_go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$no_go_bundle")"
assert_eq "$(extract_value "$no_go_policy_output" "status")" "ok" "expected NO-GO bridge replay/redaction policy check to pass"
assert_eq "$(extract_value "$no_go_policy_output" "final_decision")" "NO-GO" "expected NO-GO bridge replay/redaction policy decision"

tampered_bundle="$TMP_DIR/bridge-replay-redaction-tampered.json"
cp "$no_go_bundle" "$tampered_bundle"
python3 - "$tampered_bundle" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text())
payload["final_decision"] = "GO"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n")
PY

set +e
tampered_output="$(bash "$POLICY_CHECKER" --bundle-file "$tampered_bundle" 2>&1)"
tampered_code=$?
set -e

if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered bridge replay/redaction evidence bundle to fail policy validation" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_output" | grep -q "policy decision mismatch"; then
  echo "expected policy decision mismatch error for tampered bridge replay/redaction bundle" >&2
  exit 1
fi

# Regression: #852
if ! printf '%s\n' "$tampered_output" | grep -q "expected final_decision=NO-GO"; then
  echo "expected explicit NO-GO decision mismatch in bridge replay/redaction regression path" >&2
  exit 1
fi

echo "bridge replay/redaction evidence bundle tests passed."
