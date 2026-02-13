#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_SCRIPT="$ROOT_DIR/scripts/message/run_message_lifecycle_contract_lane.sh"
DEEP_SCRIPT="$ROOT_DIR/scripts/message/run_message_lifecycle_deep_lane.sh"
SHARED_CONTRACT="$ROOT_DIR/scripts/message/message_lifecycle_contract_lane_contract.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/message_message_lifecycle_contract_lane.json"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"

if [ ! -x "$FAST_SCRIPT" ]; then
  echo "expected message lifecycle fast-lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$DEEP_SCRIPT" ]; then
  echo "expected message lifecycle deep-lane runner to be executable" >&2
  exit 1
fi
if [ ! -x "$SHARED_CONTRACT" ]; then
  echo "expected message lifecycle shared contract-lane module to be executable" >&2
  exit 1
fi

TMP_OUT="$(mktemp)"
trap 'rm -f "$TMP_OUT"' EXIT

bash "$FAST_SCRIPT" >"$TMP_OUT"
if ! grep -q "message lifecycle snapshot contract lane tests passed." "$TMP_OUT"; then
  echo "expected message lifecycle contract lane success marker" >&2
  exit 1
fi

if [ ! -L "$FAST_SCRIPT" ]; then
  echo "expected message lifecycle contract lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi

if [ "$(readlink "$FAST_SCRIPT")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected message lifecycle contract lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$FAST_SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected message lifecycle wrapper to resolve message manifest via dispatcher" >&2
  exit 1
fi

if ! grep -Fq "message_lifecycle_contract_lane_contract.sh" "$MANIFEST_FILE"; then
  echo "expected message lifecycle manifest to dispatch shared contract module" >&2
  exit 1
fi

if ! grep -Fq "run_key_hierarchy_invariant_contract_lane.sh" "$SHARED_CONTRACT"; then
  echo "expected message lifecycle shared contract module to execute key hierarchy invariant lane checks" >&2
  exit 1
fi

if ! grep -Fq "run_group_sender_replay_ratchet_contract_lane.sh" "$SHARED_CONTRACT"; then
  echo "expected message lifecycle shared contract module to execute group sender replay/ratchet lane checks" >&2
  exit 1
fi

if ! grep -Fq "run_processor_proof_artifact_contract_lane.sh" "$SHARED_CONTRACT"; then
  echo "expected message lifecycle shared contract module to execute processor proof artifact lane checks" >&2
  exit 1
fi

if ! grep -Fq "run_didcomm_envelope_compatibility_contract_lane.sh" "$SHARED_CONTRACT"; then
  echo "expected message lifecycle shared contract module to execute DIDComm envelope compatibility lane checks" >&2
  exit 1
fi

if ! grep -Fq "run_a2a_mcp_conformance_contract_lane.sh" "$SHARED_CONTRACT"; then
  echo "expected message lifecycle shared contract module to execute A2A/MCP conformance lane checks" >&2
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
