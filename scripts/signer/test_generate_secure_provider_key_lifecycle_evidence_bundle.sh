#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/signer/generate_secure_provider_key_lifecycle_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/signer/check_secure_provider_key_lifecycle_policy.sh"
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
  echo "expected secure-provider key-lifecycle evidence generator to be executable" >&2
  exit 1
fi

if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected secure-provider key-lifecycle policy checker to be executable" >&2
  exit 1
fi

go_bundle="$TMP_DIR/secure-provider-key-lifecycle-go.json"
go_output="$(
  bash "$GENERATOR" \
    --output-file "$go_bundle" \
    --secure-key-reference "secure:aws-kms:role-operator/key-ops-go-988" \
    --provider "aws-kms" \
    --key-role "operator" \
    --lifecycle-action "rotate" \
    --previous-version 4 \
    --target-version 5 \
    --incident-ticket "INC-1988" \
    --revocation-reason-code "operator-requested" \
    --required-approvals 2 \
    --received-approvals 2 \
    --custody-attestation-hash "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" \
    --approval-quorum-hash "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb" \
    --ci-fast-gate PASS
)"

assert_eq "$(extract_value "$go_output" "status")" "generated" "expected GO secure-provider key-lifecycle bundle generation"
assert_eq "$(extract_value "$go_output" "final_decision")" "GO" "expected GO secure-provider key-lifecycle final decision"
assert_eq "$(extract_value "$go_output" "reason_key")" "secure_provider_key_lifecycle_reason_codes:GO:v1" "expected GO secure-provider key-lifecycle reason key"
assert_eq "$(extract_value "$go_output" "failed_checks")" "none" "expected GO secure-provider key-lifecycle failed checks to be none"

go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$go_bundle")"
assert_eq "$(extract_value "$go_policy_output" "status")" "ok" "expected GO secure-provider key-lifecycle policy check status"
assert_eq "$(extract_value "$go_policy_output" "final_decision")" "GO" "expected GO secure-provider key-lifecycle policy decision"
assert_eq "$(extract_value "$go_policy_output" "failed_checks")" "none" "expected GO secure-provider key-lifecycle policy failed checks to be none"

no_go_bundle="$TMP_DIR/secure-provider-key-lifecycle-no-go.json"
no_go_output="$(
  bash "$GENERATOR" \
    --output-file "$no_go_bundle" \
    --secure-key-reference "secure:aws-kms:role-admin/key-admin-no-go-988" \
    --provider "aws-kms" \
    --key-role "admin" \
    --lifecycle-action "revoke" \
    --previous-version 7 \
    --target-version 8 \
    --incident-ticket "INC-2988" \
    --revocation-reason-code "policy-violation" \
    --required-approvals 3 \
    --received-approvals 1 \
    --custody-attestation-hash "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc" \
    --approval-quorum-hash "sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd" \
    --ci-fast-gate PASS
)"

assert_eq "$(extract_value "$no_go_output" "final_decision")" "NO-GO" "expected NO-GO secure-provider key-lifecycle final decision"

no_go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$no_go_bundle")"
assert_eq "$(extract_value "$no_go_policy_output" "status")" "ok" "expected NO-GO secure-provider key-lifecycle policy check status"
assert_eq "$(extract_value "$no_go_policy_output" "final_decision")" "NO-GO" "expected NO-GO secure-provider key-lifecycle policy decision"

tampered_bundle="$TMP_DIR/secure-provider-key-lifecycle-tampered-decision.json"
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
  echo "expected tampered secure-provider key-lifecycle bundle to fail policy validation" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_output" | grep -q "policy decision mismatch"; then
  echo "expected secure-provider key-lifecycle policy decision mismatch error for tampered bundle" >&2
  exit 1
fi

missing_key_bundle="$TMP_DIR/secure-provider-key-lifecycle-missing-policy.json"
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
  echo "expected missing-key secure-provider key-lifecycle bundle to fail policy validation" >&2
  exit 1
fi

if ! printf '%s\n' "$missing_key_output" | grep -q "missing bundle field: policy_checks"; then
  echo "expected missing policy_checks field failure for secure-provider key-lifecycle bundle" >&2
  exit 1
fi

# Regression: #988
if ! printf '%s\n' "$tampered_output" | grep -q "expected final_decision=NO-GO"; then
  echo "expected explicit NO-GO drift detection marker for secure-provider key-lifecycle regression" >&2
  exit 1
fi

echo "secure-provider key-lifecycle evidence bundle tests passed."
