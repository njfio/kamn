#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LANE_SCRIPT="$ROOT_DIR/scripts/runtime/run_invariant_fuzz_concurrency_contract_lane.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_invariant_fuzz_concurrency_policy.sh"
TESTING_STRATEGY_DOC="$ROOT_DIR/docs/testing/invariant-and-fuzz-strategy.md"
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
# Regression: #897
assert report["reason_codes"] == ["none"]
PY

if ! grep -Fq "check_invariant_fuzz_concurrency_policy.sh" "$LANE_SCRIPT"; then
  echo "expected invariant/fuzz/concurrency lane to enforce policy checker" >&2
  exit 1
fi

if ! grep -Fq "docs/testing/invariant-and-fuzz-strategy.md" "$LANE_SCRIPT"; then
  echo "expected invariant/fuzz/concurrency lane to enforce testing strategy doc contract" >&2
  exit 1
fi

echo "runtime invariant/fuzz/concurrency contract lane script tests passed."
