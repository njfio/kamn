#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"
DEEP_SCRIPT="$ROOT_DIR/scripts/bridge/run_bridge_replay_redaction_deep_lane.sh"
DEEP_IMPL_SCRIPT="$ROOT_DIR/scripts/bridge/run_bridge_replay_redaction_deep_lane_impl.sh"
DEEP_MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/bridge_bridge_replay_redaction_deep_lane.json"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"

test_harness_require_executable "$DEEP_SCRIPT" "expected bridge replay/redaction deep lane script to be executable"

test_harness_require_executable "$DEEP_IMPL_SCRIPT" "expected bridge replay/redaction deep lane implementation script to be executable"

test_harness_require_file "$DEEP_MANIFEST_FILE" "expected bridge replay/redaction deep lane manifest to exist"

if [ ! -L "$DEEP_SCRIPT" ]; then
  echo "expected bridge replay/redaction deep lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi

if [ "$(readlink "$DEEP_SCRIPT")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected bridge replay/redaction deep lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_deep_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$DEEP_SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_deep_manifest" != "$DEEP_MANIFEST_FILE" ]; then
  echo "expected bridge replay/redaction deep wrapper to resolve bridge deep manifest via dispatcher" >&2
  exit 1
fi

if ! grep -Fq "run_bridge_replay_redaction_deep_lane_impl.sh" "$DEEP_MANIFEST_FILE"; then
  echo "expected bridge replay/redaction deep lane manifest to dispatch implementation script" >&2
  exit 1
fi

if ! grep -Fq "run_bridge_replay_redaction_contract_lane.sh" "$DEEP_IMPL_SCRIPT"; then
  echo "expected bridge replay/redaction deep implementation lane to run contract lane baseline checks first" >&2
  exit 1
fi

if ! grep -q "bridge_adapter,telegram_bridge,discord_bridge,cross_chain_bridge" "$DEEP_IMPL_SCRIPT"; then
  echo "expected bridge replay/redaction deep implementation lane to run full bridge replay suite" >&2
  exit 1
fi

if ! grep -q -- "--output-json" "$DEEP_IMPL_SCRIPT"; then
  echo "expected bridge replay/redaction deep implementation lane to support output-json artifacts" >&2
  exit 1
fi

echo "bridge replay/redaction deep lane script tests passed."
