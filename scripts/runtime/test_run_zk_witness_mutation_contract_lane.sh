#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/runtime/run_zk_witness_mutation_contract_lane.sh"

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected runtime zk witness mutation contract lane script to be executable" >&2
  exit 1
fi

if ! grep -q "fuzz_smoke_zk_witness_mutation_lane_is_panic_free_and_deterministic" "$CONTRACT_LANE"; then
  echo "expected zk witness mutation contract lane to include panic-free deterministic smoke coverage" >&2
  exit 1
fi

if ! grep -q "functional_zk_witness_mutation_suite_covers_malformed_missing_and_tampered_classes" "$CONTRACT_LANE"; then
  echo "expected zk witness mutation contract lane to include malformed/missing/tampered class coverage" >&2
  exit 1
fi

if ! grep -q "regression_zk_witness_mutation_reason_signatures_remain_stable" "$CONTRACT_LANE"; then
  echo "expected zk witness mutation contract lane to include fail-closed regression coverage" >&2
  exit 1
fi

lane_output="$(bash "$CONTRACT_LANE")"
if ! printf '%s\n' "$lane_output" | grep -q "runtime zk witness mutation contract lane tests passed."; then
  echo "expected runtime zk witness mutation contract lane success marker" >&2
  exit 1
fi

echo "runtime zk witness mutation contract lane script tests passed."
