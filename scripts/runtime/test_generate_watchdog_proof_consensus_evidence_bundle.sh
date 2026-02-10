#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/runtime/generate_watchdog_proof_consensus_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_watchdog_proof_consensus_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

fail() {
  printf '%s\n' "$1" >&2
  exit 1
}

if [ ! -x "$GENERATOR" ]; then
  fail "expected watchdog proof consensus evidence generator to be executable"
fi

if [ ! -x "$POLICY_CHECKER" ]; then
  fail "expected watchdog proof consensus policy checker to be executable"
fi

go_bundle="$TMP_DIR/watchdog-proof-consensus-go.json"
go_output="$(
  bash "$GENERATOR" \
    --output-file "$go_bundle" \
    --message-id "urn:uuid:watchdog-go-996" \
    --artifact-id "artifact-watchdog-go-996" \
    --consensus-status ConsensusValid \
    --required-quorum 2 \
    --valid-attestation-count 2 \
    --invalid-attestation-count 0 \
    --replay-attestation-count 0 \
    --cadence fast \
    --runtime-seconds 4 \
    --max-seconds 90 \
    --evidence-complete true \
    --ci-fast-gate PASS
)"
if ! printf '%s\n' "$go_output" | grep -q "^final_decision=GO$"; then
  fail "expected GO decision for valid watchdog proof consensus evidence bundle"
fi
if [ ! -f "$go_bundle" ]; then
  fail "expected GO watchdog proof consensus evidence bundle file to be created"
fi
if ! grep -q '"schema_version": "kamn.runtime.watchdog-proof-consensus-report.v1"' "$go_bundle"; then
  fail "expected watchdog proof consensus evidence schema marker"
fi

go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$go_bundle")"
if ! printf '%s\n' "$go_policy_output" | grep -q "^final_decision=GO$"; then
  fail "expected GO policy decision for valid watchdog proof consensus evidence bundle"
fi

no_go_bundle="$TMP_DIR/watchdog-proof-consensus-no-go.json"
no_go_output="$(
  bash "$GENERATOR" \
    --output-file "$no_go_bundle" \
    --message-id "urn:uuid:watchdog-no-go-996" \
    --artifact-id "artifact-watchdog-no-go-996" \
    --consensus-status ConsensusReplay \
    --required-quorum 2 \
    --valid-attestation-count 1 \
    --invalid-attestation-count 0 \
    --replay-attestation-count 1 \
    --cadence scheduled \
    --runtime-seconds 6 \
    --max-seconds 90 \
    --evidence-complete true \
    --ci-fast-gate PASS
)"
if ! printf '%s\n' "$no_go_output" | grep -q "^final_decision=NO-GO$"; then
  fail "expected NO-GO decision for replay watchdog proof consensus evidence bundle"
fi

no_go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$no_go_bundle")"
if ! printf '%s\n' "$no_go_policy_output" | grep -q "^final_decision=NO-GO$"; then
  fail "expected NO-GO policy decision for replay watchdog proof consensus evidence bundle"
fi

tampered_bundle="$TMP_DIR/watchdog-proof-consensus-tampered.json"
cp "$go_bundle" "$tampered_bundle"
python3 - "$tampered_bundle" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text())
payload["final_decision"] = "NO-GO"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_output="$(bash "$POLICY_CHECKER" --bundle-file "$tampered_bundle" 2>&1)"
tampered_exit=$?
set -e
if [ "$tampered_exit" -eq 0 ]; then
  fail "expected tampered watchdog proof consensus evidence bundle to fail validation"
fi
if ! printf '%s\n' "$tampered_output" | grep -q "policy decision mismatch"; then
  fail "expected explicit policy decision mismatch marker for tampered watchdog proof consensus bundle"
fi

echo "watchdog proof consensus evidence bundle tests passed."
