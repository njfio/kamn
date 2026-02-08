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
