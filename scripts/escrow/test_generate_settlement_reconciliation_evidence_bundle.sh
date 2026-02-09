#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/escrow/generate_settlement_reconciliation_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/escrow/check_settlement_reconciliation_evidence_policy.sh"
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
    --timeout-elapsed false \
    --ci-fast-gate PASS
)"

assert_eq "$(extract_value "$go_output" "status")" "generated" "expected GO settlement bundle generation to succeed"
assert_eq "$(extract_value "$go_output" "final_decision")" "GO" "expected GO settlement bundle decision"

go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$go_bundle")"
assert_eq "$(extract_value "$go_policy_output" "status")" "ok" "expected GO settlement bundle policy check to pass"
assert_eq "$(extract_value "$go_policy_output" "final_decision")" "GO" "expected GO settlement bundle policy decision"

no_go_bundle="$TMP_DIR/settlement-no-go.json"
no_go_output="$(
  bash "$GENERATOR" \
    --output-file "$no_go_bundle" \
    --escrow-id "escrow-no-go-001" \
    --settlement-outcome TIMEOUT_REFUNDED \
    --receipt-id "receipt-no-go-001" \
    --receipt-finality PENDING \
    --expected-release-amount 0 \
    --expected-refund-amount 55 \
    --observed-release-amount 0 \
    --observed-refund-amount 55 \
    --timeout-elapsed false \
    --ci-fast-gate PASS
)"

assert_eq "$(extract_value "$no_go_output" "final_decision")" "NO-GO" "expected NO-GO settlement decision for timeout/finality mismatch"

no_go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$no_go_bundle")"
assert_eq "$(extract_value "$no_go_policy_output" "status")" "ok" "expected NO-GO settlement bundle policy check to pass"
assert_eq "$(extract_value "$no_go_policy_output" "final_decision")" "NO-GO" "expected NO-GO settlement policy decision"

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

echo "settlement reconciliation evidence bundle tests passed."
