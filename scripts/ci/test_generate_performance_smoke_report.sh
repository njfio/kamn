#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/ci/generate_performance_smoke_report.sh"
FIXTURE_MATRIX="$ROOT_DIR/fixtures/ci/performance_hot_path_fixture_matrix.json"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

RUNTIME_SMOKE_REPORT="$TMP_DIR/runtime-smoke.json"
SIGNING_SMOKE_REPORT="$TMP_DIR/signing-smoke.json"
TRANSPORT_DEEP_REPORT="$TMP_DIR/transport-deep.json"

bash "$SCRIPT" \
  --lane smoke \
  --workload runtime \
  --fixture-file "$FIXTURE_MATRIX" \
  --output-json "$RUNTIME_SMOKE_REPORT" >"$TMP_DIR/runtime-smoke.out"
grep -q '"lane": "smoke"' "$RUNTIME_SMOKE_REPORT"
grep -q '"workload": "runtime"' "$RUNTIME_SMOKE_REPORT"
grep -q '"latency_p50_ms": 92' "$RUNTIME_SMOKE_REPORT"
grep -q '"throughput_tps": 11250' "$RUNTIME_SMOKE_REPORT"

bash "$SCRIPT" \
  --lane smoke \
  --workload signing \
  --fixture-file "$FIXTURE_MATRIX" \
  --output-json "$SIGNING_SMOKE_REPORT" >"$TMP_DIR/signing-smoke.out"
grep -q '"workload": "signing"' "$SIGNING_SMOKE_REPORT"
grep -q '"latency_p50_ms": 58' "$SIGNING_SMOKE_REPORT"
grep -q '"throughput_tps": 15100' "$SIGNING_SMOKE_REPORT"

bash "$SCRIPT" \
  --lane deep \
  --workload transport \
  --fixture-file "$FIXTURE_MATRIX" \
  --output-json "$TRANSPORT_DEEP_REPORT" >"$TMP_DIR/transport-deep.out"
grep -q '"lane": "deep"' "$TRANSPORT_DEEP_REPORT"
grep -q '"workload": "transport"' "$TRANSPORT_DEEP_REPORT"
grep -q '"latency_p99_ms": 430' "$TRANSPORT_DEEP_REPORT"
grep -q '"availability_pct": 99.9' "$TRANSPORT_DEEP_REPORT"

if bash "$SCRIPT" \
  --lane smoke \
  --workload unknown \
  --fixture-file "$FIXTURE_MATRIX" \
  --output-json "$TMP_DIR/unknown.json" >"$TMP_DIR/unknown.out" 2>&1; then
  echo "expected generator to fail for unknown workload" >&2
  exit 1
fi
grep -q "Unknown workload" "$TMP_DIR/unknown.out"

MUTATED_FIXTURE="$TMP_DIR/mutated-fixture.json"
cp "$FIXTURE_MATRIX" "$MUTATED_FIXTURE"
python3 - "$MUTATED_FIXTURE" <<'PY'
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["schema_version"] = "kamn.ci.performance-hot-path-matrix.v0"
path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

if bash "$SCRIPT" \
  --lane smoke \
  --workload runtime \
  --fixture-file "$MUTATED_FIXTURE" \
  --output-json "$TMP_DIR/mutated.json" >"$TMP_DIR/mutated.out" 2>&1; then
  echo "expected generator to fail for mutated fixture schema" >&2
  exit 1
fi
grep -q "fixture schema version mismatch" "$TMP_DIR/mutated.out"

echo "performance smoke report generator tests passed."
