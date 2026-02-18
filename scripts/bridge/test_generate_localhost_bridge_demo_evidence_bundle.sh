#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)/scripts/lib/common.sh"
GENERATOR="$KAMN_ROOT/scripts/bridge/generate_localhost_bridge_demo_evidence_bundle.sh"
POLICY_CHECKER="$KAMN_ROOT/scripts/bridge/check_localhost_bridge_demo_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$GENERATOR" ]; then
  echo "expected localhost bridge demo evidence generator to be executable" >&2
  exit 1
fi

if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected localhost bridge demo evidence policy checker to be executable" >&2
  exit 1
fi

go_relay_output="$TMP_DIR/relay-go.out"
cat >"$go_relay_output" <<'EOF_RELAY_GO'
bridge_demo_signed_transport=pass
bridge_demo_relay_contracts=pass
localhost bridge relay demo contract lane tests passed.
EOF_RELAY_GO

go_replay_report="$TMP_DIR/replay-go.json"
bash "$KAMN_ROOT/scripts/lib/write_json_file.sh" "$go_replay_report" <<'EOF_REPLAY_GO'
{
  "status": "pass",
  "case_count": 2,
  "failed_count": 0,
  "requested_suites": ["bridge_adapter"],
  "failed_case_ids": []
}
EOF_REPLAY_GO

go_bundle="$TMP_DIR/localhost-bridge-demo-go.json"
go_output="$(
  bash "$GENERATOR" \
    --output-file "$go_bundle" \
    --lane contract \
    --relay-lane-output-file "$go_relay_output" \
    --replay-report-file "$go_replay_report" \
    --ci-fast-gate PASS
)"

assert_eq "$(extract_value "$go_output" "status")" "generated" "expected GO evidence bundle generation to succeed"
assert_eq "$(extract_value "$go_output" "final_decision")" "GO" "expected GO localhost bridge demo evidence decision"

go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$go_bundle")"
assert_eq "$(extract_value "$go_policy_output" "status")" "ok" "expected GO localhost bridge demo policy check to pass"
assert_eq "$(extract_value "$go_policy_output" "final_decision")" "GO" "expected GO localhost bridge demo policy decision"

no_go_relay_output="$TMP_DIR/relay-no-go.out"
cat >"$no_go_relay_output" <<'EOF_RELAY_NOGO'
bridge_demo_signed_transport=pass
bridge_demo_relay_contracts=pass
localhost bridge relay demo contract lane tests passed.
EOF_RELAY_NOGO

no_go_replay_report="$TMP_DIR/replay-no-go.json"
bash "$KAMN_ROOT/scripts/lib/write_json_file.sh" "$no_go_replay_report" <<'EOF_REPLAY_NOGO'
{
  "status": "pass",
  "case_count": 2,
  "failed_count": 1,
  "requested_suites": ["bridge_adapter"],
  "failed_case_ids": ["bridge_adapter::outbound_rejects_unauthorized_approver"]
}
EOF_REPLAY_NOGO

no_go_bundle="$TMP_DIR/localhost-bridge-demo-no-go.json"
no_go_output="$(
  bash "$GENERATOR" \
    --output-file "$no_go_bundle" \
    --lane contract \
    --relay-lane-output-file "$no_go_relay_output" \
    --replay-report-file "$no_go_replay_report" \
    --ci-fast-gate PASS
)"

assert_eq "$(extract_value "$no_go_output" "status")" "generated" "expected NO-GO evidence bundle generation to succeed"
assert_eq "$(extract_value "$no_go_output" "final_decision")" "NO-GO" "expected NO-GO localhost bridge demo evidence decision"

if ! grep -q '"bridge replay matrix reported failed cases"' "$no_go_bundle"; then
  echo "expected replay drift reason marker in NO-GO localhost bridge demo evidence bundle" >&2
  exit 1
fi

no_go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$no_go_bundle")"
assert_eq "$(extract_value "$no_go_policy_output" "status")" "ok" "expected NO-GO localhost bridge demo policy check to pass"
assert_eq "$(extract_value "$no_go_policy_output" "final_decision")" "NO-GO" "expected NO-GO localhost bridge demo policy decision"

tampered_bundle="$TMP_DIR/localhost-bridge-demo-tampered.json"
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
  echo "expected tampered localhost bridge demo evidence bundle to fail policy validation" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_output" | grep -q "policy decision mismatch"; then
  echo "expected policy decision mismatch error for tampered localhost bridge demo bundle" >&2
  exit 1
fi

# Regression: #859
if ! printf '%s\n' "$tampered_output" | grep -q "expected final_decision=NO-GO"; then
  echo "expected explicit NO-GO decision mismatch in localhost bridge demo regression path" >&2
  exit 1
fi

echo "localhost bridge demo evidence bundle tests passed."
