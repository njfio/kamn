#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/ci/generate_performance_smoke_report.sh"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

SMOKE_REPORT="$TMP_DIR/smoke.json"
DEEP_REPORT="$TMP_DIR/deep.json"

bash "$SCRIPT" --lane smoke --output-json "$SMOKE_REPORT" >"$TMP_DIR/smoke.out"
grep -q '"lane": "smoke"' "$SMOKE_REPORT"
grep -q '"latency_p50_ms": 92' "$SMOKE_REPORT"
grep -q '"throughput_tps": 11250' "$SMOKE_REPORT"

bash "$SCRIPT" --lane deep --output-json "$DEEP_REPORT" >"$TMP_DIR/deep.out"
grep -q '"lane": "deep"' "$DEEP_REPORT"
grep -q '"latency_p99_ms": 340' "$DEEP_REPORT"
grep -q '"availability_pct": 99.95' "$DEEP_REPORT"

echo "performance smoke report generator tests passed."
