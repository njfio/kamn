#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/kolme/check_lane_migration_matrix_policy.py"
MATRIX_FIXTURE="$ROOT_DIR/fixtures/kolme_compatibility/lane_migration_matrix.json"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$SCRIPT" ]; then
  echo "expected lane migration matrix policy checker to be executable" >&2
  exit 1
fi

if [ ! -f "$MATRIX_FIXTURE" ]; then
  echo "expected lane migration matrix fixture to exist" >&2
  exit 1
fi

python3 "$SCRIPT" \
  --matrix-file "$MATRIX_FIXTURE"

python3 - "$MATRIX_FIXTURE" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
lanes = payload.get("lanes", [])
heavy_lane = None
for lane in lanes:
    if isinstance(lane, dict) and lane.get("lane_id") == "kolme.local.heavy.validation_matrix":
        heavy_lane = lane
        break

if heavy_lane is None:
    raise SystemExit("expected lane migration matrix to include kolme.local.heavy.validation_matrix")
if heavy_lane.get("status") != "migrated":
    raise SystemExit("expected heavy validation matrix lane status migrated")
if heavy_lane.get("current_runner") != "framework_manifest_lane":
    raise SystemExit("expected heavy validation matrix lane current_runner framework_manifest_lane")
PY

MUTATED_MATRIX="$TMP_DIR/mutated-missing-lane.json"
cp "$MATRIX_FIXTURE" "$MUTATED_MATRIX"
python3 - "$MUTATED_MATRIX" <<'PY'
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["lanes"] = [
    lane for lane in payload.get("lanes", [])
    if lane.get("lane_id") != "kolme.local.fork.rust_matrix"
]
path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

if python3 "$SCRIPT" --matrix-file "$MUTATED_MATRIX" >"$TMP_DIR/mutated.out" 2>"$TMP_DIR/mutated.err"; then
  echo "expected policy checker failure when required lane is removed" >&2
  cat "$TMP_DIR/mutated.out" >&2 || true
  cat "$TMP_DIR/mutated.err" >&2 || true
  exit 1
fi

INVALID_PRIORITY_MATRIX="$TMP_DIR/mutated-invalid-priority.json"
cp "$MATRIX_FIXTURE" "$INVALID_PRIORITY_MATRIX"
python3 - "$INVALID_PRIORITY_MATRIX" <<'PY'
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["lanes"][0]["priority"] = "PX"
path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

if python3 "$SCRIPT" --matrix-file "$INVALID_PRIORITY_MATRIX" >"$TMP_DIR/invalid-priority.out" 2>"$TMP_DIR/invalid-priority.err"; then
  echo "expected policy checker failure for invalid priority" >&2
  cat "$TMP_DIR/invalid-priority.out" >&2 || true
  cat "$TMP_DIR/invalid-priority.err" >&2 || true
  exit 1
fi

echo "lane migration matrix policy checker tests passed."
