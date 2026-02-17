#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_WORKFLOW="$ROOT_DIR/.github/workflows/ci-fast-gate.yml"

if [[ ! -f "$FAST_WORKFLOW" ]]; then
  echo "expected ci-fast-gate workflow file: $FAST_WORKFLOW" >&2
  exit 1
fi

if ! grep -Fq "timeout-minutes: 20" "$FAST_WORKFLOW"; then
  echo "expected bounded fast-gate runtime budget (timeout-minutes: 20)" >&2
  exit 1
fi

if ! grep -Fq "name: Generate combined shell-surface trend report" "$FAST_WORKFLOW"; then
  echo "expected combined shell-surface trend report generation step in ci-fast-gate workflow" >&2
  exit 1
fi
if ! grep -Fq "bash scripts/ci/generate_combined_shell_surface_trend_report.sh" "$FAST_WORKFLOW"; then
  echo "expected combined shell-surface trend report generator command in ci-fast-gate workflow" >&2
  exit 1
fi
if ! grep -Fq "name: Check combined shell-surface trend policy" "$FAST_WORKFLOW"; then
  echo "expected combined shell-surface trend policy step in ci-fast-gate workflow" >&2
  exit 1
fi
if ! grep -Fq "bash scripts/ci/check_combined_shell_surface_trend_policy.sh" "$FAST_WORKFLOW"; then
  echo "expected combined shell-surface trend policy checker command in ci-fast-gate workflow" >&2
  exit 1
fi
if ! grep -Fq "fixtures/ci/combined_shell_surface_trend_thresholds.json" "$FAST_WORKFLOW"; then
  echo "expected combined shell-surface trend threshold fixture wiring in ci-fast-gate workflow" >&2
  exit 1
fi

if ! grep -Fq "name: Collect shell-rust LOC telemetry" "$FAST_WORKFLOW"; then
  echo "expected shell-rust LOC telemetry collector step in ci-fast-gate workflow" >&2
  exit 1
fi
if ! grep -Fq "bash scripts/ci/collect_shell_rust_loc_telemetry.sh" "$FAST_WORKFLOW"; then
  echo "expected shell-rust LOC telemetry collector command in ci-fast-gate workflow" >&2
  exit 1
fi
if ! grep -Fq "ci-shell-rust-loc-telemetry.json" "$FAST_WORKFLOW"; then
  echo "expected shell-rust LOC telemetry artifact/report path wiring in ci-fast-gate workflow" >&2
  exit 1
fi
if ! grep -Fq "ci-combined-shell-surface-trend-report.json" "$FAST_WORKFLOW"; then
  echo "expected combined shell-surface trend report output wiring in ci-fast-gate workflow" >&2
  exit 1
fi
if ! grep -Fq "ci-combined-shell-surface-trend-policy.json" "$FAST_WORKFLOW"; then
  echo "expected combined shell-surface trend policy output wiring in ci-fast-gate workflow" >&2
  exit 1
fi

scope_gate_count="$(grep -Fc "if: steps.scope.outputs.run_script_surface_budget_checks == 'true'" "$FAST_WORKFLOW")"
if (( scope_gate_count < 4 )); then
  echo "expected shell-surface ratio/budget checks to remain behind run_script_surface_budget_checks scope gate" >&2
  exit 1
fi

echo "fast-gate shell-surface ratio policy wiring tests passed."
