#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/sdk/run_live_transport_replay_tamper_deep_lane.sh"
SCRIPT_IMPL="$ROOT_DIR/scripts/sdk/run_live_transport_replay_tamper_deep_lane_impl.sh"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/sdk_live_transport_replay_tamper_deep_lane.json"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$SCRIPT" ]; then
  echo "expected live transport replay/tamper deep lane script to be executable" >&2
  exit 1
fi
if [ ! -x "$SCRIPT_IMPL" ]; then
  echo "expected live transport replay/tamper deep lane implementation to be executable" >&2
  exit 1
fi
if [ ! -x "$DISPATCHER" ]; then
  echo "expected shared non-Kolme dispatcher to be executable" >&2
  exit 1
fi

if [ ! -L "$SCRIPT" ]; then
  echo "expected replay/tamper deep lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi

if [ "$(readlink "$SCRIPT")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected replay/tamper deep lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected replay/tamper deep lane wrapper to resolve sdk manifest via dispatcher" >&2
  exit 1
fi

if ! grep -q 'run_live_transport_replay_tamper_deep_lane_impl.sh' "$MANIFEST_FILE"; then
  echo "expected replay/tamper deep lane manifest to dispatch implementation module" >&2
  exit 1
fi

if ! grep -q 'run_live_transport_replay_tamper_contract_lane.sh' "$SCRIPT_IMPL"; then
  echo "expected replay/tamper deep lane implementation to delegate to contract lane script" >&2
  exit 1
fi

report_file="$TMP_DIR/live-transport-replay-tamper-deep-report.json"
output="$(bash "$SCRIPT" --output-report "$report_file")"

if ! printf '%s\n' "$output" | grep -q 'lane_mode=deep'; then
  echo "expected replay/tamper deep lane mode marker" >&2
  exit 1
fi

if ! printf '%s\n' "$output" | grep -q 'deep_no_go_status=verified'; then
  echo "expected replay/tamper deep lane NO-GO verification marker" >&2
  exit 1
fi

if ! printf '%s\n' "$output" | grep -q 'final_decision=GO'; then
  echo "expected replay/tamper deep lane final decision marker" >&2
  exit 1
fi

if ! grep -q '"schema_version": "kamn.sdk.live-transport-replay-tamper-evidence.v1"' "$report_file"; then
  echo "expected replay/tamper deep lane report schema marker" >&2
  exit 1
fi

echo "live transport replay/tamper deep lane tests passed."
