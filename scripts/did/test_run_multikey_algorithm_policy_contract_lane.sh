#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/did/run_multikey_algorithm_policy_contract_lane.sh"
SHARED_CONTRACT="$ROOT_DIR/scripts/did/multikey_algorithm_policy_contract_lane_contract.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/did_multikey_algorithm_policy_contract_lane.json"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$SCRIPT" ]; then
  echo "expected multikey algorithm policy contract lane script to be executable" >&2
  exit 1
fi
if [ ! -x "$SHARED_CONTRACT" ]; then
  echo "expected multikey algorithm policy shared contract-lane module to be executable" >&2
  exit 1
fi

if ! grep -q "generate_multikey_algorithm_policy_evidence_bundle.sh" "$SHARED_CONTRACT"; then
  echo "expected multikey algorithm policy shared contract module to execute evidence bundle generator" >&2
  exit 1
fi

if ! grep -q "check_multikey_algorithm_policy.sh" "$SHARED_CONTRACT"; then
  echo "expected multikey algorithm policy shared contract module to execute policy checker" >&2
  exit 1
fi

bundle_file="$TMP_DIR/did-multikey-algorithm-policy-contract-bundle.json"
lane_output="$(bash "$SCRIPT" --skip-tests --output-file "$bundle_file")"

if ! printf '%s\n' "$lane_output" | grep -q "multikey algorithm policy contract lane tests passed."; then
  echo "expected multikey algorithm policy contract lane success marker" >&2
  exit 1
fi

if [ ! -f "$bundle_file" ]; then
  echo "expected multikey algorithm policy contract lane to emit evidence bundle" >&2
  exit 1
fi

if ! grep -q '"schema_version": "kamn.did.multikey-algorithm-policy-report.v1"' "$bundle_file"; then
  echo "expected multikey algorithm policy evidence schema marker" >&2
  exit 1
fi

if [ ! -L "$SCRIPT" ]; then
  echo "expected multikey algorithm policy contract lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi
if [ "$(readlink "$SCRIPT")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected multikey algorithm policy contract lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi
resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected multikey algorithm policy wrapper to resolve did multikey manifest via dispatcher" >&2
  exit 1
fi
if ! grep -Fq "multikey_algorithm_policy_contract_lane_contract.sh" "$MANIFEST_FILE"; then
  echo "expected multikey algorithm policy manifest to dispatch shared contract module" >&2
  exit 1
fi

echo "multikey algorithm policy contract lane script tests passed."
