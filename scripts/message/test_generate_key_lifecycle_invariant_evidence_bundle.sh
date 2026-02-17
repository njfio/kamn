#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)/scripts/lib/common.sh"
GENERATOR="$KAMN_ROOT/scripts/message/generate_key_lifecycle_invariant_evidence_bundle.sh"
POLICY_CHECKER="$KAMN_ROOT/scripts/message/check_key_lifecycle_invariant_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$GENERATOR" ]; then
  echo "expected key lifecycle invariant evidence generator to be executable" >&2
  exit 1
fi

if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected key lifecycle invariant policy checker to be executable" >&2
  exit 1
fi

go_report="$TMP_DIR/key-lifecycle-go.json"
cat >"$go_report" <<'JSON'
{
  "status": "pass",
  "rotation_replay_detected": false,
  "stale_generation_detected": false,
  "revocation_bypass_detected": false,
  "reason_codes": ["rotation_sequence_monotonic", "revocation_state_consistent"]
}
JSON

go_bundle="$TMP_DIR/key-lifecycle-go-bundle.json"
go_output="$(
  bash "$GENERATOR" \
    --output-file "$go_bundle" \
    --lane contract \
    --report-file "$go_report" \
    --ci-fast-gate PASS
)"
assert_eq "$(extract_value "$go_output" "status")" "generated" "expected GO key lifecycle evidence generation to succeed"
assert_eq "$(extract_value "$go_output" "final_decision")" "GO" "expected GO key lifecycle evidence decision"

go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$go_bundle")"
assert_eq "$(extract_value "$go_policy_output" "status")" "ok" "expected GO key lifecycle policy check to pass"
assert_eq "$(extract_value "$go_policy_output" "final_decision")" "GO" "expected GO key lifecycle policy decision"

no_go_report="$TMP_DIR/key-lifecycle-no-go.json"
cat >"$no_go_report" <<'JSON'
{
  "status": "fail",
  "rotation_replay_detected": true,
  "stale_generation_detected": true,
  "revocation_bypass_detected": false,
  "reason_codes": ["rotation_replay_detected", "stale_generation_activation_detected"]
}
JSON

no_go_bundle="$TMP_DIR/key-lifecycle-no-go-bundle.json"
no_go_output="$(
  bash "$GENERATOR" \
    --output-file "$no_go_bundle" \
    --lane contract \
    --report-file "$no_go_report" \
    --ci-fast-gate PASS
)"
assert_eq "$(extract_value "$no_go_output" "status")" "generated" "expected NO-GO key lifecycle evidence generation to succeed"
assert_eq "$(extract_value "$no_go_output" "final_decision")" "NO-GO" "expected NO-GO key lifecycle evidence decision"

no_go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$no_go_bundle")"
assert_eq "$(extract_value "$no_go_policy_output" "status")" "ok" "expected NO-GO key lifecycle policy check to pass"
assert_eq "$(extract_value "$no_go_policy_output" "final_decision")" "NO-GO" "expected NO-GO key lifecycle policy decision"

tampered_bundle="$TMP_DIR/key-lifecycle-tampered-bundle.json"
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
  echo "expected tampered key lifecycle evidence to fail policy validation" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_output" | grep -q "policy decision mismatch"; then
  echo "expected policy decision mismatch for tampered key lifecycle evidence" >&2
  exit 1
fi

# Regression: #931
if ! printf '%s\n' "$tampered_output" | grep -q "expected final_decision=NO-GO"; then
  echo "expected explicit NO-GO mismatch in key lifecycle replay/stale regression path" >&2
  exit 1
fi

echo "key lifecycle invariant evidence bundle tests passed."
