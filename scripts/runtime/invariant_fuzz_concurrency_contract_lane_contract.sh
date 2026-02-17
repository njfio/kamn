#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PROPERTY_LANE="$ROOT_DIR/scripts/runtime/run_lifecycle_property_contract_lane.sh"
FUZZ_LANE="$ROOT_DIR/scripts/runtime/run_input_mutation_contract_lane.sh"
CONCURRENCY_LANE="$ROOT_DIR/scripts/runtime/run_concurrency_state_mutation_contract_lane.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_invariant_fuzz_concurrency_policy.sh"
RUNTIME_NETWORK_DOC="$ROOT_DIR/docs/foundation/runtime-network.md"
INVARIANTS_DOC="$ROOT_DIR/docs/foundation/invariants.md"
PERFORMANCE_DOC="$ROOT_DIR/docs/foundation/performance-target-benchmarking.md"
TESTING_STRATEGY_DOC="$ROOT_DIR/docs/testing/invariant-and-fuzz-strategy.md"

output_json=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-json)
      output_json="${2:-}"
      shift 2
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

for required_exec in "$PROPERTY_LANE" "$FUZZ_LANE" "$CONCURRENCY_LANE" "$POLICY_CHECKER"; do
  if [ ! -x "$required_exec" ]; then
    echo "expected executable script '$required_exec'" >&2
    exit 1
  fi
done

for required_doc in "$RUNTIME_NETWORK_DOC" "$INVARIANTS_DOC" "$PERFORMANCE_DOC" "$TESTING_STRATEGY_DOC"; do
  if [ ! -f "$required_doc" ]; then
    echo "expected required documentation file '$required_doc'" >&2
    exit 1
  fi
done

max_seconds="${KAMN_RUNTIME_INVARIANT_FUZZ_CONCURRENCY_MAX_SECONDS:-180}"
if [[ ! "$max_seconds" =~ ^[1-9][0-9]*$ ]]; then
  echo "contract max seconds must be a positive integer" >&2
  exit 1
fi

start_epoch="$(date +%s)"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT
property_report="$TMP_DIR/lifecycle-property-contract-report.json"
fuzz_report="$TMP_DIR/input-mutation-contract-report.json"
concurrency_report="$TMP_DIR/concurrency-mutation-contract-report.json"

property_output="$(bash "$PROPERTY_LANE" --output-json "$property_report")"
if ! printf '%s\n' "$property_output" | grep -Fq "runtime lifecycle property contract lane tests passed."; then
  echo "expected lifecycle property lane success marker" >&2
  exit 1
fi
if ! printf '%s\n' "$property_output" | grep -Fq "runtime_lifecycle_property_contract_report=$property_report"; then
  echo "expected lifecycle property lane report marker" >&2
  exit 1
fi

fuzz_output="$(bash "$FUZZ_LANE" --output-json "$fuzz_report")"
if ! printf '%s\n' "$fuzz_output" | grep -Fq "runtime input mutation contract lane tests passed."; then
  echo "expected input mutation lane success marker" >&2
  exit 1
fi
if ! printf '%s\n' "$fuzz_output" | grep -Fq "runtime_input_mutation_contract_report=$fuzz_report"; then
  echo "expected input mutation lane report marker" >&2
  exit 1
fi

concurrency_output="$(bash "$CONCURRENCY_LANE" --output-json "$concurrency_report")"
if ! printf '%s\n' "$concurrency_output" | grep -Fq "runtime concurrency state mutation contract lane tests passed."; then
  echo "expected concurrency state mutation lane success marker" >&2
  exit 1
fi
if ! printf '%s\n' "$concurrency_output" | grep -Fq "runtime_concurrency_mutation_contract_report=$concurrency_report"; then
  echo "expected concurrency state mutation lane report marker" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "runtime invariant/fuzz/concurrency contract lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

summary_report="$TMP_DIR/invariant-fuzz-concurrency-contract-report.json"

python3 - "$summary_report" "$property_report" "$fuzz_report" "$concurrency_report" "$elapsed_seconds" "$max_seconds" <<'PY'
import json
import pathlib
import sys

report_file = pathlib.Path(sys.argv[1])
property_report_file = pathlib.Path(sys.argv[2])
fuzz_report_file = pathlib.Path(sys.argv[3])
concurrency_report_file = pathlib.Path(sys.argv[4])
elapsed_seconds = int(sys.argv[5])
max_seconds = int(sys.argv[6])

