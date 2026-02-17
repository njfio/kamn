#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)/scripts/lib/common.sh"
GENERATOR="$KAMN_ROOT/scripts/did/generate_federated_did_handshake_evidence_bundle.sh"
POLICY_CHECKER="$KAMN_ROOT/scripts/did/check_federated_did_handshake_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$GENERATOR" ]; then
  echo "expected federated DID handshake evidence generator to be executable" >&2
  exit 1
fi

if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected federated DID handshake policy checker to be executable" >&2
  exit 1
fi

go_bundle="$TMP_DIR/federated-handshake-go.json"
go_output="$(
  bash "$GENERATOR" \
    --output-file "$go_bundle" \
    --handshake-id "federated-go-001" \
    --subject-did "kamn:did:agent:federated-agent-go" \
    --local-network "kolme-mainnet-a" \
    --remote-network "kolme-mainnet-b" \
    --resolver-cache-hit true \
    --resolver-version "resolver-v1" \
    --signature-policy PASS \
    --nonce-monotonic true \
    --downgrade-detected false \
    --partition-sequence-monotonic true \
    --required-quorum 2 \
    --received-quorum 2 \
    --ci-fast-gate PASS
)"

assert_eq "$(extract_value "$go_output" "status")" "generated" "expected GO handshake bundle generation to pass"
assert_eq "$(extract_value "$go_output" "final_decision")" "GO" "expected GO handshake policy decision"

go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$go_bundle")"
assert_eq "$(extract_value "$go_policy_output" "status")" "ok" "expected GO handshake policy check to pass"
assert_eq "$(extract_value "$go_policy_output" "final_decision")" "GO" "expected GO handshake checker decision"

no_go_bundle="$TMP_DIR/federated-handshake-no-go.json"
no_go_output="$(
  bash "$GENERATOR" \
    --output-file "$no_go_bundle" \
    --handshake-id "federated-no-go-001" \
    --subject-did "kamn:did:agent:federated-agent-no-go" \
    --local-network "kolme-mainnet-a" \
    --remote-network "kolme-mainnet-b" \
    --resolver-cache-hit false \
    --resolver-version "resolver-v1" \
    --signature-policy PASS \
    --nonce-monotonic false \
    --downgrade-detected true \
    --partition-sequence-monotonic false \
    --required-quorum 2 \
    --received-quorum 1 \
    --ci-fast-gate PASS
)"

assert_eq "$(extract_value "$no_go_output" "final_decision")" "NO-GO" "expected NO-GO handshake policy decision"

no_go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$no_go_bundle")"
assert_eq "$(extract_value "$no_go_policy_output" "status")" "ok" "expected NO-GO handshake policy check to pass"
assert_eq "$(extract_value "$no_go_policy_output" "final_decision")" "NO-GO" "expected NO-GO checker decision"

tampered_bundle="$TMP_DIR/federated-handshake-tampered.json"
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
  echo "expected tampered federated DID handshake bundle to fail policy validation" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_output" | grep -q "policy decision mismatch"; then
  echo "expected policy decision mismatch error for tampered federated DID handshake bundle" >&2
  exit 1
fi

# Regression: #734
if ! printf '%s\n' "$tampered_output" | grep -q "expected final_decision=NO-GO"; then
  echo "expected explicit NO-GO decision mismatch in tampered regression path" >&2
  exit 1
fi

echo "federated DID handshake evidence bundle tests passed."
