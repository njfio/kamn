#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/did/generate_lifecycle_operator_binding_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/did/check_lifecycle_operator_binding_policy.sh"
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
  echo "expected lifecycle operator-binding evidence generator to be executable" >&2
  exit 1
fi

if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected lifecycle operator-binding policy checker to be executable" >&2
  exit 1
fi

go_bundle="$TMP_DIR/lifecycle-operator-binding-go.json"
go_output="$(
  bash "$GENERATOR" \
    --output-file "$go_bundle" \
    --did "kamn:did:agent:agent-go-890" \
    --actor-did "kamn:did:human:operator-go-890" \
    --required-operator-did "kamn:did:human:operator-go-890" \
    --mutation-action "rotate" \
    --mutation-nonce 41 \
    --mutation-reason-code "did_lifecycle_mutation_allowed" \
    --audit-export-id "audit-export-go-890" \
    --audit-record-count 3 \
    --audit-digest "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" \
    --ci-fast-gate PASS
)"

assert_eq "$(extract_value "$go_output" "status")" "generated" "expected GO lifecycle operator-binding bundle generation"
assert_eq "$(extract_value "$go_output" "final_decision")" "GO" "expected GO lifecycle operator-binding final decision"
assert_eq "$(extract_value "$go_output" "reason_key")" "did_lifecycle_operator_binding_reason_codes:GO:v1" "expected GO lifecycle operator-binding reason key"

go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$go_bundle")"
assert_eq "$(extract_value "$go_policy_output" "status")" "ok" "expected GO lifecycle operator-binding policy check status"
assert_eq "$(extract_value "$go_policy_output" "final_decision")" "GO" "expected GO lifecycle operator-binding policy decision"

no_go_bundle="$TMP_DIR/lifecycle-operator-binding-no-go.json"
no_go_output="$(
  bash "$GENERATOR" \
    --output-file "$no_go_bundle" \
    --did "kamn:did:agent:agent-no-go-890" \
    --actor-did "kamn:did:human:operator-unbound-890" \
    --required-operator-did "kamn:did:human:operator-required-890" \
    --mutation-action "revoke" \
    --mutation-nonce 77 \
    --mutation-reason-code "did_lifecycle_mutation_unauthorized_actor" \
    --audit-export-id "audit-export-no-go-890" \
    --audit-record-count 1 \
    --audit-digest "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb" \
    --ci-fast-gate PASS
)"

assert_eq "$(extract_value "$no_go_output" "final_decision")" "NO-GO" "expected NO-GO lifecycle operator-binding final decision"

no_go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$no_go_bundle")"
assert_eq "$(extract_value "$no_go_policy_output" "status")" "ok" "expected NO-GO lifecycle operator-binding policy check status"
assert_eq "$(extract_value "$no_go_policy_output" "final_decision")" "NO-GO" "expected NO-GO lifecycle operator-binding policy decision"

tampered_bundle="$TMP_DIR/lifecycle-operator-binding-tampered-decision.json"
cp "$no_go_bundle" "$tampered_bundle"
python3 - "$tampered_bundle" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["final_decision"] = "GO"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_output="$(bash "$POLICY_CHECKER" --bundle-file "$tampered_bundle" 2>&1)"
tampered_code=$?
set -e

if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered lifecycle operator-binding bundle to fail policy validation" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_output" | grep -q "policy decision mismatch"; then
  echo "expected lifecycle operator-binding policy decision mismatch error for tampered bundle" >&2
  exit 1
fi

missing_key_bundle="$TMP_DIR/lifecycle-operator-binding-missing-key.json"
cp "$go_bundle" "$missing_key_bundle"
python3 - "$missing_key_bundle" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
del payload["policy_checks"]
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
missing_key_output="$(bash "$POLICY_CHECKER" --bundle-file "$missing_key_bundle" 2>&1)"
missing_key_code=$?
set -e

if [ "$missing_key_code" -eq 0 ]; then
  echo "expected missing-key lifecycle operator-binding bundle to fail policy validation" >&2
  exit 1
fi

if ! printf '%s\n' "$missing_key_output" | grep -q "missing bundle field: policy_checks"; then
  echo "expected missing policy_checks field failure for lifecycle operator-binding bundle" >&2
  exit 1
fi

# Regression: #890
if ! printf '%s\n' "$tampered_output" | grep -q "expected final_decision=NO-GO"; then
  echo "expected explicit NO-GO drift detection marker for lifecycle operator-binding regression" >&2
  exit 1
fi

echo "lifecycle operator-binding evidence bundle tests passed."
