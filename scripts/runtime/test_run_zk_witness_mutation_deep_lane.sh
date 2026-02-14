#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_LANE="$ROOT_DIR/scripts/runtime/run_zk_witness_mutation_contract_lane.sh"
DEEP_LANE="$ROOT_DIR/scripts/runtime/run_zk_witness_mutation_deep_lane.sh"
DEEP_LANE_IMPL="$ROOT_DIR/scripts/runtime/run_zk_witness_mutation_deep_lane_impl.sh"
DEEP_MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/runtime_zk_witness_mutation_deep_lane.json"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"

if [ ! -x "$FAST_LANE" ]; then
  echo "expected runtime zk witness mutation fast-lane script to be executable" >&2
  exit 1
fi

if [ ! -x "$DEEP_LANE" ]; then
  echo "expected runtime zk witness mutation deep-lane script to be executable" >&2
  exit 1
fi

if [ ! -x "$DEEP_LANE_IMPL" ]; then
  echo "expected runtime zk witness mutation deep-lane implementation script to be executable" >&2
  exit 1
fi

if [ ! -L "$DEEP_LANE" ]; then
  echo "expected runtime zk witness mutation deep-lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi

if [ "$(readlink "$DEEP_LANE")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected runtime zk witness mutation deep-lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$DEEP_LANE")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$DEEP_MANIFEST_FILE" ]; then
  echo "expected runtime zk witness mutation deep-lane wrapper to resolve runtime deep manifest via dispatcher" >&2
  exit 1
fi

if ! grep -Fq "run_zk_witness_mutation_deep_lane_impl.sh" "$DEEP_MANIFEST_FILE"; then
  echo "expected runtime zk witness mutation deep-lane manifest to dispatch implementation module" >&2
  exit 1
fi

if ! grep -Fq "run_zk_witness_mutation_contract_lane.sh" "$DEEP_LANE_IMPL"; then
  echo "expected zk witness mutation deep lane implementation to execute fast-lane checks first" >&2
  exit 1
fi

if ! grep -q "performance_zk_witness_mutation_deep_lane_stress -- --ignored" "$DEEP_LANE_IMPL"; then
  echo "expected zk witness mutation deep lane implementation to include ignored deep stress coverage" >&2
  exit 1
fi

lane_output="$(bash "$DEEP_LANE")"
if ! printf '%s\n' "$lane_output" | grep -q "runtime zk witness mutation deep lane tests passed."; then
  echo "expected runtime zk witness mutation deep lane success marker" >&2
  exit 1
fi

echo "runtime zk witness mutation deep lane script tests passed."
