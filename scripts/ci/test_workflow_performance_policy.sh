#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_WORKFLOW="$ROOT_DIR/.github/workflows/ci-fast-gate.yml"
DEEP_WORKFLOW="$ROOT_DIR/.github/workflows/ci-deep-validate.yml"

if ! grep -q "Generate performance smoke report" "$FAST_WORKFLOW"; then
  echo "expected performance smoke report generation step in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -q "generate_performance_smoke_report.sh --lane smoke --output-json performance-smoke-report.json" "$FAST_WORKFLOW"; then
  echo "expected smoke performance report command in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -q "Check performance thresholds (smoke)" "$FAST_WORKFLOW"; then
  echo "expected smoke performance threshold step in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -q "check_performance_thresholds.sh --lane smoke --report-json performance-smoke-report.json --profile-file .ci/performance-targets.env" "$FAST_WORKFLOW"; then
  echo "expected smoke threshold check command in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -q "Generate fast-gate budget delta report" "$FAST_WORKFLOW"; then
  echo "expected fast-gate budget delta generation step in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -q "generate_fast_gate_budget_delta_report.sh --current-json ci-budget-fast-gate.json --baseline-file .ci/fast-gate-budget-delta.env --output-json ci-budget-fast-gate-delta.json" "$FAST_WORKFLOW"; then
  echo "expected fast-gate budget delta generation command in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -q "Check fast-gate budget delta thresholds" "$FAST_WORKFLOW"; then
  echo "expected fast-gate budget delta threshold step in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -q "check_fast_gate_budget_delta_threshold.sh --report-json ci-budget-fast-gate-delta.json --threshold-file .ci/fast-gate-budget-delta.env --waiver-file .ci/fast-gate-budget-delta-waiver.json" "$FAST_WORKFLOW"; then
  echo "expected fast-gate budget delta threshold command in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -q "Upload fast-gate budget delta telemetry" "$FAST_WORKFLOW"; then
  echo "expected fast-gate budget delta artifact upload step in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -q "ci-budget-fast-gate-delta-\${{ github.run_id }}-\${{ github.run_attempt }}" "$FAST_WORKFLOW"; then
  echo "expected fast-gate budget delta artifact naming contract in ci-fast-gate.yml" >&2
  exit 1
fi

if ! grep -q "Generate performance smoke report" "$DEEP_WORKFLOW"; then
  echo "expected performance smoke report generation step in ci-deep-validate.yml" >&2
  exit 1
fi

if ! grep -q "generate_performance_smoke_report.sh --lane deep --output-json performance-deep-report.json" "$DEEP_WORKFLOW"; then
  echo "expected deep performance report command in ci-deep-validate.yml" >&2
  exit 1
fi

if ! grep -q "Check performance thresholds (deep)" "$DEEP_WORKFLOW"; then
  echo "expected deep performance threshold step in ci-deep-validate.yml" >&2
  exit 1
fi

if ! grep -q "check_performance_thresholds.sh --lane deep --report-json performance-deep-report.json --profile-file .ci/performance-targets.env" "$DEEP_WORKFLOW"; then
  echo "expected deep threshold check command in ci-deep-validate.yml" >&2
  exit 1
fi

echo "workflow performance policy tests passed."
