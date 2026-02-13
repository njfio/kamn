#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/runtime/run_watchdog_proof_consensus_contract_lane.sh"
SHARED_CONTRACT="$ROOT_DIR/scripts/runtime/watchdog_proof_consensus_contract_lane_contract.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/runtime_watchdog_proof_consensus_contract_lane.json"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$SCRIPT" ]; then
  echo "expected watchdog proof consensus contract lane script to be executable" >&2
  exit 1
fi

if [ ! -x "$SHARED_CONTRACT" ]; then
  echo "expected watchdog proof consensus shared contract module to be executable" >&2
  exit 1
fi

bundle_file="$TMP_DIR/watchdog-proof-consensus-contract-bundle.json"
lane_output="$(bash "$SCRIPT" --skip-tests --output-file "$bundle_file")"

if ! printf '%s\n' "$lane_output" | grep -q "watchdog proof consensus contract lane tests passed."; then
  echo "expected watchdog proof consensus contract lane success marker" >&2
  exit 1
fi

if [ ! -f "$bundle_file" ]; then
  echo "expected watchdog proof consensus contract lane to emit evidence bundle" >&2
  exit 1
fi

if ! grep -q '"schema_version": "kamn.runtime.watchdog-proof-consensus-report.v1"' "$bundle_file"; then
  echo "expected watchdog proof consensus evidence schema marker" >&2
  exit 1
fi

if [ ! -L "$SCRIPT" ]; then
  echo "expected watchdog proof consensus wrapper to be a dispatcher symlink" >&2
  exit 1
fi

if [ "$(readlink "$SCRIPT")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected watchdog proof consensus wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected watchdog proof consensus wrapper to resolve runtime manifest via dispatcher" >&2
  exit 1
fi

if ! grep -Fq "watchdog_proof_consensus_contract_lane_contract.sh" "$MANIFEST_FILE"; then
  echo "expected watchdog proof consensus manifest to dispatch shared contract module" >&2
  exit 1
fi

echo "watchdog proof consensus contract lane script tests passed."
