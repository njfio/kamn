#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_go_no_go_gate_live.sh"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected go/no-go gate live validation script to be executable" >&2
  exit 1
fi

validation_output="$(bash "$VALIDATION_SCRIPT" --output-json "$TMP_REPORT")"
if ! printf '%s\n' "$validation_output" | grep -q '^status=pass$'; then
  echo "expected go/no-go gate live validation pass status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^final_decision=GO$'; then
  echo "expected go/no-go gate live validation GO decision marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^baseline_contract_status=verified$'; then
  echo "expected go/no-go gate live validation baseline marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^fault_injection_status=verified$'; then
  echo "expected go/no-go gate live validation fault injection marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^fail_closed_status=verified$'; then
  echo "expected go/no-go gate live validation fail-closed marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.go-no-go-gate-live-validation.v1":
    raise SystemExit("unexpected go/no-go gate live validation schema")
if payload.get("status") != "pass":
    raise SystemExit("expected go/no-go gate live validation status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected go/no-go gate live validation final_decision=GO")
if payload.get("baseline_contract_status") != "verified":
    raise SystemExit("expected baseline_contract_status=verified")
if payload.get("fault_injection_status") != "verified":
    raise SystemExit("expected fault_injection_status=verified")
if payload.get("fail_closed_status") != "verified":
    raise SystemExit("expected fail_closed_status=verified")
PY

set +e
invalid_budget_output="$(
  bash "$VALIDATION_SCRIPT" --max-seconds nope 2>&1
)"
invalid_budget_code=$?
set -e
if [ "$invalid_budget_code" -eq 0 ]; then
  echo "expected go/no-go gate live validation script to reject invalid max-seconds" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_budget_output" | grep -q 'max-seconds must be an integer'; then
  echo "expected deterministic invalid max-seconds marker for go/no-go gate live validation script" >&2
  exit 1
fi

echo "go/no-go gate live validation tests passed."
