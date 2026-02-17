#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LANE_SCRIPT="$ROOT_DIR/scripts/runtime/run_invariant_fuzz_concurrency_contract_lane.sh"
SHARED_CONTRACT="$ROOT_DIR/scripts/runtime/invariant_fuzz_concurrency_contract_lane_contract.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/runtime_invariant_fuzz_concurrency_contract_lane.json"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_invariant_fuzz_concurrency_policy.sh"
TESTING_STRATEGY_DOC="$ROOT_DIR/docs/testing/invariant-and-fuzz-strategy.md"
EXPECTED_REASON_TAXONOMY_VERSION="kamn.runtime.invariant-fuzz-concurrency-policy-reason-taxonomy.v1"
EXPECTED_REASON_CODES_CSV="property_lane_failed,fuzz_lane_failed,concurrency_lane_failed,runtime_budget_exceeded,missing_required_report_fields,schema_version_mismatch,status_value_invalid,lane_status_value_invalid,property_replay_schema_version_mismatch,property_replay_artifact_key_mismatch,property_replay_test_count_invalid,fuzz_replay_schema_version_mismatch,fuzz_replay_artifact_key_mismatch,fuzz_replay_test_count_invalid,concurrency_replay_schema_version_mismatch,concurrency_replay_artifact_key_mismatch,concurrency_replay_test_count_invalid,elapsed_seconds_invalid,max_seconds_invalid,reason_codes_payload_invalid,status_contract_mismatch,reason_codes_contract_mismatch,reason_taxonomy_version_mismatch,reason_codes_csv_mismatch,reason_codes_value_mismatch,final_decision_mismatch"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$LANE_SCRIPT" ]; then
  echo "expected invariant/fuzz/concurrency contract lane script to be executable" >&2
  exit 1
fi

if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected invariant/fuzz/concurrency policy checker script to be executable" >&2
  exit 1
fi

if [ ! -x "$SHARED_CONTRACT" ]; then
  echo "expected invariant/fuzz/concurrency shared contract module to be executable" >&2
  exit 1
fi

if [ ! -f "$TESTING_STRATEGY_DOC" ]; then
  echo "expected invariant/fuzz strategy doc to exist" >&2
  exit 1
fi

report_file="$TMP_DIR/invariant-fuzz-concurrency-contract-report.json"
lane_output="$(
  bash "$LANE_SCRIPT" --output-json "$report_file"
)"

required_markers=(
  "runtime_invariant_fuzz_concurrency_property=pass"
  "runtime_invariant_fuzz_concurrency_fuzz=pass"
  "runtime_invariant_fuzz_concurrency_concurrency=pass"
  "runtime_invariant_fuzz_concurrency_policy=ok"
  "runtime invariant/fuzz/concurrency contract lane tests passed."
)

for marker in "${required_markers[@]}"; do
  if ! printf '%s\n' "$lane_output" | grep -Fq -- "$marker"; then
    echo "expected invariant/fuzz/concurrency contract lane output marker '$marker'" >&2
    exit 1
  fi
done

python3 - "$report_file" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
assert report["schema_version"] == "kamn.runtime.invariant-fuzz-concurrency-contract-report.v1"
assert report["status"] == "pass"
assert report["property_lane_status"] == "pass"
assert report["fuzz_lane_status"] == "pass"
assert report["concurrency_lane_status"] == "pass"
assert report["property_replay_schema_version"] == "kamn.runtime.lifecycle-property-contract-report.v1"
assert report["property_replay_artifact_key"] == "lifecycle_property_replay:v1"
assert report["property_replay_test_count"] >= 12
assert report["fuzz_replay_schema_version"] == "kamn.runtime.input-mutation-contract-report.v1"
assert report["fuzz_replay_artifact_key"] == "input_mutation_replay:v1"
assert report["fuzz_replay_test_count"] >= 10
assert report["concurrency_replay_schema_version"] == "kamn.runtime.concurrency-mutation-contract-report.v1"
assert report["concurrency_replay_artifact_key"] == "concurrency_mutation_replay:v1"
assert report["concurrency_replay_test_count"] >= 12
assert report["reason_taxonomy_version"] == "kamn.runtime.invariant-fuzz-concurrency-policy-reason-taxonomy.v1"
assert report["reason_codes_csv"] == "property_lane_failed,fuzz_lane_failed,concurrency_lane_failed,runtime_budget_exceeded,missing_required_report_fields,schema_version_mismatch,status_value_invalid,lane_status_value_invalid,property_replay_schema_version_mismatch,property_replay_artifact_key_mismatch,property_replay_test_count_invalid,fuzz_replay_schema_version_mismatch,fuzz_replay_artifact_key_mismatch,fuzz_replay_test_count_invalid,concurrency_replay_schema_version_mismatch,concurrency_replay_artifact_key_mismatch,concurrency_replay_test_count_invalid,elapsed_seconds_invalid,max_seconds_invalid,reason_codes_payload_invalid,status_contract_mismatch,reason_codes_contract_mismatch,reason_taxonomy_version_mismatch,reason_codes_csv_mismatch,reason_codes_value_mismatch,final_decision_mismatch"
assert report["reason_codes_value"] == "none"
assert report["final_decision"] == "GO"
assert report["reason_codes"] == ["none"]
PY

if ! grep -Fq "check_invariant_fuzz_concurrency_policy.sh" "$SHARED_CONTRACT"; then
  echo "expected invariant/fuzz/concurrency lane to enforce policy checker" >&2
  exit 1
fi

if ! grep -Fq "docs/testing/invariant-and-fuzz-strategy.md" "$SHARED_CONTRACT"; then
  echo "expected invariant/fuzz/concurrency lane to enforce testing strategy doc contract" >&2
  exit 1
fi

if [ ! -L "$LANE_SCRIPT" ]; then
  echo "expected invariant/fuzz/concurrency wrapper to be a dispatcher symlink" >&2
  exit 1
fi

if [ "$(readlink "$LANE_SCRIPT")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected invariant/fuzz/concurrency wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$LANE_SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected invariant/fuzz/concurrency wrapper to resolve runtime manifest via dispatcher" >&2
  exit 1
fi

if ! grep -Fq "invariant_fuzz_concurrency_contract_lane_contract.sh" "$MANIFEST_FILE"; then
  echo "expected invariant/fuzz/concurrency manifest to dispatch shared contract module" >&2
  exit 1
fi

policy_output="$(bash "$POLICY_CHECKER" --report-file "$report_file")"
if ! printf '%s\n' "$policy_output" | grep -Fq "invariant_policy_reason_taxonomy_version=$EXPECTED_REASON_TAXONOMY_VERSION"; then
  echo "expected invariant/fuzz/concurrency checker taxonomy marker for contract-lane report" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -Fq "invariant_policy_reason_codes_csv=$EXPECTED_REASON_CODES_CSV"; then
  echo "expected invariant/fuzz/concurrency checker reason codes csv marker for contract-lane report" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -Fq "invariant_policy_reason_codes_value=none"; then
  echo "expected invariant/fuzz/concurrency checker reason value marker for contract-lane report" >&2
  exit 1
fi

echo "runtime invariant/fuzz/concurrency contract lane script tests passed."
