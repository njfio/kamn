#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/runtime/generate_processor_proof_admission_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_processor_proof_admission_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

fail() {
  printf '%s\n' "$1" >&2
  exit 1
}

if [ ! -x "$GENERATOR" ]; then
  fail "expected processor proof admission evidence generator to be executable"
fi

if [ ! -x "$POLICY_CHECKER" ]; then
  fail "expected processor proof admission policy checker to be executable"
fi

go_bundle="$TMP_DIR/processor-proof-admission-go.json"
go_output="$(
  bash "$GENERATOR" \
    --output-file "$go_bundle" \
    --artifact-id "artifact-go-995" \
    --message-id "urn:uuid:go-995" \
    --message-id-match true \
    --commitment-match true \
    --proof-format-valid true \
    --replay-guard-active true \
    --ci-fast-gate PASS
)"
if ! printf '%s\n' "$go_output" | grep -q "^final_decision=GO$"; then
  fail "expected GO decision for valid processor proof admission evidence bundle"
fi
if [ ! -f "$go_bundle" ]; then
  fail "expected GO processor proof admission evidence bundle file to be created"
fi
if ! grep -q '"schema_version": "kamn.runtime.processor-proof-admission-report.v1"' "$go_bundle"; then
  fail "expected processor proof admission evidence schema marker"
fi

go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$go_bundle")"
if ! printf '%s\n' "$go_policy_output" | grep -q "^final_decision=GO$"; then
  fail "expected GO policy decision for valid processor proof admission evidence bundle"
fi

no_go_bundle="$TMP_DIR/processor-proof-admission-no-go.json"
no_go_output="$(
  bash "$GENERATOR" \
    --output-file "$no_go_bundle" \
    --artifact-id "artifact-no-go-995" \
    --message-id "urn:uuid:no-go-995" \
    --message-id-match false \
    --commitment-match false \
    --proof-format-valid false \
    --replay-guard-active false \
    --ci-fast-gate FAIL
)"
if ! printf '%s\n' "$no_go_output" | grep -q "^final_decision=NO-GO$"; then
  fail "expected NO-GO decision for invalid processor proof admission evidence bundle"
fi

no_go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$no_go_bundle")"
if ! printf '%s\n' "$no_go_policy_output" | grep -q "^final_decision=NO-GO$"; then
  fail "expected NO-GO policy decision for invalid processor proof admission evidence bundle"
fi

tampered_bundle="$TMP_DIR/processor-proof-admission-tampered.json"
cp "$go_bundle" "$tampered_bundle"
python3 - "$tampered_bundle" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text())
payload["final_decision"] = "NO-GO"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n")
PY

set +e
tampered_output="$(bash "$POLICY_CHECKER" --bundle-file "$tampered_bundle" 2>&1)"
tampered_exit=$?
set -e
if [ "$tampered_exit" -eq 0 ]; then
  fail "expected tampered processor proof admission evidence bundle to fail validation"
fi
if ! printf '%s\n' "$tampered_output" | grep -q "policy decision mismatch"; then
  fail "expected explicit policy decision mismatch marker for tampered processor proof admission bundle"
fi

echo "processor proof admission evidence bundle tests passed."
