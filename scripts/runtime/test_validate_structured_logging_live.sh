#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_structured_logging_live.sh"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected structured logging live validation script to be executable" >&2
  exit 1
fi

validation_output="$(bash "$VALIDATION_SCRIPT" --output-json "$TMP_REPORT")"
if ! printf '%s\n' "$validation_output" | grep -q '^status=pass$'; then
  echo "expected structured logging live validation pass status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^final_decision=GO$'; then
  echo "expected structured logging live validation GO decision marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^structured_logging_contract_status=verified$'; then
  echo "expected structured logging contract status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^correlation_contract_status=verified$'; then
  echo "expected correlation contract status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^correlation_id_parity_status=verified$'; then
  echo "expected correlation id parity status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^trace_classification_contract_status=verified$'; then
  echo "expected trace classification contract status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^log_classification_gate_status=verified$'; then
  echo "expected log classification gate status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^reason_taxonomy_version=kamn.runtime.structured-logging-live-fail-closed-reason-taxonomy.v1$'; then
  echo "expected structured logging reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^correlation_error_reason_taxonomy_version=kamn.runtime.correlation-error-reason-taxonomy.v1$'; then
  echo "expected correlation error reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^correlation_error_reason_codes_csv=correlation_id_missing,correlation_id_mismatch,trace_classification_unmapped$'; then
  echo "expected deterministic correlation error reason codes taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^docs_contract_status=verified$'; then
  echo "expected docs contract status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^fail_closed_status=verified$'; then
  echo "expected fail-closed status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^performance_budget_status=verified$'; then
  echo "expected performance budget status marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.structured-logging-live-validation.v1":
    raise SystemExit("unexpected structured logging live validation schema")
if payload.get("status") != "pass":
    raise SystemExit("expected status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected final_decision=GO")
if payload.get("structured_logging_contract_status") != "verified":
    raise SystemExit("expected structured_logging_contract_status=verified")
if payload.get("correlation_contract_status") != "verified":
    raise SystemExit("expected correlation_contract_status=verified")
if payload.get("correlation_id_parity_status") != "verified":
    raise SystemExit("expected correlation_id_parity_status=verified")
if payload.get("trace_classification_contract_status") != "verified":
    raise SystemExit("expected trace_classification_contract_status=verified")
if payload.get("log_classification_gate_status") != "verified":
    raise SystemExit("expected log_classification_gate_status=verified")
if payload.get("reason_taxonomy_version") != "kamn.runtime.structured-logging-live-fail-closed-reason-taxonomy.v1":
    raise SystemExit("expected deterministic reason_taxonomy_version marker")
if payload.get("correlation_error_reason_taxonomy_version") != "kamn.runtime.correlation-error-reason-taxonomy.v1":
    raise SystemExit("expected deterministic correlation_error_reason_taxonomy_version marker")
if payload.get("correlation_error_reason_codes_csv") != "correlation_id_missing,correlation_id_mismatch,trace_classification_unmapped":
    raise SystemExit("expected deterministic correlation_error_reason_codes_csv marker")
if payload.get("docs_contract_status") != "verified":
    raise SystemExit("expected docs_contract_status=verified")
if payload.get("fail_closed_status") != "verified":
    raise SystemExit("expected fail_closed_status=verified")
if payload.get("performance_budget_status") != "verified":
    raise SystemExit("expected performance_budget_status=verified")
if payload.get("fail_closed_reason_code") != "invalid_log_config_level":
    raise SystemExit("expected deterministic fail_closed_reason_code marker")
PY

set +e
invalid_budget_output="$(
  bash "$VALIDATION_SCRIPT" \
    --max-seconds invalid 2>&1
)"
invalid_budget_code=$?
set -e
if [ "$invalid_budget_code" -eq 0 ]; then
  echo "expected structured logging live validation script to reject invalid max-seconds" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_budget_output" | grep -q 'max-seconds must be an integer'; then
  echo "expected deterministic invalid max-seconds marker" >&2
  exit 1
fi

echo "structured logging live validation tests passed."
