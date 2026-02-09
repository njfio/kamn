#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_SCRIPT="$ROOT_DIR/scripts/message/run_message_lifecycle_contract_lane.sh"
DEEP_SCRIPT="$ROOT_DIR/scripts/message/run_message_lifecycle_deep_lane.sh"

if [ ! -x "$FAST_SCRIPT" ]; then
  echo "expected message lifecycle fast-lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$DEEP_SCRIPT" ]; then
  echo "expected message lifecycle deep-lane runner to be executable" >&2
  exit 1
fi

TMP_OUT="$(mktemp)"
trap 'rm -f "$TMP_OUT"' EXIT

bash "$FAST_SCRIPT" >"$TMP_OUT"
if ! grep -q "message lifecycle snapshot contract lane tests passed." "$TMP_OUT"; then
  echo "expected message lifecycle contract lane success marker" >&2
  exit 1
fi

if ! grep -Fq "run_key_hierarchy_invariant_contract_lane.sh" "$FAST_SCRIPT"; then
  echo "expected message lifecycle lane to execute key hierarchy invariant lane checks" >&2
  exit 1
fi

if ! grep -Fq "run_group_sender_replay_ratchet_contract_lane.sh" "$FAST_SCRIPT"; then
  echo "expected message lifecycle lane to execute group sender replay/ratchet lane checks" >&2
  exit 1
fi

if ! grep -Fq "run_didcomm_envelope_compatibility_contract_lane.sh" "$FAST_SCRIPT"; then
  echo "expected message lifecycle lane to execute DIDComm envelope compatibility lane checks" >&2
  exit 1
fi

if ! grep -Fq "run_message_lifecycle_contract_lane.sh" "$DEEP_SCRIPT"; then
  echo "expected deep-lane script to execute message lifecycle fast-lane checks first" >&2
  exit 1
fi

if ! grep -q "performance_message_lifecycle_snapshot_deep_lane_stress -- --ignored" "$DEEP_SCRIPT"; then
  echo "expected deep-lane script to run ignored snapshot deep-lane stress test" >&2
  exit 1
fi

echo "message lifecycle contract lane script tests passed."
