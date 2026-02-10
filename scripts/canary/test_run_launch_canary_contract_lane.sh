#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_SCRIPT="$ROOT_DIR/scripts/canary/run_launch_canary_contract_lane.sh"
DEEP_SCRIPT="$ROOT_DIR/scripts/canary/run_launch_canary_deep_lane.sh"
SHARED_CONTRACT="$ROOT_DIR/scripts/canary/launch_canary_contract_lane_contract.py"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/canary_launch_canary_contract_lane.json"

if [ ! -x "$FAST_SCRIPT" ]; then
  echo "expected launch canary fast-lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$DEEP_SCRIPT" ]; then
  echo "expected launch canary deep-lane runner to be executable" >&2
  exit 1
fi
if [ ! -x "$SHARED_CONTRACT" ]; then
  echo "expected launch canary shared contract-lane module to be executable" >&2
  exit 1
fi
if [ ! -f "$MANIFEST" ]; then
  echo "expected launch canary contract-lane manifest to exist" >&2
  exit 1
fi

tmp_out="$(mktemp)"
trap 'rm -f "$tmp_out"' EXIT

bash "$FAST_SCRIPT" >"$tmp_out"
if ! grep -q "launch canary contract lane tests passed." "$tmp_out"; then
  echo "expected launch canary contract lane success marker" >&2
  exit 1
fi
if ! grep -q "run_manifest_lane.sh" "$FAST_SCRIPT"; then
  echo "expected launch canary fast-lane wrapper to delegate via manifest runner" >&2
  exit 1
fi
if ! grep -q "canary_launch_canary_contract_lane.json" "$FAST_SCRIPT"; then
  echo "expected launch canary fast-lane wrapper to reference launch canary manifest" >&2
  exit 1
fi
if ! grep -q "launch_canary_contract_lane_contract.py" "$MANIFEST"; then
  echo "expected launch canary manifest to dispatch to shared contract module" >&2
  exit 1
fi
if ! grep -q "run_launch_canary_matrix.py" "$SHARED_CONTRACT"; then
  echo "expected launch canary shared contract-lane module to run matrix script" >&2
  exit 1
fi
if ! grep -q "critical_path_probe_cases.json" "$SHARED_CONTRACT"; then
  echo "expected launch canary shared contract-lane module to use critical path probe fixture" >&2
  exit 1
fi
if ! grep -q "missing_probe_evidence" "$SHARED_CONTRACT"; then
  echo "expected launch canary shared contract-lane module to enforce missing_probe_evidence regression case" >&2
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
