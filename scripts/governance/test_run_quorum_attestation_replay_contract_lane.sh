#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_SCRIPT="$ROOT_DIR/scripts/governance/run_quorum_attestation_replay_contract_lane.sh"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"
LANE_SCRIPT="$ROOT_DIR/scripts/governance/run_quorum_attestation_replay_guard_lane.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/governance/check_quorum_attestation_replay_policy.sh"
SHARED_CONTRACT="$ROOT_DIR/scripts/governance/governance_quorum_attestation_replay_contract_lane_contract.py"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/governance_quorum_attestation_replay_contract_lane.json"

if [ ! -x "$CONTRACT_SCRIPT" ]; then
  echo "expected governance quorum attestation contract lane script to be executable" >&2
  exit 1
fi
if [ ! -x "$LANE_SCRIPT" ]; then
  echo "expected governance quorum attestation lane script to be executable" >&2
  exit 1
fi
if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected governance quorum attestation policy checker script to be executable" >&2
  exit 1
fi
if [ ! -x "$SHARED_CONTRACT" ]; then
  echo "expected governance quorum attestation shared contract-lane module to be executable" >&2
  exit 1
fi
if [ ! -f "$MANIFEST" ]; then
  echo "expected governance quorum attestation manifest to exist" >&2
  exit 1
fi

tmp_out="$(mktemp)"
trap 'rm -f "$tmp_out"' EXIT

bash "$CONTRACT_SCRIPT" >"$tmp_out"
if ! grep -q "governance quorum attestation replay contract lane tests passed." "$tmp_out"; then
  echo "expected governance quorum attestation contract lane success marker" >&2
  exit 1
fi

if ! grep -q "run_manifest_lane.sh" "$CONTRACT_SCRIPT"; then
  echo "expected governance quorum attestation contract lane wrapper to dispatch via manifest runner" >&2
  exit 1
fi
if [ ! -x "$DISPATCHER" ]; then
  echo "expected shared non-Kolme dispatcher to be executable" >&2
  exit 1
fi
resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$CONTRACT_SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST" ]; then
  echo "expected governance quorum attestation contract lane wrapper to resolve quorum manifest via dispatcher" >&2
  exit 1
fi
if ! grep -q '"wrapper_name": "run_quorum_attestation_replay_contract_lane.sh"' "$MANIFEST"; then
  echo "expected governance quorum attestation manifest wrapper_name metadata marker" >&2
  exit 1
fi
if ! grep -q '"phase": "contract"' "$MANIFEST"; then
  echo "expected governance quorum attestation manifest phase metadata marker" >&2
  exit 1
fi
if ! grep -q "governance_quorum_attestation_replay_contract_lane_contract.py" "$MANIFEST"; then
  echo "expected governance quorum attestation manifest to dispatch to shared module" >&2
  exit 1
fi
if ! grep -q "KAMN_GOVERNANCE_QUORUM_ATTESTATION_CONTRACT_MAX_SECONDS" "$SHARED_CONTRACT"; then
  echo "expected governance quorum attestation contract lane runtime guard env marker" >&2
  exit 1
fi
if ! grep -q "KAMN_GOVERNANCE_QUORUM_ATTESTATION_FORCE_REPLAY_DETECTED" "$SHARED_CONTRACT"; then
  echo "expected governance quorum attestation contract lane forced replay path" >&2
  exit 1
fi
if ! grep -q "reason_key mismatch" "$SHARED_CONTRACT"; then
  echo "expected governance quorum attestation contract lane to enforce reason_key drift failures" >&2
  exit 1
fi

echo "governance quorum attestation replay contract lane script tests passed."
