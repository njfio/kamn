#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_async_runtime_live.sh"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected async runtime live validation lane script to be executable" >&2
  exit 1
fi

lane_output="$(bash "$VALIDATION_SCRIPT" --output-json "$TMP_REPORT")"
if ! printf '%s\n' "$lane_output" | grep -q '^status=pass$'; then
  echo "expected async runtime live validation lane pass status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^final_decision=GO$'; then
  echo "expected async runtime live validation lane GO decision marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^runtime_entrypoint=tokio-main$'; then
  echo "expected async runtime live validation lane tokio entrypoint marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^failure_case_status=verified$'; then
  echo "expected async runtime live validation lane fail-closed drill marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.async-runtime-live-validation.v1":
    raise SystemExit("unexpected async runtime live validation schema")
if payload.get("status") != "pass":
    raise SystemExit("expected async runtime live validation status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected async runtime live validation final_decision=GO")
if payload.get("runtime_entrypoint") != "tokio-main":
    raise SystemExit("expected tokio-main runtime entrypoint marker")
if payload.get("failure_case_status") != "verified":
    raise SystemExit("expected failure_case_status=verified")
PY

echo "async runtime live validation lane tests passed."
