#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)/scripts/lib/common.sh"
GENERATOR="$KAMN_ROOT/scripts/message/generate_processor_proof_artifact_evidence_bundle.sh"
POLICY_CHECKER="$KAMN_ROOT/scripts/message/check_processor_proof_artifact_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

fail() {
  printf '%s\n' "$1" >&2
  exit 1
}

if [ ! -x "$GENERATOR" ]; then
  fail "expected processor proof artifact evidence generator to be executable"
fi

if [ ! -x "$POLICY_CHECKER" ]; then
  fail "expected processor proof artifact policy checker to be executable"
fi

go_bundle="$TMP_DIR/processor-proof-artifact-go.json"
go_output="$(
  bash "$GENERATOR" \
    --output-file "$go_bundle" \
    --artifact-id "artifact-go-1" \
    --message-id "urn:uuid:msg-go-1" \
    --payload-commitment "fnv1a64:1111222233334444" \
    --proof-value "proof:ok:artifact-go-1" \
    --private-selector "task.description" \
    --ci-fast-gate PASS
)"
if ! printf '%s\n' "$go_output" | grep -q "^final_decision=GO$"; then
  fail "expected GO decision for valid processor proof artifact evidence bundle"
fi

if [ ! -f "$go_bundle" ]; then
  fail "expected GO processor proof artifact evidence bundle file to be created"
fi
if ! grep -q '"schema_version": "kamn.zk.processor-proof-artifact-evidence.v1"' "$go_bundle"; then
  fail "expected processor proof artifact evidence schema marker"
fi

go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$go_bundle")"
if ! printf '%s\n' "$go_policy_output" | grep -q "^final_decision=GO$"; then
  fail "expected GO policy decision for valid processor proof artifact evidence bundle"
fi
if ! printf '%s\n' "$go_policy_output" | grep -q "^failed_checks=none$"; then
  fail "expected no failed checks for valid processor proof artifact evidence bundle"
fi

no_go_bundle="$TMP_DIR/processor-proof-artifact-no-go.json"
no_go_output="$(
  bash "$GENERATOR" \
    --output-file "$no_go_bundle" \
    --artifact-id "artifact-no-go-1" \
    --message-id "urn:uuid:msg-no-go-1" \
    --payload-commitment "fnv1a64:5555666677778888" \
    --proof-value "proof:ok:artifact-no-go-1" \
    --private-selector "task..description" \
    --ci-fast-gate FAIL
)"
if ! printf '%s\n' "$no_go_output" | grep -q "^final_decision=NO-GO$"; then
  fail "expected NO-GO decision for invalid selector and failed ci fast gate"
fi
if ! printf '%s\n' "$no_go_output" | grep -q "^reason_key=zk_processor_proof_artifact_reason_codes:NO-GO:v1$"; then
  fail "expected NO-GO reason key marker for invalid selector path"
fi

no_go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$no_go_bundle")"
if ! printf '%s\n' "$no_go_policy_output" | grep -q "^final_decision=NO-GO$"; then
  fail "expected NO-GO policy decision for invalid selector/ci path"
fi
if ! printf '%s\n' "$no_go_policy_output" | grep -q "^failed_checks=ci_fast_gate_passed,private_selector_format_valid$"; then
  fail "expected ci_fast_gate_passed and private_selector_format_valid failed checks"
fi

tampered_bundle="$TMP_DIR/processor-proof-artifact-tampered.json"
cp "$go_bundle" "$tampered_bundle"
python3 - "$tampered_bundle" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text())
payload["final_decision"] = "NO-GO"
path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")
PY

set +e
tampered_output="$(bash "$POLICY_CHECKER" --bundle-file "$tampered_bundle" 2>&1)"
tampered_exit=$?
set -e
if [ "$tampered_exit" -eq 0 ]; then
  fail "expected tampered final_decision processor proof artifact bundle to fail validation"
fi
if ! printf '%s\n' "$tampered_output" | grep -q "policy decision mismatch"; then
  fail "expected explicit policy decision mismatch marker for tampered processor proof artifact bundle"
fi

echo "processor proof artifact evidence bundle tests passed."