property_report = json.loads(property_report_file.read_text(encoding="utf-8"))
schema_version = property_report.get("schema_version")
if schema_version != "kamn.runtime.lifecycle-property-contract-report.v1":
    raise SystemExit("unexpected lifecycle property report schema")
if property_report.get("status") != "pass":
    raise SystemExit("lifecycle property report status must be pass")
artifact_key = property_report.get("replay_artifact_key")
if artifact_key != "lifecycle_property_replay:v1":
    raise SystemExit("unexpected lifecycle property replay artifact key")
executed_tests = property_report.get("executed_tests")
if not isinstance(executed_tests, list) or not executed_tests:
    raise SystemExit("lifecycle property report must include non-empty executed_tests")

fuzz_report = json.loads(fuzz_report_file.read_text(encoding="utf-8"))
fuzz_schema_version = fuzz_report.get("schema_version")
if fuzz_schema_version != "kamn.runtime.input-mutation-contract-report.v1":
    raise SystemExit("unexpected input mutation report schema")
if fuzz_report.get("status") != "pass":
    raise SystemExit("input mutation report status must be pass")
fuzz_artifact_key = fuzz_report.get("replay_artifact_key")
if fuzz_artifact_key != "input_mutation_replay:v1":
    raise SystemExit("unexpected input mutation replay artifact key")
envelope_test_count = fuzz_report.get("envelope_test_count")
did_test_count = fuzz_report.get("did_test_count")
if (
    not isinstance(envelope_test_count, int)
    or envelope_test_count <= 0
    or not isinstance(did_test_count, int)
    or did_test_count <= 0
):
    raise SystemExit("input mutation report must include positive envelope/did test counts")

concurrency_report = json.loads(concurrency_report_file.read_text(encoding="utf-8"))
concurrency_schema_version = concurrency_report.get("schema_version")
if concurrency_schema_version != "kamn.runtime.concurrency-mutation-contract-report.v1":
    raise SystemExit("unexpected concurrency mutation report schema")
if concurrency_report.get("status") != "pass":
    raise SystemExit("concurrency mutation report status must be pass")
concurrency_artifact_key = concurrency_report.get("replay_artifact_key")
if concurrency_artifact_key != "concurrency_mutation_replay:v1":
    raise SystemExit("unexpected concurrency mutation replay artifact key")
concurrency_test_count = concurrency_report.get("test_count")
if not isinstance(concurrency_test_count, int) or concurrency_test_count <= 0:
    raise SystemExit("concurrency mutation report must include positive test_count")

summary = {
    "schema_version": "kamn.runtime.invariant-fuzz-concurrency-contract-report.v1",
    "status": "pass",
    "property_lane_status": "pass",
    "fuzz_lane_status": "pass",
    "concurrency_lane_status": "pass",
    "ci_smoke_local_heavy_boundary_status": "verified",
    "ci_smoke_lane_cost_profile": "low",
    "local_heavy_lane_execution_mode": "opt_in",
    "property_replay_schema_version": schema_version,
    "property_replay_artifact_key": artifact_key,
    "property_replay_test_count": len(executed_tests),
    "fuzz_replay_schema_version": fuzz_schema_version,
    "fuzz_replay_artifact_key": fuzz_artifact_key,
    "fuzz_replay_test_count": envelope_test_count + did_test_count,
    "concurrency_replay_schema_version": concurrency_schema_version,
    "concurrency_replay_artifact_key": concurrency_artifact_key,
    "concurrency_replay_test_count": concurrency_test_count,
    "elapsed_seconds": elapsed_seconds,
    "max_seconds": max_seconds,
    "reason_taxonomy_version": "kamn.runtime.invariant-fuzz-concurrency-policy-reason-taxonomy.v1",
    "reason_codes_csv": "property_lane_failed,fuzz_lane_failed,concurrency_lane_failed,runtime_budget_exceeded,ci_smoke_local_heavy_boundary_status_mismatch,ci_smoke_lane_cost_profile_mismatch,local_heavy_lane_execution_mode_mismatch,missing_required_report_fields,schema_version_mismatch,status_value_invalid,lane_status_value_invalid,property_replay_schema_version_mismatch,property_replay_artifact_key_mismatch,property_replay_test_count_invalid,fuzz_replay_schema_version_mismatch,fuzz_replay_artifact_key_mismatch,fuzz_replay_test_count_invalid,concurrency_replay_schema_version_mismatch,concurrency_replay_artifact_key_mismatch,concurrency_replay_test_count_invalid,elapsed_seconds_invalid,max_seconds_invalid,reason_codes_payload_invalid,status_contract_mismatch,reason_codes_contract_mismatch,reason_taxonomy_version_mismatch,reason_codes_csv_mismatch,reason_codes_value_mismatch,final_decision_mismatch",
    "reason_codes_value": "none",
    "final_decision": "GO",
    "reason_codes": ["none"],
}
report_file.write_text(json.dumps(summary, separators=(",", ":")), encoding="utf-8")
PY

