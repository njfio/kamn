#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_SCRIPT="$ROOT_DIR/scripts/canary/run_post_cutover_slo_contract_lane.sh"
DEEP_SCRIPT="$ROOT_DIR/scripts/canary/run_post_cutover_slo_deep_lane.sh"

if [ ! -x "$FAST_SCRIPT" ]; then
  echo "expected post-cutover SLO fast-lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$DEEP_SCRIPT" ]; then
  echo "expected post-cutover SLO deep-lane runner to be executable" >&2
  exit 1
fi

tmp_out="$(mktemp)"
trap 'rm -f "$tmp_out"' EXIT

bash "$FAST_SCRIPT" >"$tmp_out"
if ! grep -q "post-cutover SLO contract lane tests passed." "$tmp_out"; then
  echo "expected post-cutover SLO contract lane success marker" >&2
  exit 1
fi

if ! grep -Fq "run_post_cutover_slo_contract_lane.sh" "$DEEP_SCRIPT"; then
  echo "expected SLO deep-lane script to execute fast-lane checks first" >&2
  exit 1
fi

if ! grep -q "final_decision=NO-GO" "$DEEP_SCRIPT"; then
  echo "expected SLO deep-lane script to validate NO-GO decision path" >&2
  exit 1
fi

echo "post-cutover SLO contract lane script tests passed."
