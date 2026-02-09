#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/governance/generate_governance_simulation_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/governance/check_governance_simulation_policy.sh"
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
  echo "expected governance simulation evidence generator to be executable" >&2
  exit 1
fi

if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected governance simulation policy checker to be executable" >&2
  exit 1
fi

go_bundle="$TMP_DIR/governance-go.json"
go_output="$(
  bash "$GENERATOR" \
    --output-file "$go_bundle" \
    --proposal-id "gov-proposal-go-001" \
    --simulation-hash "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" \
    --simulation-complete true \
    --veto-window-open false \
    --veto-recorded false \
    --timelock-expired true \
    --required-approvals 2 \
    --received-approvals 2 \
    --ci-fast-gate PASS
)"

assert_eq "$(extract_value "$go_output" "status")" "generated" "expected GO bundle generation to pass"
assert_eq "$(extract_value "$go_output" "final_decision")" "GO" "expected GO policy decision"

go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$go_bundle")"
assert_eq "$(extract_value "$go_policy_output" "status")" "ok" "expected GO policy check to pass"
assert_eq "$(extract_value "$go_policy_output" "final_decision")" "GO" "expected GO policy checker decision"
assert_eq "$(extract_value "$go_policy_output" "failed_checks")" "none" "expected no failed checks for GO"

no_go_bundle="$TMP_DIR/governance-no-go.json"
no_go_output="$(
  bash "$GENERATOR" \
    --output-file "$no_go_bundle" \
    --proposal-id "gov-proposal-no-go-001" \
    --simulation-hash "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb" \
    --simulation-complete true \
    --veto-window-open false \
    --veto-recorded true \
    --timelock-expired true \
    --required-approvals 2 \
    --received-approvals 2 \
    --ci-fast-gate PASS
)"

assert_eq "$(extract_value "$no_go_output" "final_decision")" "NO-GO" "expected NO-GO policy decision"

no_go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$no_go_bundle")"
assert_eq "$(extract_value "$no_go_policy_output" "status")" "ok" "expected NO-GO policy check to pass"
assert_eq "$(extract_value "$no_go_policy_output" "final_decision")" "NO-GO" "expected NO-GO checker decision"
assert_eq "$(extract_value "$no_go_policy_output" "failed_checks")" "veto_recorded" "expected veto failure reason"

tampered_bundle="$TMP_DIR/governance-tampered.json"
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
  echo "expected tampered governance bundle to fail policy validation" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_output" | grep -q "policy decision mismatch"; then
  echo "expected policy decision mismatch error for tampered governance bundle" >&2
  exit 1
fi

# Regression: #733
if ! printf '%s\n' "$tampered_output" | grep -q "expected final_decision=NO-GO"; then
  echo "expected explicit NO-GO mismatch message for tampered governance bundle" >&2
  exit 1
fi

echo "governance simulation evidence bundle tests passed."

