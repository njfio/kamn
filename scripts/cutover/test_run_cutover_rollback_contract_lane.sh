#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"
FAST_SCRIPT="$ROOT_DIR/scripts/cutover/run_cutover_rollback_contract_lane.sh"
DEEP_SCRIPT="$ROOT_DIR/scripts/cutover/run_cutover_rollback_deep_lane.sh"
SHARED_CONTRACT="$ROOT_DIR/scripts/cutover/cutover_rollback_contract_lane_contract.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/cutover_cutover_rollback_contract_lane.json"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"

test_harness_require_executable "$FAST_SCRIPT" "expected cutover rollback fast-lane runner to be executable"

test_harness_require_executable "$DEEP_SCRIPT" "expected cutover rollback deep-lane runner to be executable"
test_harness_require_executable "$SHARED_CONTRACT" "expected cutover rollback shared contract-lane module to be executable"

tmp_out="$(mktemp)"
trap 'rm -f "$tmp_out"' EXIT

bash "$FAST_SCRIPT" >"$tmp_out"
if ! grep -q "cutover rollback contract lane tests passed." "$tmp_out"; then
  echo "expected cutover rollback contract lane success marker" >&2
  exit 1
fi

if [ ! -L "$FAST_SCRIPT" ]; then
  echo "expected cutover rollback contract lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi

if [ "$(readlink "$FAST_SCRIPT")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected cutover rollback contract lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$FAST_SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected cutover rollback wrapper to resolve cutover manifest via dispatcher" >&2
  exit 1
fi

if ! grep -Fq "cutover_rollback_contract_lane_contract.sh" "$MANIFEST_FILE"; then
  echo "expected cutover rollback manifest to dispatch shared contract module" >&2
  exit 1
fi

if ! grep -q "generate_cutover_rollback_evidence_bundle.sh" "$SHARED_CONTRACT"; then
  echo "expected cutover rollback shared contract module to execute rollback evidence generator" >&2
  exit 1
fi

if ! grep -q "check_cutover_rollback_evidence_policy.sh" "$SHARED_CONTRACT"; then
  echo "expected cutover rollback shared contract module to execute rollback policy checker" >&2
  exit 1
fi

if ! grep -Fq "run_cutover_rollback_contract_lane.sh" "$DEEP_SCRIPT"; then
  echo "expected rollback deep-lane script to execute fast-lane contract checks first" >&2
  exit 1
fi

if ! grep -q "final_decision=NO-GO" "$DEEP_SCRIPT"; then
  echo "expected rollback deep-lane script to validate NO-GO decision path" >&2
  exit 1
fi

echo "cutover rollback contract lane script tests passed."
