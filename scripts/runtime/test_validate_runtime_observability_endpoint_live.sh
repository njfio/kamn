#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_runtime_observability_endpoint_live.sh"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected runtime observability endpoint live validation script to be executable" >&2
  exit 1
fi

validation_output="$(bash "$VALIDATION_SCRIPT" --output-json "$TMP_REPORT")"
if ! printf '%s\n' "$validation_output" | grep -q '^status=pass$'; then
  echo "expected runtime observability endpoint live validation pass status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^final_decision=GO$'; then
  echo "expected runtime observability endpoint live validation GO decision marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^runtime_observability_stream_contract_status=verified$'; then
  echo "expected runtime observability stream contract marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^fail_closed_status=verified$'; then
  echo "expected fail-closed status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^docs_contract_status=verified$'; then
  echo "expected docs contract status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^performance_budget_status=verified$'; then
  echo "expected performance budget marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.observability-endpoint-live-validation.v1":
    raise SystemExit("unexpected runtime observability endpoint live schema")
if payload.get("runtime_observability_stream_contract_status") != "verified":
    raise SystemExit("expected runtime_observability_stream_contract_status=verified")
if payload.get("fail_closed_status") != "verified":
    raise SystemExit("expected fail_closed_status=verified")
if payload.get("docs_contract_status") != "verified":
    raise SystemExit("expected docs_contract_status=verified")
if payload.get("performance_budget_status") != "verified":
    raise SystemExit("expected performance_budget_status=verified")
PY

set +e
invalid_budget_output="$(
  bash "$VALIDATION_SCRIPT" \
    --max-seconds invalid 2>&1
)"
invalid_budget_code=$?
set -e
if [ "$invalid_budget_code" -eq 0 ]; then
  echo "expected runtime observability endpoint live validation to reject invalid max-seconds" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_budget_output" | grep -q 'max-seconds must be an integer'; then
  echo "expected deterministic invalid max-seconds marker" >&2
  exit 1
fi

echo "runtime observability endpoint live validation tests passed."
