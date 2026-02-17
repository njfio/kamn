#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)/scripts/lib/common.sh"
GENERATOR="$KAMN_ROOT/scripts/channel/generate_channel_retention_redaction_evidence_bundle.sh"
POLICY_CHECKER="$KAMN_ROOT/scripts/channel/check_channel_retention_redaction_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$GENERATOR" ]; then
  echo "expected channel retention/redaction evidence generator to be executable" >&2
  exit 1
fi

if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected channel retention/redaction policy checker to be executable" >&2
  exit 1
fi

go_retention_report="$TMP_DIR/retention-go.json"
cat >"$go_retention_report" <<'JSON'
{
  "status": "pass",
  "total_candidates": 3,
  "replay_safe": true,
  "reason_codes": ["retention_window_valid", "candidate_order_stable"]
}
JSON

go_redaction_report="$TMP_DIR/redaction-go.json"
cat >"$go_redaction_report" <<'JSON'
{
  "status": "pass",
  "applied_count": 2,
  "replay_safe": true,
  "reason_codes": ["redaction_hash_stable", "redaction_replay_guard_active"]
}
JSON

go_bundle="$TMP_DIR/channel-retention-redaction-go.json"
go_output="$(
  bash "$GENERATOR" \
    --output-file "$go_bundle" \
    --lane contract \
    --retention-report-file "$go_retention_report" \
    --redaction-report-file "$go_redaction_report" \
    --ci-fast-gate PASS
)"

assert_eq "$(extract_value "$go_output" "status")" "generated" "expected GO channel evidence generation to succeed"
assert_eq "$(extract_value "$go_output" "final_decision")" "GO" "expected GO channel evidence decision"

go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$go_bundle")"
assert_eq "$(extract_value "$go_policy_output" "status")" "ok" "expected GO channel policy check to pass"
assert_eq "$(extract_value "$go_policy_output" "final_decision")" "GO" "expected GO channel policy decision"

no_go_retention_report="$TMP_DIR/retention-no-go.json"
cat >"$no_go_retention_report" <<'JSON'
{
  "status": "fail",
  "total_candidates": 3,
  "replay_safe": false,
  "reason_codes": ["retention_replay_risk_detected"]
}
JSON

no_go_redaction_report="$TMP_DIR/redaction-no-go.json"
cat >"$no_go_redaction_report" <<'JSON'
{
  "status": "pass",
  "applied_count": 2,
  "replay_safe": true,
  "reason_codes": ["redaction_hash_stable", "redaction_replay_guard_active"]
}
JSON

no_go_bundle="$TMP_DIR/channel-retention-redaction-no-go.json"
no_go_output="$(
  bash "$GENERATOR" \
    --output-file "$no_go_bundle" \
    --lane contract \
    --retention-report-file "$no_go_retention_report" \
    --redaction-report-file "$no_go_redaction_report" \
    --ci-fast-gate PASS
)"

assert_eq "$(extract_value "$no_go_output" "status")" "generated" "expected NO-GO channel evidence generation to succeed"
assert_eq "$(extract_value "$no_go_output" "final_decision")" "NO-GO" "expected NO-GO channel evidence decision"

no_go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$no_go_bundle")"
assert_eq "$(extract_value "$no_go_policy_output" "status")" "ok" "expected NO-GO channel policy check to pass"
assert_eq "$(extract_value "$no_go_policy_output" "final_decision")" "NO-GO" "expected NO-GO channel policy decision"

tampered_bundle="$TMP_DIR/channel-retention-redaction-tampered.json"
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
  echo "expected tampered channel retention/redaction evidence to fail policy validation" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_output" | grep -q "policy decision mismatch"; then
  echo "expected policy decision mismatch for tampered channel retention/redaction evidence" >&2
  exit 1
fi

# Regression: #930
if ! printf '%s\n' "$tampered_output" | grep -q "expected final_decision=NO-GO"; then
  echo "expected explicit NO-GO mismatch in replay-safe reason-code regression path" >&2
  exit 1
fi

echo "channel retention/redaction evidence bundle tests passed."
