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
if ! printf '%s\n' "$validation_output" | grep -q '^endpoint_readiness_status=verified$'; then
  echo "expected endpoint readiness status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^stream_parity_status=verified$'; then
  echo "expected stream parity status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^unknown_path_contract_status=verified$'; then
  echo "expected unknown-path contract marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^malformed_input_contract_status=verified$'; then
  echo "expected malformed-input contract marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^timeout_contract_status=verified$'; then
  echo "expected timeout contract marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^observability_tls_route_contract_status=verified$'; then
  echo "expected observability TLS route contract marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^reason_taxonomy_version=kamn.runtime.observability-endpoint-reason-taxonomy.v1$'; then
  echo "expected reason taxonomy version marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^reason_codes_csv=runtime_observability_endpoint_readiness_progress_stalled,runtime_observability_stream_parity_bypass_detected,ci_local_observability_endpoint_budget_boundary_exceeded$'; then
  echo "expected reason codes taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^ci_local_budget_boundary_status=verified$'; then
  echo "expected ci-local budget boundary marker" >&2
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
if payload.get("endpoint_readiness_status") != "verified":
    raise SystemExit("expected endpoint_readiness_status=verified")
if payload.get("stream_parity_status") != "verified":
    raise SystemExit("expected stream_parity_status=verified")
if payload.get("unknown_path_contract_status") != "verified":
    raise SystemExit("expected unknown_path_contract_status=verified")
if payload.get("malformed_input_contract_status") != "verified":
    raise SystemExit("expected malformed_input_contract_status=verified")
if payload.get("timeout_contract_status") != "verified":
    raise SystemExit("expected timeout_contract_status=verified")
if payload.get("observability_tls_route_contract_status") != "verified":
    raise SystemExit("expected observability_tls_route_contract_status=verified")
if payload.get("reason_taxonomy_version") != "kamn.runtime.observability-endpoint-reason-taxonomy.v1":
    raise SystemExit("expected deterministic reason_taxonomy_version marker")
if payload.get("reason_codes_csv") != "runtime_observability_endpoint_readiness_progress_stalled,runtime_observability_stream_parity_bypass_detected,ci_local_observability_endpoint_budget_boundary_exceeded":
    raise SystemExit("expected deterministic reason_codes_csv marker")
if payload.get("ci_local_budget_boundary_status") != "verified":
    raise SystemExit("expected ci_local_budget_boundary_status=verified")
if payload.get("fail_closed_status") != "verified":
    raise SystemExit("expected fail_closed_status=verified")
if payload.get("docs_contract_status") != "verified":
    raise SystemExit("expected docs_contract_status=verified")
if payload.get("performance_budget_status") != "verified":
    raise SystemExit("expected performance_budget_status=verified")
if payload.get("fail_closed_reason_codes_csv") != "observability_endpoint_not_found,observability_endpoint_malformed_request,observability_endpoint_idle_timeout":
    raise SystemExit("expected deterministic fail_closed_reason_codes_csv taxonomy")
if payload.get("max_seconds") != 120:
    raise SystemExit("expected max_seconds=120 in validation report")
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
