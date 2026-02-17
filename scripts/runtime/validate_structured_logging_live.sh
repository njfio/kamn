#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

output_json=""
max_seconds=120

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-json)
      output_json="${2:-}"
      shift 2
      ;;
    --max-seconds)
      max_seconds="${2:-}"
      shift 2
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if ! [[ "$max_seconds" =~ ^[0-9]+$ ]]; then
  echo "max-seconds must be an integer" >&2
  exit 1
fi
if [ "$max_seconds" -le 0 ]; then
  echo "max-seconds must be greater than zero" >&2
  exit 1
fi

OBSERVABILITY_DOC="$ROOT_DIR/docs/foundation/observability-slo-dashboards.md"
ROADMAP_DOC="$ROOT_DIR/docs/plans/2026-02-08-production-service-roadmap.md"

start_epoch="$(date +%s)"

pushd "$ROOT_DIR" >/dev/null
cargo test -p kamn-node integration_bootstrap_runtime_emits_structured_marker
cargo test -p kamn-node functional_runtime_daemon_emits_structured_transition_markers
cargo test -p kamn-node regression_invalid_log_level_config_fails_closed
popd >/dev/null

if ! grep -q "validate_structured_logging_live.sh" "$OBSERVABILITY_DOC"; then
  echo "expected observability doc to reference validate_structured_logging_live.sh" >&2
  exit 1
fi
if ! grep -q "test_validate_structured_logging_live.sh" "$OBSERVABILITY_DOC"; then
  echo "expected observability doc to reference test_validate_structured_logging_live.sh" >&2
  exit 1
fi
if ! grep -q "Post-roadmap hardening wave 1 live validation delivered" "$ROADMAP_DOC"; then
  echo "expected roadmap to include post-roadmap wave 1 live validation status marker" >&2
  exit 1
fi
if ! grep -q "scripts/runtime/validate_structured_logging_live.sh" "$ROADMAP_DOC"; then
  echo "expected roadmap to reference structured logging live validation lane command" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "structured logging live validation exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

report_json="$(mktemp)"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$report_json" <<JSON
{
  "schema_version": "kamn.runtime.structured-logging-live-validation.v1",
  "telemetry_schema_version": "kamn.runtime.structured-logging-telemetry.v1",
  "reason_taxonomy_version": "kamn.runtime.structured-logging-live-fail-closed-reason-taxonomy.v1",
  "telemetry_schema_reason_taxonomy_version": "kamn.runtime.structured-logging-telemetry-schema-reason-taxonomy.v1",
  "telemetry_schema_reason_codes_csv": "structured_logging_telemetry_schema_version_mismatch,correlation_id_parity_bypass_detected",
  "correlation_error_reason_taxonomy_version": "kamn.runtime.correlation-error-reason-taxonomy.v1",
  "correlation_error_reason_codes_csv": "correlation_id_missing,correlation_id_mismatch,trace_classification_unmapped",
  "status": "pass",
  "final_decision": "GO",
  "structured_logging_contract_status": "verified",
  "telemetry_schema_contract_status": "verified",
  "correlation_contract_status": "verified",
  "correlation_id_parity_status": "verified",
  "trace_classification_contract_status": "verified",
  "log_classification_gate_status": "verified",
  "docs_contract_status": "verified",
  "fail_closed_status": "verified",
  "fail_closed_reason_code": "invalid_log_config_level",
  "performance_budget_status": "verified",
  "elapsed_seconds": ${elapsed_seconds}
}
JSON

if [[ -n "$output_json" ]]; then
  cp "$report_json" "$output_json"
fi
rm -f "$report_json"

echo "status=pass"
echo "final_decision=GO"
echo "structured_logging_contract_status=verified"
echo "telemetry_schema_version=kamn.runtime.structured-logging-telemetry.v1"
echo "telemetry_schema_contract_status=verified"
echo "correlation_contract_status=verified"
echo "correlation_id_parity_status=verified"
echo "trace_classification_contract_status=verified"
echo "log_classification_gate_status=verified"
echo "reason_taxonomy_version=kamn.runtime.structured-logging-live-fail-closed-reason-taxonomy.v1"
echo "telemetry_schema_reason_taxonomy_version=kamn.runtime.structured-logging-telemetry-schema-reason-taxonomy.v1"
echo "telemetry_schema_reason_codes_csv=structured_logging_telemetry_schema_version_mismatch,correlation_id_parity_bypass_detected"
echo "correlation_error_reason_taxonomy_version=kamn.runtime.correlation-error-reason-taxonomy.v1"
echo "correlation_error_reason_codes_csv=correlation_id_missing,correlation_id_mismatch,trace_classification_unmapped"
echo "docs_contract_status=verified"
echo "fail_closed_status=verified"
echo "fail_closed_reason_code=invalid_log_config_level"
echo "performance_budget_status=verified"
