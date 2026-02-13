#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/message/run_group_sender_replay_ratchet_contract_lane.sh"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"
CONTRACT_MODULE="$ROOT_DIR/scripts/message/group_sender_replay_ratchet_contract_lane_contract.sh"
EXPECTED_MANIFEST="$ROOT_DIR/scripts/framework/manifests/message_group_sender_replay_ratchet_contract_lane.json"

if [ ! -x "$SCRIPT" ]; then
  echo "expected group sender replay/ratchet contract lane script to be executable" >&2
  exit 1
fi

if [ ! -x "$DISPATCHER" ]; then
  echo "expected non-Kolme dispatcher to be executable: $DISPATCHER" >&2
  exit 1
fi

if [ ! -x "$CONTRACT_MODULE" ]; then
  echo "expected shared group sender replay/ratchet contract module to be executable: $CONTRACT_MODULE" >&2
  exit 1
fi

if [ ! -L "$SCRIPT" ]; then
  echo "expected group sender replay/ratchet wrapper to be a symlink: $SCRIPT" >&2
  exit 1
fi

manifest_path="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$SCRIPT")" --resolve-manifest-path)"
if [ "$manifest_path" != "$EXPECTED_MANIFEST" ]; then
  echo "expected dispatcher to resolve $EXPECTED_MANIFEST but found $manifest_path" >&2
  exit 1
fi

if ! grep -Fq "\"scripts/message/$(basename "$CONTRACT_MODULE")\"" "$manifest_path"; then
  echo "expected manifest to dispatch shared group sender replay/ratchet contract module: $manifest_path" >&2
  exit 1
fi

echo "group sender replay/ratchet contract lane wrapper tests passed."
