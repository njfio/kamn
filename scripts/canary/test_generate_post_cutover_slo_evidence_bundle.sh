#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/canary/generate_post_cutover_slo_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/canary/check_post_cutover_slo_policy.sh"
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
  echo "expected post-cutover SLO evidence bundle generator to be executable" >&2
  exit 1
fi

if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected post-cutover SLO evidence policy checker to be executable" >&2
  exit 1
fi

go_bundle="$TMP_DIR/slo-go.json"
go_generate_output="$(
  bash "$GENERATOR" \
    --output-file "$go_bundle" \
    --window-minutes 15 \
    --p95-latency-ms 140 \
    --max-p95-latency-ms 200 \
    --error-rate-bps 18 \
    --max-error-rate-bps 25 \
    --delivery-success-bps 9992 \
    --min-delivery-success-bps 9950 \
    --snapshot-age-seconds 30 \
    --max-snapshot-age-seconds 120 \
    --evidence-complete true \
    --ci-fast-gate PASS
)"

assert_eq "$(extract_value "$go_generate_output" "status")" "generated" "expected GO SLO bundle generation to succeed"
assert_eq "$(extract_value "$go_generate_output" "final_decision")" "GO" "expected generator to derive GO SLO decision"

go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$go_bundle")"
assert_eq "$(extract_value "$go_policy_output" "status")" "ok" "expected GO SLO bundle policy check to pass"
assert_eq "$(extract_value "$go_policy_output" "final_decision")" "GO" "expected policy check to keep GO SLO decision"

no_go_bundle="$TMP_DIR/slo-no-go.json"
no_go_generate_output="$(
  bash "$GENERATOR" \
    --output-file "$no_go_bundle" \
    --window-minutes 15 \
    --p95-latency-ms 245 \
    --max-p95-latency-ms 200 \
    --error-rate-bps 18 \
    --max-error-rate-bps 25 \
    --delivery-success-bps 9992 \
    --min-delivery-success-bps 9950 \
    --snapshot-age-seconds 360 \
    --max-snapshot-age-seconds 120 \
    --evidence-complete true \
    --ci-fast-gate PASS
)"

assert_eq "$(extract_value "$no_go_generate_output" "final_decision")" "NO-GO" "expected stale/threshold-breached SLO bundle to force NO-GO"

no_go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$no_go_bundle")"
assert_eq "$(extract_value "$no_go_policy_output" "status")" "ok" "expected NO-GO SLO policy check to pass"
assert_eq "$(extract_value "$no_go_policy_output" "final_decision")" "NO-GO" "expected policy check to keep NO-GO SLO decision"

tampered_bundle="$TMP_DIR/slo-tampered.json"
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
  echo "expected tampered SLO decision bundle to fail policy validation" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_output" | grep -q "expected final_decision=NO-GO"; then
  echo "expected explicit final-decision mismatch error from SLO policy checker" >&2
  exit 1
fi

# Regression: #711
if ! printf '%s\n' "$tampered_output" | grep -q "stale-snapshot-evidence"; then
  echo "expected stale snapshot regression guard to be enforced" >&2
  exit 1
fi

echo "post-cutover SLO evidence bundle tests passed."
