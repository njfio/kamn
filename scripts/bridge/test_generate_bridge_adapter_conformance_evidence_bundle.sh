#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/bridge/generate_bridge_adapter_conformance_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/bridge/check_bridge_adapter_conformance_policy.sh"
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
  echo "expected bridge adapter conformance evidence generator to be executable" >&2
  exit 1
fi

if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected bridge adapter conformance policy checker to be executable" >&2
  exit 1
fi

go_bundle="$TMP_DIR/bridge-adapter-conformance-go.json"
go_output="$(
  bash "$GENERATOR" \
    --output-file "$go_bundle" \
    --adapter-id "adapter-discord-settlement-v1" \
    --bridge-network ethereum \
    --dry-run true \
    --request-expected-schema-version "kamn.bridge.settlement-request.v1" \
    --request-observed-schema-version "kamn.bridge.settlement-request.v1" \
    --request-required-fields "request_id,destination_channel,payload_hash" \
    --request-observed-fields "destination_channel,payload_hash,request_id" \
    --receipt-expected-schema-version "kamn.bridge.settlement-receipt.v1" \
    --receipt-observed-schema-version "kamn.bridge.settlement-receipt.v1" \
    --receipt-required-fields "receipt_id,settlement_ref,finality" \
    --receipt-observed-fields "finality,receipt_id,settlement_ref" \
    --ci-fast-gate PASS
)"

assert_eq "$(extract_value "$go_output" "status")" "generated" "expected GO bundle generation to pass"
assert_eq "$(extract_value "$go_output" "final_decision")" "GO" "expected GO policy decision"
assert_eq "$(extract_value "$go_output" "reason_key")" "bridge_adapter_conformance_reason_codes:GO:v1" "expected GO reason key marker"

go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$go_bundle")"
assert_eq "$(extract_value "$go_policy_output" "status")" "ok" "expected GO bundle policy check to pass"
assert_eq "$(extract_value "$go_policy_output" "final_decision")" "GO" "expected GO policy checker decision"
assert_eq "$(extract_value "$go_policy_output" "reason_key")" "bridge_adapter_conformance_reason_codes:GO:v1" "expected GO policy checker reason key marker"

no_go_bundle="$TMP_DIR/bridge-adapter-conformance-no-go.json"
no_go_output="$(
  bash "$GENERATOR" \
    --output-file "$no_go_bundle" \
    --adapter-id "adapter-discord-settlement-v1" \
    --bridge-network ethereum \
    --dry-run true \
    --request-expected-schema-version "kamn.bridge.settlement-request.v1" \
    --request-observed-schema-version "kamn.bridge.settlement-request.v2" \
    --request-required-fields "request_id,destination_channel,payload_hash" \
    --request-observed-fields "destination_channel,request_id" \
    --receipt-expected-schema-version "kamn.bridge.settlement-receipt.v1" \
    --receipt-observed-schema-version "kamn.bridge.settlement-receipt.v1" \
    --receipt-required-fields "receipt_id,settlement_ref,finality" \
    --receipt-observed-fields "receipt_id,settlement_ref" \
    --ci-fast-gate FAIL
)"

assert_eq "$(extract_value "$no_go_output" "status")" "generated" "expected NO-GO bundle generation to pass"
assert_eq "$(extract_value "$no_go_output" "final_decision")" "NO-GO" "expected NO-GO policy decision"
assert_eq "$(extract_value "$no_go_output" "reason_key")" "bridge_adapter_conformance_reason_codes:NO-GO:v1" "expected NO-GO reason key marker"

no_go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$no_go_bundle")"
assert_eq "$(extract_value "$no_go_policy_output" "status")" "ok" "expected NO-GO bundle policy check to pass"
assert_eq "$(extract_value "$no_go_policy_output" "final_decision")" "NO-GO" "expected NO-GO policy checker decision"
assert_eq "$(extract_value "$no_go_policy_output" "reason_key")" "bridge_adapter_conformance_reason_codes:NO-GO:v1" "expected NO-GO policy checker reason key marker"

tampered_bundle="$TMP_DIR/bridge-adapter-conformance-tampered.json"
cp "$no_go_bundle" "$tampered_bundle"
python3 - "$tampered_bundle" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text())
payload["final_decision"] = "GO"
payload["reason_key"] = "bridge_adapter_conformance_reason_codes:GO:v1"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n")
PY

set +e
tampered_output="$(bash "$POLICY_CHECKER" --bundle-file "$tampered_bundle" 2>&1)"
tampered_code=$?
set -e

if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered bridge adapter conformance bundle to fail policy validation" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_output" | grep -q "policy decision mismatch"; then
  echo "expected policy decision mismatch error for tampered bridge adapter conformance bundle" >&2
  exit 1
fi

# Regression: #907
if ! printf '%s\n' "$tampered_output" | grep -q "expected final_decision=NO-GO"; then
  echo "expected explicit NO-GO mismatch marker for bridge adapter conformance tamper regression" >&2
  exit 1
fi

echo "bridge adapter conformance evidence bundle tests passed."
