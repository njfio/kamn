#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_SCRIPT="$ROOT_DIR/scripts/canary/run_launch_canary_contract_lane.sh"
DEEP_SCRIPT="$ROOT_DIR/scripts/canary/run_launch_canary_deep_lane.sh"

if [ ! -x "$FAST_SCRIPT" ]; then
  echo "expected launch canary fast-lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$DEEP_SCRIPT" ]; then
  echo "expected launch canary deep-lane runner to be executable" >&2
  exit 1
fi

tmp_out="$(mktemp)"
trap 'rm -f "$tmp_out"' EXIT

bash "$FAST_SCRIPT" >"$tmp_out"
if ! grep -q "launch canary contract lane tests passed." "$tmp_out"; then
  echo "expected launch canary contract lane success marker" >&2
  exit 1
fi

if ! grep -Fq "run_launch_canary_contract_lane.sh" "$DEEP_SCRIPT"; then
  echo "expected canary deep-lane script to execute fast-lane checks first" >&2
  exit 1
fi

if ! grep -Fq "critical_path_probe_cases.json" "$DEEP_SCRIPT"; then
  echo "expected canary deep-lane script to use critical path probe fixture" >&2
  exit 1
fi

echo "launch canary contract lane script tests passed."
