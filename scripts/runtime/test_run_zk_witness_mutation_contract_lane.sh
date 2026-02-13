#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/runtime/run_zk_witness_mutation_contract_lane.sh"
SHARED_CONTRACT="$ROOT_DIR/scripts/runtime/zk_witness_mutation_contract_lane_contract.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/runtime_zk_witness_mutation_contract_lane.json"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected runtime zk witness mutation contract lane script to be executable" >&2
  exit 1
fi

if [ ! -x "$SHARED_CONTRACT" ]; then
  echo "expected runtime zk witness mutation shared contract-lane module to be executable" >&2
  exit 1
fi

if ! grep -q "fuzz_smoke_zk_witness_mutation_lane_is_panic_free_and_deterministic" "$SHARED_CONTRACT"; then
  echo "expected zk witness mutation shared contract module to include panic-free deterministic smoke coverage" >&2
  exit 1
fi

if ! grep -q "functional_zk_witness_mutation_suite_covers_malformed_missing_and_tampered_classes" "$SHARED_CONTRACT"; then
  echo "expected zk witness mutation shared contract module to include malformed/missing/tampered class coverage" >&2
  exit 1
fi

if ! grep -q "regression_zk_witness_mutation_reason_signatures_remain_stable" "$SHARED_CONTRACT"; then
  echo "expected zk witness mutation shared contract module to include fail-closed regression coverage" >&2
  exit 1
fi

lane_output="$(bash "$CONTRACT_LANE")"
if ! printf '%s\n' "$lane_output" | grep -q "runtime zk witness mutation contract lane tests passed."; then
  echo "expected runtime zk witness mutation contract lane success marker" >&2
  exit 1
fi

if [ ! -L "$CONTRACT_LANE" ]; then
  echo "expected runtime zk witness mutation contract lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi

if [ "$(readlink "$CONTRACT_LANE")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected runtime zk witness mutation contract lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$CONTRACT_LANE")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected runtime zk witness mutation wrapper to resolve runtime manifest via dispatcher" >&2
  exit 1
fi

if ! grep -Fq "zk_witness_mutation_contract_lane_contract.sh" "$MANIFEST_FILE"; then
  echo "expected runtime zk witness mutation manifest to dispatch shared contract module" >&2
  exit 1
fi

echo "runtime zk witness mutation contract lane script tests passed."
