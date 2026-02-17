#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)/scripts/lib/common.sh"
GENERATOR="$KAMN_ROOT/scripts/escrow/generate_settlement_reconciliation_evidence_bundle.sh"
POLICY_CHECKER="$KAMN_ROOT/scripts/escrow/check_settlement_reconciliation_evidence_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$GENERATOR" ]; then
  echo "expected settlement reconciliation evidence generator to be executable" >&2
  exit 1
fi

if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected settlement reconciliation evidence policy checker to be executable" >&2
  exit 1
fi

go_bundle="$TMP_DIR/settlement-go.json"
go_output="$(
  bash "$GENERATOR" \
    --output-file "$go_bundle" \
    --escrow-id "escrow-go-001" \
    --settlement-outcome RELEASED \
    --receipt-id "receipt-go-001" \
    --receipt-finality FINAL \
    --expected-release-amount 55 \
    --expected-refund-amount 0 \
    --observed-release-amount 55 \
    --observed-refund-amount 0 \
    --ledger-reference-id "ledger-entry-go-001" \
    --timeout-elapsed false \
    --ci-fast-gate PASS
)"

assert_eq "$(extract_value "$go_output" "status")" "generated" "expected GO settlement bundle generation to succeed"
assert_eq "$(extract_value "$go_output" "final_decision")" "GO" "expected GO settlement bundle decision"
assert_eq "$(extract_value "$go_output" "reason_key")" "settlement_reconciliation_reason_codes:GO:v1" "expected GO settlement bundle reason key"

go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$go_bundle")"
assert_eq "$(extract_value "$go_policy_output" "status")" "ok" "expected GO settlement bundle policy check to pass"
assert_eq "$(extract_value "$go_policy_output" "final_decision")" "GO" "expected GO settlement bundle policy decision"
assert_eq "$(extract_value "$go_policy_output" "reason_key")" "settlement_reconciliation_reason_codes:GO:v1" "expected GO settlement policy reason key"
if ! grep -q '"settlement_path": "PAYOUT"' "$go_bundle"; then
  echo "expected RELEASED settlement path to map to PAYOUT" >&2
  exit 1
fi
if ! grep -q '"settlement_path_payout"' "$go_bundle"; then
  echo "expected payout path reason code for RELEASED settlement" >&2
  exit 1
fi

refund_bundle="$TMP_DIR/settlement-refund-go.json"
refund_output="$(
  bash "$GENERATOR" \
    --output-file "$refund_bundle" \
    --escrow-id "escrow-refund-go-001" \
    --settlement-outcome REFUNDED \
    --receipt-id "receipt-refund-go-001" \
    --receipt-finality FINAL \
    --expected-release-amount 0 \
    --expected-refund-amount 42 \
    --observed-release-amount 0 \
    --observed-refund-amount 42 \
    --ledger-reference-id "ledger-entry-refund-go-001" \
    --timeout-elapsed false \
    --ci-fast-gate PASS
)"
assert_eq "$(extract_value "$refund_output" "final_decision")" "GO" "expected REFUNDED settlement bundle decision"
if ! grep -q '"settlement_path": "REFUND"' "$refund_bundle"; then
  echo "expected REFUNDED settlement path to map to REFUND" >&2
  exit 1
fi
if ! grep -q '"settlement_path_refund"' "$refund_bundle"; then
  echo "expected refund path reason code for REFUNDED settlement" >&2
  exit 1
fi