policy_output="$(bash "$POLICY_CHECKER" --report-file "$summary_report")"
if ! printf '%s\n' "$policy_output" | grep -Fq "status=ok"; then
  echo "expected invariant/fuzz/concurrency policy checker success marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -Fq "invariant_policy_reason_taxonomy_version=kamn.runtime.invariant-fuzz-concurrency-policy-reason-taxonomy.v1"; then
  echo "expected invariant/fuzz/concurrency policy checker taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -Fq "invariant_policy_reason_codes_value=none"; then
  echo "expected invariant/fuzz/concurrency policy checker reason value marker" >&2
  exit 1
fi

if ! grep -Fq "run_invariant_fuzz_concurrency_contract_lane.sh" "$RUNTIME_NETWORK_DOC"; then
  echo "expected runtime-network docs to reference invariant/fuzz/concurrency lane command" >&2
  exit 1
fi

if ! grep -Fq "check_invariant_fuzz_concurrency_policy.sh" "$RUNTIME_NETWORK_DOC"; then
  echo "expected runtime-network docs to reference invariant/fuzz/concurrency policy checker command" >&2
  exit 1
fi

if ! grep -Fq "kamn.runtime.invariant-fuzz-concurrency-contract-report.v1" "$RUNTIME_NETWORK_DOC"; then
  echo "expected runtime-network docs to reference invariant/fuzz/concurrency report schema" >&2
  exit 1
fi

if ! grep -Fq "run_invariant_fuzz_concurrency_contract_lane.sh" "$INVARIANTS_DOC"; then
  echo "expected invariants docs to reference invariant/fuzz/concurrency lane command" >&2
  exit 1
fi

if ! grep -Fq "kamn.runtime.lifecycle-property-contract-report.v1" "$INVARIANTS_DOC"; then
  echo "expected invariants docs to reference lifecycle property report schema" >&2
  exit 1
fi

if ! grep -Fq "lifecycle_property_replay:v1" "$INVARIANTS_DOC"; then
  echo "expected invariants docs to reference lifecycle property replay artifact key" >&2
  exit 1
fi

if ! grep -Fq "kamn.runtime.input-mutation-contract-report.v1" "$INVARIANTS_DOC"; then
  echo "expected invariants docs to reference input mutation report schema" >&2
  exit 1
fi

if ! grep -Fq "input_mutation_replay:v1" "$INVARIANTS_DOC"; then
  echo "expected invariants docs to reference input mutation replay artifact key" >&2
  exit 1
fi

if ! grep -Fq "kamn.runtime.concurrency-mutation-contract-report.v1" "$INVARIANTS_DOC"; then
  echo "expected invariants docs to reference concurrency mutation report schema" >&2
  exit 1
fi

if ! grep -Fq "concurrency_mutation_replay:v1" "$INVARIANTS_DOC"; then
  echo "expected invariants docs to reference concurrency mutation replay artifact key" >&2
  exit 1
fi

if ! grep -Fq "KAMN_RUNTIME_INVARIANT_FUZZ_CONCURRENCY_MAX_SECONDS" "$PERFORMANCE_DOC"; then
  echo "expected performance benchmarking docs to reference invariant/fuzz/concurrency runtime budget env" >&2
  exit 1
fi

if ! grep -Fq "run_lifecycle_property_contract_lane.sh" "$TESTING_STRATEGY_DOC"; then
  echo "expected invariant/fuzz strategy doc to reference lifecycle property lane command" >&2
  exit 1
fi

