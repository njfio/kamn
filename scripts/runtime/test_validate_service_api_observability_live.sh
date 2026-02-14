#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_service_api_observability_live.sh"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected service api observability live validation script to be executable" >&2
  exit 1
fi

validation_output="$(bash "$VALIDATION_SCRIPT" --output-json "$TMP_REPORT")"
if ! printf '%s\n' "$validation_output" | grep -q '^status=pass$'; then
  echo "expected service api observability live validation pass status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^final_decision=GO$'; then
  echo "expected service api observability live validation GO decision marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^metrics_contract_status=verified$'; then
  echo "expected observability metrics contract marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^health_contract_status=verified$'; then
  echo "expected observability health contract marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^fail_closed_status=verified$'; then
  echo "expected observability fail-closed marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.service-api-observability-live-validation.v1":
    raise SystemExit("unexpected service api observability validation schema")
if payload.get("status") != "pass":
    raise SystemExit("expected status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected final_decision=GO")
if payload.get("metrics_contract_status") != "verified":
    raise SystemExit("expected metrics_contract_status=verified")
if payload.get("health_contract_status") != "verified":
    raise SystemExit("expected health_contract_status=verified")
if payload.get("fail_closed_status") != "verified":
    raise SystemExit("expected fail_closed_status=verified")
PY

set +e
invalid_arg_output="$(bash "$VALIDATION_SCRIPT" --max-seconds nope 2>&1)"
invalid_arg_code=$?
set -e
if [ "$invalid_arg_code" -eq 0 ]; then
  echo "expected invalid --max-seconds to fail closed" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_arg_output" | grep -q 'max-seconds must be an integer'; then
  printf '%s\n' "$invalid_arg_output" >&2
  echo "expected invalid --max-seconds reason marker" >&2
  exit 1
fi

echo "service api observability live validation tests passed."
