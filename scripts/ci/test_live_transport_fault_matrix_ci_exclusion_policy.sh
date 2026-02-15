#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_WORKFLOW="$ROOT_DIR/.github/workflows/ci-fast-gate.yml"
CI_TOOLS_SCRIPT="$ROOT_DIR/scripts/ci/test_ci_tools.sh"
STRATEGY_DOC="$ROOT_DIR/docs/ci/strategy.md"

if grep -Fq "bash scripts/runtime/validate_live_transport_fault_matrix_live.sh --mode run" "$FAST_WORKFLOW"; then
  echo "expected live transport fault matrix run-mode lane to remain excluded from ci-fast-gate.yml" >&2
  exit 1
fi

fast_mode_block="$(
  awk '
    BEGIN { in_fast_mode = 0 }
    /if \[ "\$\{KAMN_CI_TOOLS_FAST_MODE:-false\}" = "true" \]; then/ { in_fast_mode = 1; next }
    /^  exit 0$/ && in_fast_mode == 1 { in_fast_mode = 0; next }
    in_fast_mode == 1 { print }
  ' "$CI_TOOLS_SCRIPT"
)"

if printf '%s\n' "$fast_mode_block" | grep -Fq 'bash "$ROOT_DIR/scripts/runtime/test_validate_live_transport_fault_matrix_live_contract_lane.sh"'; then
  echo "expected live transport fault matrix contract lane to remain excluded from ci-tools fast mode" >&2
  exit 1
fi

for required_command in \
  'bash "$ROOT_DIR/scripts/runtime/test_check_live_transport_fault_matrix_live_policy.sh"' \
  'bash "$ROOT_DIR/scripts/runtime/test_validate_live_transport_fault_matrix_live_contract_lane.sh"'; do
  if ! grep -Fq "$required_command" "$CI_TOOLS_SCRIPT"; then
    echo "expected ci-tools regression lane to include live transport fault matrix command: $required_command" >&2
    exit 1
  fi
done

if ! grep -Fq "live transport fault matrix run-mode commands remain excluded from ci-fast-gate and ci-tools fast mode." "$STRATEGY_DOC"; then
  echo "expected CI strategy docs to include live transport fault matrix run-mode exclusion marker" >&2
  exit 1
fi

echo "live transport fault matrix CI exclusion policy tests passed."