if ! grep -Fq "kamn.runtime.lifecycle-property-contract-report.v1" "$TESTING_STRATEGY_DOC"; then
  echo "expected invariant/fuzz strategy doc to reference lifecycle property report schema" >&2
  exit 1
fi

if ! grep -Fq "lifecycle_property_replay:v1" "$TESTING_STRATEGY_DOC"; then
  echo "expected invariant/fuzz strategy doc to reference lifecycle property replay artifact key" >&2
  exit 1
fi

if ! grep -Fq "kamn.runtime.input-mutation-contract-report.v1" "$TESTING_STRATEGY_DOC"; then
  echo "expected invariant/fuzz strategy doc to reference input mutation report schema" >&2
  exit 1
fi

if ! grep -Fq "input_mutation_replay:v1" "$TESTING_STRATEGY_DOC"; then
  echo "expected invariant/fuzz strategy doc to reference input mutation replay artifact key" >&2
  exit 1
fi

if ! grep -Fq "kamn.runtime.concurrency-mutation-contract-report.v1" "$TESTING_STRATEGY_DOC"; then
  echo "expected invariant/fuzz strategy doc to reference concurrency mutation report schema" >&2
  exit 1
fi

if ! grep -Fq "concurrency_mutation_replay:v1" "$TESTING_STRATEGY_DOC"; then
  echo "expected invariant/fuzz strategy doc to reference concurrency mutation replay artifact key" >&2
  exit 1
fi

if ! grep -Fq "run_input_mutation_contract_lane.sh" "$TESTING_STRATEGY_DOC"; then
  echo "expected invariant/fuzz strategy doc to reference input mutation lane command" >&2
  exit 1
fi

if ! grep -Fq "run_concurrency_state_mutation_contract_lane.sh" "$TESTING_STRATEGY_DOC"; then
  echo "expected invariant/fuzz strategy doc to reference concurrency state mutation lane command" >&2
  exit 1
fi

if ! grep -Fq "run_invariant_fuzz_concurrency_contract_lane.sh" "$TESTING_STRATEGY_DOC"; then
  echo "expected invariant/fuzz strategy doc to reference combined contract lane command" >&2
  exit 1
fi

if ! grep -Fq "check_invariant_fuzz_concurrency_policy.sh" "$TESTING_STRATEGY_DOC"; then
  echo "expected invariant/fuzz strategy doc to reference combined policy checker command" >&2
  exit 1
fi

if ! grep -Fq "kamn.runtime.invariant-fuzz-concurrency-contract-report.v1" "$TESTING_STRATEGY_DOC"; then
  echo "expected invariant/fuzz strategy doc to reference combined report schema" >&2
  exit 1
fi

if ! grep -Fq "KAMN_RUNTIME_INVARIANT_FUZZ_CONCURRENCY_MAX_SECONDS" "$TESTING_STRATEGY_DOC"; then
  echo "expected invariant/fuzz strategy doc to reference combined lane runtime budget env" >&2
  exit 1
fi

if ! grep -Fq "KAMN_RUNTIME_LIFECYCLE_PROPERTY_MAX_SECONDS" "$TESTING_STRATEGY_DOC"; then
  echo "expected invariant/fuzz strategy doc to reference lifecycle property runtime budget env" >&2
  exit 1
fi

if ! grep -Fq "KAMN_RUNTIME_INPUT_MUTATION_MAX_SECONDS" "$TESTING_STRATEGY_DOC"; then
  echo "expected invariant/fuzz strategy doc to reference input mutation runtime budget env" >&2
  exit 1
fi

if ! grep -Fq "KAMN_RUNTIME_CONCURRENCY_MUTATION_MAX_SECONDS" "$TESTING_STRATEGY_DOC"; then
  echo "expected invariant/fuzz strategy doc to reference concurrency mutation runtime budget env" >&2
  exit 1
fi

if [[ -n "$output_json" ]]; then
  cp "$summary_report" "$output_json"
fi

echo "runtime_invariant_fuzz_concurrency_property=pass"
echo "runtime_invariant_fuzz_concurrency_fuzz=pass"
echo "runtime_invariant_fuzz_concurrency_concurrency=pass"
echo "runtime_invariant_fuzz_concurrency_policy=ok"
echo "runtime invariant/fuzz/concurrency contract lane tests passed."
