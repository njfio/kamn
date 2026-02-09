#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/reputation/generate_weighted_decay_property_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/reputation/check_weighted_decay_property_policy.sh"
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
  echo "expected weighted decay property evidence generator to be executable" >&2
  exit 1
fi

if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected weighted decay property policy checker to be executable" >&2
  exit 1
fi

compact_report="$TMP_DIR/weighted-decay-compact-go.json"
cat >"$compact_report" <<'JSON'
{
  "schema_version": "kamn.reputation.weighted-decay.matrix.v1",
  "status": "pass",
  "case_count": 3,
  "failed_count": 0,
  "failed_case_ids": [],
  "cases": [
    {
      "case_id": "clean-go-001",
      "expected_abuse_penalty_kind": "None",
      "actual_abuse_penalty_kind": "None",
      "passed": true
    },
    {
      "case_id": "reciprocity-ring-001",
      "expected_abuse_penalty_kind": "ReciprocityRing",
      "actual_abuse_penalty_kind": "ReciprocityRing",
      "passed": true
    },
    {
      "case_id": "burst-spam-001",
      "expected_abuse_penalty_kind": "BurstSpam",
      "actual_abuse_penalty_kind": "BurstSpam",
      "passed": true
    }
  ]
}
JSON

adversarial_report="$TMP_DIR/weighted-decay-adversarial-go.json"
cat >"$adversarial_report" <<'JSON'
{
  "schema_version": "kamn.reputation.weighted-decay.matrix.v1",
  "status": "pass",
  "case_count": 2,
  "failed_count": 0,
  "failed_case_ids": [],
  "cases": [
    {
      "case_id": "churn-spike-001",
      "expected_abuse_penalty_kind": "ChurnSpike",
      "actual_abuse_penalty_kind": "ChurnSpike",
      "passed": true
    },
    {
      "case_id": "compound-abuse-001",
      "expected_abuse_penalty_kind": "Compound",
      "actual_abuse_penalty_kind": "Compound",
      "passed": true
    }
  ]
}
JSON

go_bundle="$TMP_DIR/weighted-decay-property-go-bundle.json"
go_output="$(
  bash "$GENERATOR" \
    --output-file "$go_bundle" \
    --lane contract \
    --compact-report-file "$compact_report" \
    --adversarial-report-file "$adversarial_report" \
    --property-suite-status pass \
    --runtime-budget-status within \
    --ci-fast-gate PASS
)"

assert_eq "$(extract_value "$go_output" "status")" "generated" "expected GO weighted decay property evidence generation to succeed"
assert_eq "$(extract_value "$go_output" "final_decision")" "GO" "expected GO weighted decay property evidence decision"

go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$go_bundle")"
assert_eq "$(extract_value "$go_policy_output" "status")" "ok" "expected GO weighted decay property policy check to pass"
assert_eq "$(extract_value "$go_policy_output" "final_decision")" "GO" "expected GO weighted decay property policy decision"

adversarial_fail_report="$TMP_DIR/weighted-decay-adversarial-fail.json"
cat >"$adversarial_fail_report" <<'JSON'
{
  "schema_version": "kamn.reputation.weighted-decay.matrix.v1",
  "status": "fail",
  "case_count": 2,
  "failed_count": 1,
  "failed_case_ids": ["churn-spike-001"],
  "cases": [
    {
      "case_id": "churn-spike-001",
      "expected_abuse_penalty_kind": "ChurnSpike",
      "actual_abuse_penalty_kind": "BurstSpam",
      "passed": false
    },
    {
      "case_id": "compound-abuse-001",
      "expected_abuse_penalty_kind": "Compound",
      "actual_abuse_penalty_kind": "Compound",
      "passed": true
    }
  ]
}
JSON

no_go_bundle="$TMP_DIR/weighted-decay-property-no-go-bundle.json"
no_go_output="$(
  bash "$GENERATOR" \
    --output-file "$no_go_bundle" \
    --lane contract \
    --compact-report-file "$compact_report" \
    --adversarial-report-file "$adversarial_fail_report" \
    --property-suite-status pass \
    --runtime-budget-status within \
    --ci-fast-gate PASS
)"

assert_eq "$(extract_value "$no_go_output" "status")" "generated" "expected NO-GO weighted decay property evidence generation to succeed"
assert_eq "$(extract_value "$no_go_output" "final_decision")" "NO-GO" "expected NO-GO weighted decay property evidence decision"

no_go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$no_go_bundle")"
assert_eq "$(extract_value "$no_go_policy_output" "status")" "ok" "expected NO-GO weighted decay property policy check to pass"
assert_eq "$(extract_value "$no_go_policy_output" "final_decision")" "NO-GO" "expected NO-GO weighted decay property policy decision"

tampered_bundle="$TMP_DIR/weighted-decay-property-tampered.json"
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
  echo "expected tampered weighted decay property evidence to fail policy validation" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_output" | grep -q "policy decision mismatch"; then
  echo "expected policy decision mismatch for tampered weighted decay property evidence" >&2
  exit 1
fi

# Regression: #933
if ! printf '%s\n' "$tampered_output" | grep -q "expected final_decision=NO-GO"; then
  echo "expected explicit NO-GO mismatch for weighted decay anti-gaming regression path" >&2
  exit 1
fi

echo "weighted decay property evidence bundle tests passed."
