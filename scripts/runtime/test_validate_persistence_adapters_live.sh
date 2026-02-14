#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_persistence_adapters_live.sh"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected persistence adapter live validation script to be executable" >&2
  exit 1
fi

validation_output="$(bash "$VALIDATION_SCRIPT" --output-json "$TMP_REPORT")"
if ! printf '%s\n' "$validation_output" | grep -q '^status=pass$'; then
  echo "expected persistence adapter live validation pass status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^final_decision=GO$'; then
  echo "expected persistence adapter live validation GO decision marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^content_persistence_status=verified$'; then
  echo "expected persistence adapter live validation content marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^did_duplicate_detection_status=verified$'; then
  echo "expected persistence adapter live validation did marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^fail_closed_status=verified$'; then
  echo "expected persistence adapter live validation fail-closed marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.persistence.adapters-live-validation.v1":
    raise SystemExit("unexpected persistence adapter live validation schema")
if payload.get("status") != "pass":
    raise SystemExit("expected persistence adapter live validation status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected persistence adapter live validation final_decision=GO")
if payload.get("content_persistence_status") != "verified":
    raise SystemExit("expected content_persistence_status=verified")
if payload.get("did_duplicate_detection_status") != "verified":
    raise SystemExit("expected did_duplicate_detection_status=verified")
if payload.get("fail_closed_status") != "verified":
    raise SystemExit("expected fail_closed_status=verified")
PY

echo "persistence adapter live validation tests passed."
