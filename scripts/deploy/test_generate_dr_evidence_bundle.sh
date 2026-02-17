#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)/scripts/lib/common.sh"
GENERATOR="$KAMN_ROOT/scripts/deploy/generate_dr_evidence_bundle.sh"
POLICY_CHECKER="$KAMN_ROOT/scripts/deploy/check_release_slo_gates.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$GENERATOR" ]; then
  echo "expected dr evidence bundle generator to be executable" >&2
  exit 1
fi

if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected release SLO gate policy checker to be executable" >&2
  exit 1
fi

go_bundle="$TMP_DIR/dr-go.json"
go_generate_output="$(
  bash "$GENERATOR" \
    --output-file "$go_bundle" \
    --drill-id "dr-2026-02-08-a" \
    --recovery-rto-seconds 240 \
    --recovery-rpo-seconds 90 \
    --max-rto-seconds 300 \
    --max-rpo-seconds 120 \
    --rollback-restored true \
    --evidence-complete true \
    --ci-fast-gate PASS
)"

assert_eq "$(extract_value "$go_generate_output" "status")" "generated" "expected GO DR bundle generation to succeed"
assert_eq "$(extract_value "$go_generate_output" "final_decision")" "GO" "expected DR generator to derive GO decision"

go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$go_bundle")"
assert_eq "$(extract_value "$go_policy_output" "status")" "ok" "expected GO DR policy check to pass"
assert_eq "$(extract_value "$go_policy_output" "final_decision")" "GO" "expected GO DR policy decision"

no_go_bundle="$TMP_DIR/dr-no-go.json"
no_go_generate_output="$(
  bash "$GENERATOR" \
    --output-file "$no_go_bundle" \
    --drill-id "dr-2026-02-08-b" \
    --recovery-rto-seconds 360 \
    --recovery-rpo-seconds 90 \
    --max-rto-seconds 300 \
    --max-rpo-seconds 120 \
    --rollback-restored true \
    --evidence-complete true \
    --ci-fast-gate PASS
)"

assert_eq "$(extract_value "$no_go_generate_output" "final_decision")" "NO-GO" "expected SLO threshold breach to force NO-GO"

no_go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$no_go_bundle")"
assert_eq "$(extract_value "$no_go_policy_output" "status")" "ok" "expected NO-GO DR policy check to pass"
assert_eq "$(extract_value "$no_go_policy_output" "final_decision")" "NO-GO" "expected NO-GO DR policy decision"

tampered_bundle="$TMP_DIR/dr-tampered.json"
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
  echo "expected tampered DR decision bundle to fail policy validation" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_output" | grep -q "expected final_decision=NO-GO"; then
  echo "expected explicit final-decision mismatch error from DR policy checker" >&2
  exit 1
fi

# Regression: #623
if ! printf '%s\n' "$tampered_output" | grep -q "rto threshold exceeded"; then
  echo "expected rto threshold regression guard to be enforced" >&2
  exit 1
fi

echo "dr evidence bundle tests passed."
