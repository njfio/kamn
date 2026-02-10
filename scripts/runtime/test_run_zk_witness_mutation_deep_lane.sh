#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_LANE="$ROOT_DIR/scripts/runtime/run_zk_witness_mutation_contract_lane.sh"
DEEP_LANE="$ROOT_DIR/scripts/runtime/run_zk_witness_mutation_deep_lane.sh"

if [ ! -x "$FAST_LANE" ]; then
  echo "expected runtime zk witness mutation fast-lane script to be executable" >&2
  exit 1
fi

if [ ! -x "$DEEP_LANE" ]; then
  echo "expected runtime zk witness mutation deep-lane script to be executable" >&2
  exit 1
fi

if ! grep -Fq "run_zk_witness_mutation_contract_lane.sh" "$DEEP_LANE"; then
  echo "expected zk witness mutation deep lane to execute fast-lane checks first" >&2
  exit 1
fi

if ! grep -q "performance_zk_witness_mutation_deep_lane_stress -- --ignored" "$DEEP_LANE"; then
  echo "expected zk witness mutation deep lane to include ignored deep stress coverage" >&2
  exit 1
fi

lane_output="$(bash "$DEEP_LANE")"
if ! printf '%s\n' "$lane_output" | grep -q "runtime zk witness mutation deep lane tests passed."; then
  echo "expected runtime zk witness mutation deep lane success marker" >&2
  exit 1
fi

echo "runtime zk witness mutation deep lane script tests passed."
