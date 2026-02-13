#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_LANE="$ROOT_DIR/scripts/runtime/run_input_mutation_coverage_guided_contract_lane.sh"
DEEP_LANE="$ROOT_DIR/scripts/runtime/run_input_mutation_coverage_guided_deep_lane.sh"

if [ ! -x "$FAST_LANE" ]; then
  echo "expected runtime input mutation coverage-guided fast-lane script to be executable" >&2
  exit 1
fi

if [ ! -x "$DEEP_LANE" ]; then
  echo "expected runtime input mutation coverage-guided deep-lane script to be executable" >&2
  exit 1
fi

if ! grep -Fq "run_input_mutation_coverage_guided_contract_lane.sh" "$DEEP_LANE"; then
  echo "expected coverage-guided deep lane to execute fast-lane checks first" >&2
  exit 1
fi

if ! grep -q "performance_input_mutation_coverage_guided_deep_lane_stress -- --ignored" "$DEEP_LANE"; then
  echo "expected coverage-guided deep lane to include ignored deep stress coverage" >&2
  exit 1
fi

if ! grep -q "KAMN_RUNTIME_INPUT_MUTATION_COVERAGE_GUIDED_DEEP_MAX_SECONDS" "$DEEP_LANE"; then
  echo "expected coverage-guided deep lane to enforce deterministic runtime budget" >&2
  exit 1
fi

lane_output="$(bash "$DEEP_LANE")"
if ! printf '%s\n' "$lane_output" | grep -q "runtime input mutation coverage-guided deep lane tests passed."; then
  echo "expected coverage-guided deep lane success marker" >&2
  exit 1
fi

echo "runtime input mutation coverage-guided deep lane script tests passed."
