#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"
FAST_SCRIPT="$ROOT_DIR/scripts/cutover/run_mainnet_cutover_contract_lane.sh"
SHARED_CONTRACT="$ROOT_DIR/scripts/cutover/mainnet_cutover_contract_lane_contract.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/cutover_mainnet_cutover_contract_lane.json"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"

test_harness_require_executable "$FAST_SCRIPT" "expected mainnet cutover contract lane runner to be executable"
test_harness_require_executable "$SHARED_CONTRACT" "expected mainnet cutover shared contract-lane module to be executable"

tmp_out="$(mktemp)"
trap 'rm -f "$tmp_out"' EXIT

bash "$FAST_SCRIPT" >"$tmp_out"
if ! grep -q "mainnet cutover contract lane tests passed." "$tmp_out"; then
  echo "expected mainnet cutover contract lane success marker" >&2
  exit 1
fi

if [ ! -L "$FAST_SCRIPT" ]; then
  echo "expected mainnet cutover contract lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi

if [ "$(readlink "$FAST_SCRIPT")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected mainnet cutover contract lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$FAST_SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected mainnet cutover wrapper to resolve cutover manifest via dispatcher" >&2
  exit 1
fi

if ! grep -Fq "mainnet_cutover_contract_lane_contract.sh" "$MANIFEST_FILE"; then
  echo "expected mainnet cutover manifest to dispatch shared contract module" >&2
  exit 1
fi

if ! grep -Fq "validate_mainnet_cutover_manifest.py" "$SHARED_CONTRACT"; then
  echo "expected mainnet cutover shared contract module to run mainnet cutover validator" >&2
  exit 1
fi

if ! grep -Fq "mainnet_cutover_manifest.invalid_dependency.json" "$SHARED_CONTRACT"; then
  echo "expected mainnet cutover shared contract module to validate dependency regression fixture" >&2
  exit 1
fi

if ! grep -Fq "mainnet_cutover_manifest.invalid_approvals.json" "$SHARED_CONTRACT"; then
  echo "expected mainnet cutover shared contract module to validate approval regression fixture" >&2
  exit 1
fi

echo "mainnet cutover contract lane script tests passed."
