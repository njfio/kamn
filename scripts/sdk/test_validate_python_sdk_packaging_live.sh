#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/sdk/validate_python_sdk_packaging_live.sh"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected python sdk packaging live validation script to be executable" >&2
  exit 1
fi

validation_output="$(bash "$VALIDATION_SCRIPT" --output-json "$TMP_REPORT")"
if ! printf '%s\n' "$validation_output" | grep -q '^status=pass$'; then
  echo "expected python sdk packaging live pass status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^final_decision=GO$'; then
  echo "expected python sdk packaging live GO decision marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^packaging_contract_status=verified$'; then
  echo "expected python sdk packaging live packaging contract marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^evidence_bundle_status=verified$'; then
  echo "expected python sdk packaging live evidence bundle marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^fail_closed_status=verified$'; then
  echo "expected python sdk packaging live fail-closed marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^fail_closed_reason_code=missing_pyproject$'; then
  echo "expected python sdk packaging live fail-closed reason marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.sdk.python-packaging-live-validation.v1":
    raise SystemExit("unexpected python sdk packaging live schema")
if payload.get("status") != "pass":
    raise SystemExit("expected live status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected live final_decision=GO")
if payload.get("packaging_contract_status") != "verified":
    raise SystemExit("expected packaging_contract_status=verified")
if payload.get("evidence_bundle_status") != "verified":
    raise SystemExit("expected evidence_bundle_status=verified")
if payload.get("fail_closed_status") != "verified":
    raise SystemExit("expected fail_closed_status=verified")
if payload.get("fail_closed_reason_code") != "missing_pyproject":
    raise SystemExit("expected fail_closed_reason_code=missing_pyproject")
PY

set +e
invalid_budget_output="$({ bash "$VALIDATION_SCRIPT" --max-seconds invalid; } 2>&1)"
invalid_budget_code=$?
set -e
if [ "$invalid_budget_code" -eq 0 ]; then
  echo "expected python sdk packaging live validation script to reject invalid max-seconds" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_budget_output" | grep -q 'max-seconds must be an integer'; then
  echo "expected deterministic invalid max-seconds marker" >&2
  exit 1
fi

echo "python sdk packaging live validation tests passed."
