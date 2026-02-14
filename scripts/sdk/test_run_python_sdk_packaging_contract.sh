#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/sdk/run_python_sdk_packaging_contract.sh"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$RUNNER" ]; then
  echo "expected python sdk packaging contract runner to be executable" >&2
  exit 1
fi

run_output="$(bash "$RUNNER" --output-json "$TMP_REPORT")"
if ! printf '%s\n' "$run_output" | grep -q '^status=pass$'; then
  echo "expected python sdk packaging contract pass status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^final_decision=GO$'; then
  echo "expected python sdk packaging contract GO decision marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^package_metadata_status=verified$'; then
  echo "expected python sdk packaging metadata marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^sdk_import_status=verified$'; then
  echo "expected python sdk import marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^packaging_contract_status=verified$'; then
  echo "expected python sdk packaging contract marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.sdk.python-packaging-contract.v1":
    raise SystemExit("unexpected python packaging schema")
if payload.get("status") != "pass":
    raise SystemExit("expected status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected final_decision=GO")
if payload.get("package_metadata_status") != "verified":
    raise SystemExit("expected package_metadata_status=verified")
if payload.get("sdk_import_status") != "verified":
    raise SystemExit("expected sdk_import_status=verified")
if payload.get("packaging_contract_status") != "verified":
    raise SystemExit("expected packaging_contract_status=verified")
PY

set +e
invalid_budget_output="$({ bash "$RUNNER" --max-seconds invalid; } 2>&1)"
invalid_budget_code=$?
set -e
if [ "$invalid_budget_code" -eq 0 ]; then
  echo "expected python sdk packaging contract runner to reject invalid max-seconds" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_budget_output" | grep -q 'max-seconds must be an integer'; then
  echo "expected deterministic invalid max-seconds marker" >&2
  exit 1
fi

echo "python sdk packaging contract runner tests passed."
