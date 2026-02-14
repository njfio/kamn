#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_live_validation_environment_live.sh"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected live validation environment live script to be executable" >&2
  exit 1
fi

validation_output="$(bash "$VALIDATION_SCRIPT" --output-json "$TMP_REPORT")"
if ! printf '%s\n' "$validation_output" | grep -q '^status=pass$'; then
  echo "expected live validation environment live pass status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^final_decision=GO$'; then
  echo "expected live validation environment live GO decision marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^lane_contract_status=verified$'; then
  echo "expected live validation environment live lane contract marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^evidence_bundle_status=verified$'; then
  echo "expected live validation environment live evidence bundle marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^fail_closed_status=verified$'; then
  echo "expected live validation environment live fail-closed marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.live-validation-environment-live-validation.v1":
    raise SystemExit("unexpected live validation environment live schema")
if payload.get("status") != "pass":
    raise SystemExit("expected live validation environment live status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected live validation environment live final_decision=GO")
if payload.get("lane_contract_status") != "verified":
    raise SystemExit("expected lane_contract_status=verified")
if payload.get("evidence_bundle_status") != "verified":
    raise SystemExit("expected evidence_bundle_status=verified")
if payload.get("fail_closed_status") != "verified":
    raise SystemExit("expected fail_closed_status=verified")
PY

set +e
invalid_budget_output="$(
  bash "$VALIDATION_SCRIPT" \
    --max-seconds invalid 2>&1
)"
invalid_budget_code=$?
set -e
if [ "$invalid_budget_code" -eq 0 ]; then
  echo "expected live validation environment live script to reject invalid max-seconds" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_budget_output" | grep -q 'max-seconds must be an integer'; then
  echo "expected deterministic invalid max-seconds marker for live validation environment live script" >&2
  exit 1
fi

echo "live validation environment live validation tests passed."
