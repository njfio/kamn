#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/did/generate_service_endpoint_canonicalization_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/did/check_service_endpoint_canonicalization_policy.sh"
FIXTURE="$ROOT_DIR/fixtures/did_core_conformance/service_endpoint_canonicalization_vectors.json"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

fail() {
  printf '%s\n' "$1" >&2
  exit 1
}

if [ ! -x "$GENERATOR" ]; then
  fail "expected service endpoint canonicalization evidence generator to be executable"
fi
if [ ! -x "$POLICY_CHECKER" ]; then
  fail "expected service endpoint canonicalization policy checker to be executable"
fi
if [ ! -f "$FIXTURE" ]; then
  fail "expected service endpoint canonicalization fixture to exist"
fi

go_bundle="$TMP_DIR/did-service-endpoint-canonicalization-go.json"
go_output="$(
  bash "$GENERATOR" \
    --output-file "$go_bundle" \
    --fixture "$FIXTURE" \
    --ci-fast-gate PASS
)"
if ! printf '%s\n' "$go_output" | grep -q "^final_decision=GO$"; then
  fail "expected GO decision for canonical service endpoint conformance fixture"
fi
if [ ! -f "$go_bundle" ]; then
  fail "expected GO canonicalization evidence bundle file to be created"
fi
if ! grep -q '"schema_version": "kamn.did.service-endpoint-canonicalization-report.v1"' "$go_bundle"; then
  fail "expected canonicalization evidence schema marker"
fi

go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$go_bundle")"
if ! printf '%s\n' "$go_policy_output" | grep -q "^final_decision=GO$"; then
  fail "expected GO policy decision for canonical service endpoint conformance fixture"
fi

no_go_fixture="$TMP_DIR/nonconformant-vectors.json"
python3 - "$FIXTURE" "$no_go_fixture" <<'PY'
import json
import pathlib
import sys

source = pathlib.Path(sys.argv[1])
dest = pathlib.Path(sys.argv[2])
vectors = json.loads(source.read_text(encoding="utf-8"))
vectors[0]["expected_endpoint"] = "kamn://messaging/unexpected"
dest.write_text(json.dumps(vectors, indent=2) + "\n", encoding="utf-8")
PY

no_go_bundle="$TMP_DIR/did-service-endpoint-canonicalization-no-go.json"
no_go_output="$(
  bash "$GENERATOR" \
    --output-file "$no_go_bundle" \
    --fixture "$no_go_fixture" \
    --ci-fast-gate PASS
)"
if ! printf '%s\n' "$no_go_output" | grep -q "^final_decision=NO-GO$"; then
  fail "expected NO-GO decision for drifted canonicalization conformance fixture"
fi

no_go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$no_go_bundle")"
if ! printf '%s\n' "$no_go_policy_output" | grep -q "^final_decision=NO-GO$"; then
  fail "expected NO-GO policy decision for drifted canonicalization conformance fixture"
fi

tampered_bundle="$TMP_DIR/did-service-endpoint-canonicalization-tampered.json"
cp "$go_bundle" "$tampered_bundle"
python3 - "$tampered_bundle" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["final_decision"] = "NO-GO"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_output="$(bash "$POLICY_CHECKER" --bundle-file "$tampered_bundle" 2>&1)"
tampered_exit=$?
set -e
if [ "$tampered_exit" -eq 0 ]; then
  fail "expected tampered canonicalization evidence bundle to fail validation"
fi
if ! printf '%s\n' "$tampered_output" | grep -q "policy decision mismatch"; then
  fail "expected explicit policy decision mismatch marker for tampered canonicalization bundle"
fi

echo "service endpoint canonicalization evidence bundle tests passed."
