#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/did/run_lifecycle_operator_binding_contract_lane.sh"
SHARED_CONTRACT="$ROOT_DIR/scripts/did/lifecycle_operator_binding_contract_lane_contract.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/did_lifecycle_operator_binding_contract_lane.json"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$SCRIPT" ]; then
  echo "expected lifecycle operator-binding contract lane script to be executable" >&2
  exit 1
fi
if [ ! -x "$SHARED_CONTRACT" ]; then
  echo "expected lifecycle operator-binding shared contract-lane module to be executable" >&2
  exit 1
fi

if ! grep -q "generate_lifecycle_operator_binding_evidence_bundle.sh" "$SHARED_CONTRACT"; then
  echo "expected lifecycle operator-binding shared contract module to execute evidence bundle generator" >&2
  exit 1
fi

if ! grep -q "check_lifecycle_operator_binding_policy.sh" "$SHARED_CONTRACT"; then
  echo "expected lifecycle operator-binding shared contract module to execute policy checker" >&2
  exit 1
fi

bundle_file="$TMP_DIR/lifecycle-operator-binding-contract-bundle.json"
output="$(bash "$SCRIPT" --output-file "$bundle_file" --skip-tests)"

if ! printf '%s\n' "$output" | grep -q "lifecycle operator-binding contract lane tests passed."; then
  echo "expected success output from lifecycle operator-binding contract lane" >&2
  exit 1
fi

if [ ! -f "$bundle_file" ]; then
  echo "expected lifecycle operator-binding contract lane to emit evidence bundle" >&2
  exit 1
fi

if ! grep -q '"schema_version": "kamn.did.lifecycle-operator-binding.v1"' "$bundle_file"; then
  echo "expected lifecycle operator-binding evidence schema marker" >&2
  exit 1
fi

if ! grep -q '"reason_key": "did_lifecycle_operator_binding_reason_codes:GO:v1"' "$bundle_file"; then
  echo "expected lifecycle operator-binding reason key marker in emitted bundle" >&2
  exit 1
fi

if [ ! -L "$SCRIPT" ]; then
  echo "expected lifecycle operator-binding contract lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi
if [ "$(readlink "$SCRIPT")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected lifecycle operator-binding contract lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi
resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected lifecycle operator-binding wrapper to resolve did lifecycle manifest via dispatcher" >&2
  exit 1
fi
if ! grep -Fq "lifecycle_operator_binding_contract_lane_contract.sh" "$MANIFEST_FILE"; then
  echo "expected lifecycle operator-binding manifest to dispatch shared contract module" >&2
  exit 1
fi

echo "lifecycle operator-binding contract lane script tests passed."
