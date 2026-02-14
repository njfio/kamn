#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/deploy/validate_deployment_assets_live.sh"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected deployment assets live validation script to be executable" >&2
  exit 1
fi

validation_output="$(bash "$VALIDATION_SCRIPT" --output-json "$TMP_REPORT")"
if ! printf '%s\n' "$validation_output" | grep -q '^status=pass$'; then
  echo "expected deployment assets live validation status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^final_decision=GO$'; then
  echo "expected deployment assets live validation GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^asset_contract_status=verified$'; then
  echo "expected deployment asset contract marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^fail_closed_status=verified$'; then
  echo "expected deployment fail-closed marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.deploy.assets.live-validation.v1":
    raise SystemExit("unexpected deployment assets live validation schema")
if payload.get("status") != "pass":
    raise SystemExit("expected status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected final_decision=GO")
if payload.get("asset_contract_status") != "verified":
    raise SystemExit("expected asset_contract_status=verified")
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

echo "deployment assets live validation tests passed."
