#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATE_SCRIPT="$ROOT_DIR/scripts/ci/generate_performance_smoke_report.sh"
CHECK_SCRIPT="$ROOT_DIR/scripts/ci/check_performance_thresholds.sh"
FIXTURE_MATRIX="$ROOT_DIR/fixtures/ci/performance_hot_path_fixture_matrix.json"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

RUNTIME_REPORT="$TMP_DIR/runtime-smoke.json"

bash "$GENERATE_SCRIPT" \
  --lane smoke \
  --workload runtime \
  --fixture-file "$FIXTURE_MATRIX" \
  --output-json "$RUNTIME_REPORT" >"$TMP_DIR/generate.out"

bash "$CHECK_SCRIPT" \
  --lane smoke \
  --report-json "$RUNTIME_REPORT" >"$TMP_DIR/check-pass.out"
grep -q '^status=pass; lane=smoke;' "$TMP_DIR/check-pass.out"

MISSING_PROVENANCE_REPORT="$TMP_DIR/runtime-smoke-missing-provenance.json"
cp "$RUNTIME_REPORT" "$MISSING_PROVENANCE_REPORT"
python3 - "$MISSING_PROVENANCE_REPORT" <<'PY'
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload.pop("baseline_provenance_artifact_version", None)
payload.pop("drift_threshold_seed_id", None)
path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

if bash "$CHECK_SCRIPT" \
  --lane smoke \
  --report-json "$MISSING_PROVENANCE_REPORT" >"$TMP_DIR/check-missing-provenance.out" 2>&1; then
  echo "expected checker to fail when baseline provenance/seed markers are missing" >&2
  exit 1
fi
grep -q 'missing required baseline marker: baseline_provenance_artifact_version' "$TMP_DIR/check-missing-provenance.out"

echo "performance threshold checker tests passed."
