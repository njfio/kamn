#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/reputation/run_reputation_dispute_contract_lane.sh"
SHARED_CONTRACT="$ROOT_DIR/scripts/reputation/reputation_dispute_contract_lane_contract.py"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/reputation_dispute_contract_lane.json"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected reputation dispute contract lane script to be executable" >&2
  exit 1
fi

if [ ! -x "$SHARED_CONTRACT" ]; then
  echo "expected reputation dispute shared contract-lane module to be executable" >&2
  exit 1
fi
if [ ! -f "$MANIFEST" ]; then
  echo "expected reputation dispute contract lane manifest to exist" >&2
  exit 1
fi

if [ ! -L "$CONTRACT_LANE" ]; then
  echo "expected reputation dispute contract lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi
if [ "$(readlink "$CONTRACT_LANE")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected reputation dispute contract lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi
resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$CONTRACT_LANE")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST" ]; then
  echo "expected reputation dispute contract lane wrapper to resolve reputation manifest via dispatcher" >&2
  exit 1
fi
if ! grep -q "reputation_dispute_contract_lane_contract.py" "$MANIFEST"; then
  echo "expected reputation dispute manifest to dispatch to shared module" >&2
  exit 1
fi

if ! grep -q "generate_reputation_dispute_evidence_bundle.sh" "$SHARED_CONTRACT"; then
  echo "expected reputation dispute shared contract-lane module to run evidence generator" >&2
  exit 1
fi

if ! grep -q "check_reputation_dispute_policy.sh" "$SHARED_CONTRACT"; then
  echo "expected reputation dispute shared contract-lane module to run policy checker" >&2
  exit 1
fi

if ! grep -q "from framework.contract_lane_helpers import" "$SHARED_CONTRACT"; then
  echo "expected reputation dispute shared contract-lane module to use framework lane helpers" >&2
  exit 1
fi

bundle_file="$TMP_DIR/reputation-dispute-contract-bundle.json"
output="$(bash "$CONTRACT_LANE" --output-file "$bundle_file")"

if ! printf '%s\n' "$output" | grep -q "reputation dispute contract lane tests passed."; then
  echo "expected success output from reputation dispute contract lane" >&2
  exit 1
fi

if [ ! -f "$bundle_file" ]; then
  echo "expected reputation dispute contract lane to emit evidence bundle" >&2
  exit 1
fi

if ! grep -q '"schema_version": "kamn.reputation.dispute-evidence.v1"' "$bundle_file"; then
  echo "expected reputation dispute evidence schema marker" >&2
  exit 1
fi

if ! grep -q '"reason_key": "reputation_dispute_reason_codes:GO:v1"' "$bundle_file"; then
  echo "expected reputation dispute reason key marker in emitted bundle" >&2
  exit 1
fi

echo "reputation dispute contract lane script tests passed."
