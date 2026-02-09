#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_SCRIPT="$ROOT_DIR/scripts/governance/run_quorum_attestation_replay_contract_lane.sh"
LANE_SCRIPT="$ROOT_DIR/scripts/governance/run_quorum_attestation_replay_guard_lane.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/governance/check_quorum_attestation_replay_policy.sh"

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

tmp_out="$(mktemp)"
trap 'rm -f "$tmp_out"' EXIT

bash "$CONTRACT_SCRIPT" >"$tmp_out"
if ! grep -q "governance quorum attestation replay contract lane tests passed." "$tmp_out"; then
  echo "expected governance quorum attestation contract lane success marker" >&2
  exit 1
fi

if ! grep -q "KAMN_GOVERNANCE_QUORUM_ATTESTATION_CONTRACT_MAX_SECONDS" "$CONTRACT_SCRIPT"; then
  echo "expected governance quorum attestation contract lane runtime guard env marker" >&2
  exit 1
fi
if ! grep -q "KAMN_GOVERNANCE_QUORUM_ATTESTATION_FORCE_REPLAY_DETECTED" "$CONTRACT_SCRIPT"; then
  echo "expected governance quorum attestation contract lane forced replay path" >&2
  exit 1
fi
if ! grep -q "reason_key mismatch" "$CONTRACT_SCRIPT"; then
  echo "expected governance quorum attestation contract lane to enforce reason_key drift failures" >&2
  exit 1
fi

echo "governance quorum attestation replay contract lane script tests passed."
