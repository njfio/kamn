#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)/scripts/lib/common.sh"
GENERATOR="$KAMN_ROOT/scripts/sdk/generate_live_transport_replay_tamper_evidence_bundle.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$GENERATOR" ]; then
  echo "expected live transport replay/tamper evidence generator to be executable" >&2
  exit 1
fi

go_bundle="$TMP_DIR/live-transport-replay-tamper-go.json"
go_output="$(
  bash "$GENERATOR" \
    --output-file "$go_bundle" \
    --transport-lane-id "localhost-signed-integration" \
    --message-id "msg-go-001" \
    --from-did "kamn:did:agent:sender-1" \
    --to-did "kamn:did:agent:listener-1" \
    --nonce 41 \
    --signature-status valid \
    --replay-detected false \
    --tamper-detected false \
    --ci-fast-gate PASS
)"

if [ "$(extract_value "$go_output" "status")" != "generated" ]; then
  echo "expected GO replay/tamper evidence bundle generation to succeed" >&2
  exit 1
fi
if [ "$(extract_value "$go_output" "final_decision")" != "GO" ]; then
  echo "expected GO replay/tamper final decision for clean input" >&2
  exit 1
fi

if ! grep -q '"schema_version": "kamn.sdk.live-transport-replay-tamper-evidence.v1"' "$go_bundle"; then
  echo "expected replay/tamper evidence bundle schema marker" >&2
  exit 1
fi

no_go_bundle="$TMP_DIR/live-transport-replay-tamper-no-go.json"
no_go_output="$(
  bash "$GENERATOR" \
    --output-file "$no_go_bundle" \
    --transport-lane-id "localhost-signed-integration" \
    --message-id "msg-no-go-001" \
    --from-did "kamn:did:agent:sender-1" \
    --to-did "kamn:did:agent:listener-1" \
    --nonce 41 \
    --signature-status malformed \
    --replay-detected true \
    --tamper-detected true \
    --ci-fast-gate PASS
)"

if [ "$(extract_value "$no_go_output" "final_decision")" != "NO-GO" ]; then
  echo "expected NO-GO replay/tamper final decision for malformed+replay+tamper input" >&2
  exit 1
fi

if ! grep -q '"malformed_signature_detected"' "$no_go_bundle"; then
  echo "expected malformed signature reason marker in replay/tamper NO-GO bundle" >&2
  exit 1
fi
if ! grep -q '"replay_nonce_detected"' "$no_go_bundle"; then
  echo "expected replay nonce reason marker in replay/tamper NO-GO bundle" >&2
  exit 1
fi
if ! grep -q '"tamper_payload_detected"' "$no_go_bundle"; then
  echo "expected tamper reason marker in replay/tamper NO-GO bundle" >&2
  exit 1
fi

echo "live transport replay/tamper evidence bundle generator tests passed."
