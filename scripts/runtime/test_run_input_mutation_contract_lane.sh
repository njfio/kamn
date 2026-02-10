#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/runtime/run_input_mutation_contract_lane.sh"

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected runtime input mutation contract lane script to be executable" >&2
  exit 1
fi

if ! grep -q "functional_envelope_mutation_suite_covers_malformed_truncated_and_tampered_classes" "$CONTRACT_LANE"; then
  echo "expected mutation lane to include malformed/truncated/tampered envelope coverage" >&2
  exit 1
fi

if ! grep -q "functional_did_mutation_suite_covers_normalization_encoding_and_method_mismatch_classes" "$CONTRACT_LANE"; then
  echo "expected mutation lane to include normalization/encoding/method mismatch DID coverage" >&2
  exit 1
fi

if ! grep -q "regression_envelope_mutation_reason_signatures_remain_stable" "$CONTRACT_LANE"; then
  echo "expected mutation lane to include envelope fail-closed regression coverage" >&2
  exit 1
fi

if ! grep -q "regression_did_mutation_reason_signatures_remain_stable" "$CONTRACT_LANE"; then
  echo "expected mutation lane to include DID fail-closed regression coverage" >&2
  exit 1
fi

if ! grep -q "run_zk_witness_mutation_contract_lane.sh" "$CONTRACT_LANE"; then
  echo "expected mutation lane to include ZK witness mutation contract lane coverage" >&2
  exit 1
fi

if ! grep -q "KAMN_RUNTIME_ZK_WITNESS_MUTATION_DEEP" "$CONTRACT_LANE"; then
  echo "expected mutation lane to support fast/deep ZK witness mutation routing" >&2
  exit 1
fi

lane_output="$(bash "$CONTRACT_LANE")"
if ! printf '%s\n' "$lane_output" | grep -q "runtime input mutation contract lane tests passed."; then
  echo "expected runtime input mutation contract lane success marker" >&2
  exit 1
fi

echo "runtime input mutation contract lane script tests passed."
