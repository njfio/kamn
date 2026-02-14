#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_service_api_request_auth_live.sh"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected request-auth live validation script to be executable" >&2
  exit 1
fi

validation_output="$(bash "$VALIDATION_SCRIPT" --output-json "$TMP_REPORT")"
if ! printf '%s\n' "$validation_output" | grep -q '^status=pass$'; then
  echo "expected request-auth live validation pass status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^final_decision=GO$'; then
  echo "expected request-auth live validation GO decision marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^unauthorized_guard_status=verified$'; then
  echo "expected unauthorized guard marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^replay_guard_status=verified$'; then
  echo "expected replay guard marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^probe_status=verified$'; then
  echo "expected probe status marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.service-api-request-auth-live-validation.v1":
    raise SystemExit("unexpected request-auth live validation schema")
if payload.get("status") != "pass":
    raise SystemExit("expected request-auth live validation status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected request-auth live validation final_decision=GO")
if payload.get("unauthorized_guard_status") != "verified":
    raise SystemExit("expected unauthorized_guard_status=verified")
if payload.get("replay_guard_status") != "verified":
    raise SystemExit("expected replay_guard_status=verified")
if payload.get("probe_status") != "verified":
    raise SystemExit("expected probe_status=verified")
PY

echo "service api request-auth live validation tests passed."
