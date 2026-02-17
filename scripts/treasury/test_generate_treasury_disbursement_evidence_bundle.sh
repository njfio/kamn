#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)/scripts/lib/common.sh"
GENERATOR="$KAMN_ROOT/scripts/treasury/generate_treasury_disbursement_evidence_bundle.sh"
POLICY_CHECKER="$KAMN_ROOT/scripts/treasury/check_treasury_disbursement_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$GENERATOR" ]; then
  echo "expected treasury disbursement evidence generator to be executable" >&2
  exit 1
fi

if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected treasury disbursement policy checker to be executable" >&2
  exit 1
fi

go_bundle="$TMP_DIR/treasury-go.json"
go_output="$(
  bash "$GENERATOR" \
    --output-file "$go_bundle" \
    --disbursement-id "disbursement-go-001" \
    --treasury-account-id "treasury-main-001" \
    --destination-account-id "ops-wallet-001" \
    --asset-symbol "KAMN" \
    --disbursement-amount 250000 \
    --daily-limit-amount 500000 \
    --required-approvals 2 \
    --received-approvals 2 \
    --approval-quorum-hash "sha256:approval-go-001" \
    --policy-window-open true \
    --ci-fast-gate PASS
)"

assert_eq "$(extract_value "$go_output" "status")" "generated" "expected GO treasury evidence bundle generation to succeed"
assert_eq "$(extract_value "$go_output" "final_decision")" "GO" "expected GO treasury evidence decision"

go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$go_bundle")"
assert_eq "$(extract_value "$go_policy_output" "status")" "ok" "expected GO treasury bundle policy check to pass"
assert_eq "$(extract_value "$go_policy_output" "final_decision")" "GO" "expected GO treasury policy decision"

no_go_bundle="$TMP_DIR/treasury-no-go.json"
no_go_output="$(
  bash "$GENERATOR" \
    --output-file "$no_go_bundle" \
    --disbursement-id "disbursement-no-go-001" \
    --treasury-account-id "treasury-main-001" \
    --destination-account-id "ops-wallet-001" \
    --asset-symbol "KAMN" \
    --disbursement-amount 750000 \
    --daily-limit-amount 500000 \
    --required-approvals 3 \
    --received-approvals 2 \
    --approval-quorum-hash "sha256:approval-no-go-001" \
    --policy-window-open false \
    --ci-fast-gate PASS
)"

assert_eq "$(extract_value "$no_go_output" "final_decision")" "NO-GO" "expected NO-GO treasury evidence decision for approval/limit/policy-window mismatches"

no_go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$no_go_bundle")"
assert_eq "$(extract_value "$no_go_policy_output" "status")" "ok" "expected NO-GO treasury bundle policy check to pass"
assert_eq "$(extract_value "$no_go_policy_output" "final_decision")" "NO-GO" "expected NO-GO treasury policy decision"

tampered_bundle="$TMP_DIR/treasury-tampered.json"
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
  echo "expected tampered treasury evidence bundle to fail policy validation" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_output" | grep -q "expected final_decision=NO-GO"; then
  echo "expected explicit final-decision mismatch error from treasury policy checker" >&2
  exit 1
fi

# Regression: #716
if ! printf '%s\n' "$tampered_output" | grep -q "policy decision mismatch"; then
  echo "expected regression guard to catch treasury policy decision mismatch" >&2
  exit 1
fi

echo "treasury disbursement evidence bundle tests passed."
