#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_config_layering_live.sh"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected config layering live validation script to be executable" >&2
  exit 1
fi

validation_output="$(bash "$VALIDATION_SCRIPT" --output-json "$TMP_REPORT")"
if ! printf '%s\n' "$validation_output" | grep -q '^status=pass$'; then
  echo "expected config layering live pass status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^final_decision=GO$'; then
  echo "expected config layering live GO decision marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^layering_contract_status=verified$'; then
  echo "expected config layering live layering contract marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^precedence_contract_status=verified$'; then
  echo "expected config layering live precedence contract marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^fail_closed_status=verified$'; then
  echo "expected config layering live fail-closed marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^fail_closed_reason_code=invalid_sync_mode_override$'; then
  echo "expected config layering live fail-closed reason marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.config-layering-live-validation.v1":
    raise SystemExit("unexpected config layering live schema")
if payload.get("status") != "pass":
    raise SystemExit("expected config layering live status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected config layering live final_decision=GO")
if payload.get("layering_contract_status") != "verified":
    raise SystemExit("expected layering_contract_status=verified")
if payload.get("precedence_contract_status") != "verified":
    raise SystemExit("expected precedence_contract_status=verified")
if payload.get("fail_closed_status") != "verified":
    raise SystemExit("expected fail_closed_status=verified")
if payload.get("fail_closed_reason_code") != "invalid_sync_mode_override":
    raise SystemExit("expected fail_closed_reason_code=invalid_sync_mode_override")
PY

set +e
invalid_budget_output="$({ bash "$VALIDATION_SCRIPT" --max-seconds invalid; } 2>&1)"
invalid_budget_code=$?
set -e
if [ "$invalid_budget_code" -eq 0 ]; then
  echo "expected config layering live script to reject invalid max-seconds" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_budget_output" | grep -q 'max-seconds must be an integer'; then
  echo "expected deterministic invalid max-seconds marker for config layering live script" >&2
  exit 1
fi

echo "config layering live validation tests passed."
