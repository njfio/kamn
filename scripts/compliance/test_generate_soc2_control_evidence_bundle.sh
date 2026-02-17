#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)/scripts/lib/common.sh"
GENERATOR="$KAMN_ROOT/scripts/compliance/generate_soc2_control_evidence_bundle.sh"
POLICY_CHECKER="$KAMN_ROOT/scripts/compliance/check_soc2_control_evidence_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$GENERATOR" ]; then
  echo "expected SOC2 control evidence generator to be executable" >&2
  exit 1
fi

if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected SOC2 control evidence policy checker to be executable" >&2
  exit 1
fi

go_bundle="$TMP_DIR/soc2-go.json"
go_output="$(
  bash "$GENERATOR" \
    --output-file "$go_bundle" \
    --control-id "CC6.1" \
    --audit-period-start "2026-01-01" \
    --audit-period-end "2026-01-31" \
    --collector-did "did:kamn:auditor-go" \
    --evidence-uri "s3://kamn-audit/soc2/cc6_1/go/evidence.json" \
    --evidence-sha256 "sha256:5555555555555555555555555555555555555555555555555555555555555555" \
    --tamper-check PASS \
    --completeness-check PASS \
    --ci-fast-gate PASS
)"

assert_eq "$(extract_value "$go_output" "status")" "generated" "expected GO bundle generation to pass"
assert_eq "$(extract_value "$go_output" "final_decision")" "GO" "expected GO decision"

go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$go_bundle")"
assert_eq "$(extract_value "$go_policy_output" "status")" "ok" "expected GO policy check to pass"
assert_eq "$(extract_value "$go_policy_output" "final_decision")" "GO" "expected GO checker decision"
assert_eq "$(extract_value "$go_policy_output" "failed_checks")" "none" "expected no failed checks for GO"

no_go_bundle="$TMP_DIR/soc2-no-go.json"
no_go_output="$(
  bash "$GENERATOR" \
    --output-file "$no_go_bundle" \
    --control-id "CC7.2" \
    --audit-period-start "2026-01-01" \
    --audit-period-end "2026-01-31" \
    --collector-did "did:kamn:auditor-no-go" \
    --evidence-uri "s3://kamn-audit/soc2/cc7_2/no-go/evidence.json" \
    --evidence-sha256 "sha256:6666666666666666666666666666666666666666666666666666666666666666" \
    --tamper-check FAIL \
    --completeness-check PASS \
    --ci-fast-gate PASS
)"

assert_eq "$(extract_value "$no_go_output" "final_decision")" "NO-GO" "expected NO-GO decision"

no_go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$no_go_bundle")"
assert_eq "$(extract_value "$no_go_policy_output" "status")" "ok" "expected NO-GO policy check to pass"
assert_eq "$(extract_value "$no_go_policy_output" "final_decision")" "NO-GO" "expected NO-GO checker decision"
assert_eq "$(extract_value "$no_go_policy_output" "failed_checks")" "tamper" "expected tamper failure reason"

tampered_bundle="$TMP_DIR/soc2-tampered.json"
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
  echo "expected tampered SOC2 bundle to fail policy validation" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_output" | grep -q "policy decision mismatch"; then
  echo "expected policy decision mismatch error for tampered SOC2 bundle" >&2
  exit 1
fi

# Regression: #732
if ! printf '%s\n' "$tampered_output" | grep -q "expected final_decision=NO-GO"; then
  echo "expected explicit NO-GO mismatch message for tampered SOC2 bundle" >&2
  exit 1
fi

echo "soc2 control evidence bundle tests passed."

