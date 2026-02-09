#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_SCRIPT="$ROOT_DIR/scripts/cutover/run_mainnet_cutover_contract_lane.sh"

if [ ! -x "$FAST_SCRIPT" ]; then
  echo "expected mainnet cutover contract lane runner to be executable" >&2
  exit 1
fi

tmp_out="$(mktemp)"
trap 'rm -f "$tmp_out"' EXIT

bash "$FAST_SCRIPT" >"$tmp_out"
if ! grep -q "mainnet cutover contract lane tests passed." "$tmp_out"; then
  echo "expected mainnet cutover contract lane success marker" >&2
  exit 1
fi

if ! grep -Fq "validate_mainnet_cutover_manifest.py" "$FAST_SCRIPT"; then
  echo "expected contract lane to run mainnet cutover validator" >&2
  exit 1
fi

if ! grep -Fq "mainnet_cutover_manifest.invalid_dependency.json" "$FAST_SCRIPT"; then
  echo "expected contract lane to validate dependency regression fixture" >&2
  exit 1
fi

if ! grep -Fq "mainnet_cutover_manifest.invalid_approvals.json" "$FAST_SCRIPT"; then
  echo "expected contract lane to validate approval regression fixture" >&2
  exit 1
fi

echo "mainnet cutover contract lane script tests passed."
