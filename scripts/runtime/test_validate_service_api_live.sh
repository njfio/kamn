#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_service_api_live.sh"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected service api live validation script to be executable" >&2
  exit 1
fi

validation_output="$(bash "$VALIDATION_SCRIPT" --output-json "$TMP_REPORT")"
if ! printf '%s\n' "$validation_output" | grep -q '^status=pass$'; then
  echo "expected service api live validation pass status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^final_decision=GO$'; then
  echo "expected service api live validation GO decision marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^route_contract_status=verified$'; then
  echo "expected service api route contract marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^failure_case_status=verified$'; then
  echo "expected service api fail-closed marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.service-api-live-validation.v1":
    raise SystemExit("unexpected service api live validation schema")
if payload.get("status") != "pass":
    raise SystemExit("expected service api live validation status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected service api live validation final_decision=GO")
if payload.get("route_contract_status") != "verified":
    raise SystemExit("expected route_contract_status=verified")
if payload.get("failure_case_status") != "verified":
    raise SystemExit("expected failure_case_status=verified")
if not payload.get("message_id"):
    raise SystemExit("expected non-empty message_id")
if not payload.get("channel_id"):
    raise SystemExit("expected non-empty channel_id")
if not payload.get("task_id"):
    raise SystemExit("expected non-empty task_id")
PY

echo "service api live validation tests passed."
