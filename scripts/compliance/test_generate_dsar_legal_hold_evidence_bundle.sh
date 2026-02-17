#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)/scripts/lib/common.sh"
GENERATOR="$KAMN_ROOT/scripts/compliance/generate_dsar_legal_hold_evidence_bundle.sh"
POLICY_CHECKER="$KAMN_ROOT/scripts/compliance/check_dsar_legal_hold_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$GENERATOR" ]; then
  echo "expected DSAR legal-hold evidence generator to be executable" >&2
  exit 1
fi

if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected DSAR legal-hold policy checker to be executable" >&2
  exit 1
fi

go_bundle="$TMP_DIR/dsar-go.json"
go_output="$(
  bash "$GENERATOR" \
    --output-file "$go_bundle" \
    --request-id "dsar-export-go-001" \
    --subject-did "did:kamn:subject-go" \
    --request-type EXPORT \
    --legal-hold-active false \
    --retention-expired true \
    --evidence-complete true \
    --approval-recorded true \
    --tamper-check PASS \
    --ci-fast-gate PASS
)"

assert_eq "$(extract_value "$go_output" "status")" "generated" "expected GO bundle generation to pass"
assert_eq "$(extract_value "$go_output" "final_decision")" "GO" "expected GO policy decision"

go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$go_bundle")"
assert_eq "$(extract_value "$go_policy_output" "status")" "ok" "expected GO bundle policy check to pass"
assert_eq "$(extract_value "$go_policy_output" "final_decision")" "GO" "expected GO policy checker decision"
assert_eq "$(extract_value "$go_policy_output" "failed_checks")" "none" "expected no failed checks for GO"

no_go_bundle="$TMP_DIR/dsar-no-go.json"
no_go_output="$(
  bash "$GENERATOR" \
    --output-file "$no_go_bundle" \
    --request-id "dsar-erasure-no-go-001" \
    --subject-did "did:kamn:subject-no-go" \
    --request-type ERASURE \
    --legal-hold-active true \
    --retention-expired true \
    --evidence-complete true \
    --approval-recorded true \
    --tamper-check PASS \
    --ci-fast-gate PASS
)"

assert_eq "$(extract_value "$no_go_output" "final_decision")" "NO-GO" "expected NO-GO policy decision"

no_go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$no_go_bundle")"
assert_eq "$(extract_value "$no_go_policy_output" "status")" "ok" "expected NO-GO bundle policy check to pass"
assert_eq "$(extract_value "$no_go_policy_output" "final_decision")" "NO-GO" "expected NO-GO checker decision"
assert_eq "$(extract_value "$no_go_policy_output" "failed_checks")" "legal_hold_precedence_block" "expected legal hold precedence failure reason"

tampered_bundle="$TMP_DIR/dsar-tampered.json"
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
  echo "expected tampered DSAR legal-hold bundle to fail policy validation" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_output" | grep -q "policy decision mismatch"; then
  echo "expected policy decision mismatch error for tampered DSAR bundle" >&2
  exit 1
fi

# Regression: #732
if ! printf '%s\n' "$tampered_output" | grep -q "expected final_decision=NO-GO"; then
  echo "expected explicit NO-GO mismatch message for tampered DSAR bundle" >&2
  exit 1
fi

echo "dsar legal-hold evidence bundle tests passed."

