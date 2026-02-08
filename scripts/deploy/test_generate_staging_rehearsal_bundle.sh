#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/deploy/generate_staging_rehearsal_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/deploy/check_staging_rehearsal_policy.sh"
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
  echo "expected staging rehearsal bundle generator to be executable" >&2
  exit 1
fi

if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected staging rehearsal policy checker to be executable" >&2
  exit 1
fi

go_bundle="$TMP_DIR/rehearsal-go.json"
go_generate_output="$(
  bash "$GENERATOR" \
    --output-file "$go_bundle" \
    --release-candidate "v1.1.0-rc.1" \
    --deploy-status PASS \
    --rollback-status PASS \
    --rollback-target-hash "state-hash-abc" \
    --post-rollback-hash "state-hash-abc" \
    --evidence-complete true \
    --ci-fast-gate PASS
)"

assert_eq "$(extract_value "$go_generate_output" "status")" "generated" "expected GO rehearsal bundle generation to succeed"
assert_eq "$(extract_value "$go_generate_output" "final_decision")" "GO" "expected generator to derive GO rehearsal decision"

go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$go_bundle")"
assert_eq "$(extract_value "$go_policy_output" "status")" "ok" "expected GO rehearsal policy check to pass"
assert_eq "$(extract_value "$go_policy_output" "final_decision")" "GO" "expected GO rehearsal policy check decision"

no_go_bundle="$TMP_DIR/rehearsal-no-go.json"
no_go_generate_output="$(
  bash "$GENERATOR" \
    --output-file "$no_go_bundle" \
    --release-candidate "v1.1.0-rc.2" \
    --deploy-status PASS \
    --rollback-status PASS \
    --rollback-target-hash "state-hash-expected" \
    --post-rollback-hash "state-hash-observed" \
    --evidence-complete true \
    --ci-fast-gate PASS
)"

assert_eq "$(extract_value "$no_go_generate_output" "final_decision")" "NO-GO" "expected rollback hash mismatch to force NO-GO"

no_go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$no_go_bundle")"
assert_eq "$(extract_value "$no_go_policy_output" "status")" "ok" "expected NO-GO rehearsal policy check to pass"
assert_eq "$(extract_value "$no_go_policy_output" "final_decision")" "NO-GO" "expected NO-GO rehearsal policy check decision"

tampered_bundle="$TMP_DIR/rehearsal-tampered.json"
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
  echo "expected tampered rehearsal decision bundle to fail policy validation" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_output" | grep -q "expected final_decision=NO-GO"; then
  echo "expected explicit final-decision mismatch error from rehearsal policy checker" >&2
  exit 1
fi

# Regression: #623
if ! printf '%s\n' "$tampered_output" | grep -q "rollback target hash mismatch"; then
  echo "expected rollback mismatch regression guard to be enforced" >&2
  exit 1
fi

echo "staging rehearsal bundle tests passed."
