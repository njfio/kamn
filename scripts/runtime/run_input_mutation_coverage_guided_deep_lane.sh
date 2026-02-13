#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_LANE="$ROOT_DIR/scripts/runtime/run_input_mutation_coverage_guided_contract_lane.sh"
cd "$ROOT_DIR"

max_seconds="${KAMN_RUNTIME_INPUT_MUTATION_COVERAGE_GUIDED_DEEP_MAX_SECONDS:-180}"
if [[ ! "$max_seconds" =~ ^[1-9][0-9]*$ ]]; then
  echo "KAMN_RUNTIME_INPUT_MUTATION_COVERAGE_GUIDED_DEEP_MAX_SECONDS must be a positive integer" >&2
  exit 1
fi

start_epoch="$(date +%s)"
bash "$FAST_LANE" >/dev/null
cargo test -p kamn-core --test input_mutation_coverage_guided performance_input_mutation_coverage_guided_deep_lane_stress -- --ignored >/dev/null

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "runtime input mutation coverage-guided deep lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

echo "runtime input mutation coverage-guided deep lane tests passed."