dispute_bundle="$TMP_DIR/settlement-dispute-go.json"
dispute_output="$(
  bash "$GENERATOR" \
    --output-file "$dispute_bundle" \
    --escrow-id "escrow-dispute-go-001" \
    --settlement-outcome DISPUTED_RESOLVED \
    --receipt-id "receipt-dispute-go-001" \
    --receipt-finality FINAL \
    --expected-release-amount 24 \
    --expected-refund-amount 18 \
    --observed-release-amount 24 \
    --observed-refund-amount 18 \
    --ledger-reference-id "ledger-entry-dispute-go-001" \
    --timeout-elapsed false \
    --ci-fast-gate PASS
)"
assert_eq "$(extract_value "$dispute_output" "final_decision")" "GO" "expected DISPUTED_RESOLVED settlement bundle decision"
if ! grep -q '"settlement_path": "DISPUTE"' "$dispute_bundle"; then
  echo "expected DISPUTED_RESOLVED settlement path to map to DISPUTE" >&2
  exit 1
fi
if ! grep -q '"settlement_path_dispute"' "$dispute_bundle"; then
  echo "expected dispute path reason code for DISPUTED_RESOLVED settlement" >&2
  exit 1
fi

no_go_bundle="$TMP_DIR/settlement-no-go.json"
no_go_output="$(
  bash "$GENERATOR" \
    --output-file "$no_go_bundle" \
    --escrow-id "escrow-no-go-001" \
    --settlement-outcome RELEASED \
    --receipt-id "receipt-no-go-001" \
    --receipt-finality FINAL \
    --expected-release-amount 55 \
    --expected-refund-amount 0 \
    --observed-release-amount 55 \
    --observed-refund-amount 0 \
    --ledger-reference-id "" \
    --timeout-elapsed false \
    --ci-fast-gate PASS
)"

assert_eq "$(extract_value "$no_go_output" "final_decision")" "NO-GO" "expected NO-GO settlement decision for missing ledger reference evidence"
assert_eq "$(extract_value "$no_go_output" "reason_key")" "settlement_reconciliation_reason_codes:NO-GO:v1" "expected NO-GO settlement reason key"

no_go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$no_go_bundle")"
assert_eq "$(extract_value "$no_go_policy_output" "status")" "ok" "expected NO-GO settlement bundle policy check to pass"
assert_eq "$(extract_value "$no_go_policy_output" "final_decision")" "NO-GO" "expected NO-GO settlement policy decision"
assert_eq "$(extract_value "$no_go_policy_output" "reason_key")" "settlement_reconciliation_reason_codes:NO-GO:v1" "expected NO-GO settlement policy reason key"
if ! printf '%s\n' "$no_go_policy_output" | grep -q "ledger_reference_missing"; then
  echo "expected policy output to include deterministic ledger_reference_missing reason code" >&2
  exit 1
fi

tampered_bundle="$TMP_DIR/settlement-tampered.json"
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
  echo "expected tampered settlement evidence bundle to fail policy validation" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_output" | grep -q "expected final_decision=NO-GO"; then
  echo "expected explicit final-decision mismatch error from settlement policy checker" >&2
  exit 1
fi

# Regression: #678
if ! printf '%s\n' "$tampered_output" | grep -q "policy decision mismatch"; then
  echo "expected regression guard to catch settlement policy decision mismatch" >&2
  exit 1
fi

tampered_reason_key_bundle="$TMP_DIR/settlement-tampered-reason-key.json"
cp "$go_bundle" "$tampered_reason_key_bundle"
python3 - "$tampered_reason_key_bundle" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text())
payload["reason_key"] = "settlement_reconciliation_reason_codes:NO-GO:v1"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n")
PY

set +e
tampered_reason_key_output="$(bash "$POLICY_CHECKER" --bundle-file "$tampered_reason_key_bundle" 2>&1)"
tampered_reason_key_code=$?
set -e

if [ "$tampered_reason_key_code" -eq 0 ]; then
  echo "expected reason-key tampered settlement evidence bundle to fail policy validation" >&2
  exit 1
fi

# Regression: #906
if ! printf '%s\n' "$tampered_reason_key_output" | grep -q "reason_key mismatch"; then
  echo "expected regression guard to catch settlement reason_key mismatch" >&2
  exit 1
fi

echo "settlement reconciliation evidence bundle tests passed."
