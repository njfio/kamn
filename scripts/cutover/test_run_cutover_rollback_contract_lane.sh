#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_SCRIPT="$ROOT_DIR/scripts/cutover/run_cutover_rollback_contract_lane.sh"
DEEP_SCRIPT="$ROOT_DIR/scripts/cutover/run_cutover_rollback_deep_lane.sh"

if [ ! -x "$FAST_SCRIPT" ]; then
  echo "expected cutover rollback fast-lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$DEEP_SCRIPT" ]; then
  echo "expected cutover rollback deep-lane runner to be executable" >&2
  exit 1
fi

tmp_out="$(mktemp)"
trap 'rm -f "$tmp_out"' EXIT

bash "$FAST_SCRIPT" >"$tmp_out"
if ! grep -q "cutover rollback contract lane tests passed." "$tmp_out"; then
  echo "expected cutover rollback contract lane success marker" >&2
  exit 1
fi

if ! grep -Fq "run_cutover_rollback_contract_lane.sh" "$DEEP_SCRIPT"; then
  echo "expected rollback deep-lane script to execute fast-lane contract checks first" >&2
  exit 1
fi

if ! grep -q "final_decision=NO-GO" "$DEEP_SCRIPT"; then
  echo "expected rollback deep-lane script to validate NO-GO decision path" >&2
  exit 1
fi

echo "cutover rollback contract lane script tests passed."
