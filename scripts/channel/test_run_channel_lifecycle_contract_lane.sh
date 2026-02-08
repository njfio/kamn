#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_SCRIPT="$ROOT_DIR/scripts/channel/run_channel_lifecycle_contract_lane.sh"
DEEP_SCRIPT="$ROOT_DIR/scripts/channel/run_channel_lifecycle_deep_lane.sh"

if [ ! -x "$FAST_SCRIPT" ]; then
  echo "expected channel lifecycle fast-lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$DEEP_SCRIPT" ]; then
  echo "expected channel lifecycle deep-lane runner to be executable" >&2
  exit 1
fi

TMP_OUT="$(mktemp)"
trap 'rm -f "$TMP_OUT"' EXIT

bash "$FAST_SCRIPT" >"$TMP_OUT"
if ! grep -q "channel lifecycle snapshot contract lane tests passed." "$TMP_OUT"; then
  echo "expected channel lifecycle contract lane success marker" >&2
  exit 1
fi

if ! grep -Fq "run_channel_lifecycle_contract_lane.sh" "$DEEP_SCRIPT"; then
  echo "expected deep-lane script to execute channel lifecycle fast-lane checks first" >&2
  exit 1
fi

if ! grep -q "performance_channel_snapshot_deep_lane_stress -- --ignored" "$DEEP_SCRIPT"; then
  echo "expected deep-lane script to run ignored channel snapshot stress test" >&2
  exit 1
fi

echo "channel lifecycle contract lane script tests passed."
