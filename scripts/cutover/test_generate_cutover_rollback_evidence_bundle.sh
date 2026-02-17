#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)/scripts/lib/common.sh"
GENERATOR="$KAMN_ROOT/scripts/cutover/generate_cutover_rollback_evidence_bundle.sh"
POLICY_CHECKER="$KAMN_ROOT/scripts/cutover/check_cutover_rollback_evidence_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$GENERATOR" ]; then
  echo "expected cutover rollback evidence bundle generator to be executable" >&2
  exit 1
fi

if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected cutover rollback evidence policy checker to be executable" >&2
  exit 1
fi

go_bundle="$TMP_DIR/rollback-go.json"
go_generate_output="$(
  bash "$GENERATOR" \
    --output-file "$go_bundle" \
    --cutover-manifest-id "cutover-mainnet-2026-02-09" \
    --rollback-trigger-status CLEAR \
    --checkpoint-state READY \
    --failed-checkpoint-id "" \
    --rollback-target-hash "state-hash-abc" \
    --post-rollback-hash "state-hash-abc" \
    --evidence-complete true \
    --ci-fast-gate PASS
)"

assert_eq "$(extract_value "$go_generate_output" "status")" "generated" "expected GO rollback bundle generation to succeed"
assert_eq "$(extract_value "$go_generate_output" "final_decision")" "GO" "expected generator to derive GO rollback decision"

go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$go_bundle")"
assert_eq "$(extract_value "$go_policy_output" "status")" "ok" "expected GO rollback bundle policy check to pass"
assert_eq "$(extract_value "$go_policy_output" "final_decision")" "GO" "expected rollback policy check to keep GO decision"

no_go_bundle="$TMP_DIR/rollback-no-go.json"
no_go_generate_output="$(
  bash "$GENERATOR" \
    --output-file "$no_go_bundle" \
    --cutover-manifest-id "cutover-mainnet-2026-02-09" \
    --rollback-trigger-status TRIGGERED \
    --checkpoint-state FAILED \
    --failed-checkpoint-id "" \
    --rollback-target-hash "state-hash-abc" \
    --post-rollback-hash "state-hash-def" \
    --evidence-complete true \
    --ci-fast-gate PASS
)"

assert_eq "$(extract_value "$no_go_generate_output" "final_decision")" "NO-GO" "expected invalid rollback evidence to force NO-GO"

no_go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$no_go_bundle")"
assert_eq "$(extract_value "$no_go_policy_output" "status")" "ok" "expected NO-GO rollback bundle policy check to pass"
assert_eq "$(extract_value "$no_go_policy_output" "final_decision")" "NO-GO" "expected rollback policy check to keep NO-GO decision"

tampered_bundle="$TMP_DIR/rollback-tampered.json"
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
  echo "expected tampered rollback decision bundle to fail policy validation" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_output" | grep -q "expected final_decision=NO-GO"; then
  echo "expected explicit final-decision mismatch error from rollback policy checker" >&2
  exit 1
fi

# Regression: #708
if ! printf '%s\n' "$tampered_output" | grep -q "missing failed checkpoint evidence"; then
  echo "expected missing failed checkpoint regression guard to be enforced" >&2
  exit 1
fi

echo "cutover rollback evidence bundle tests passed."
