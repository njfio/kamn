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
REASON_TAXONOMY_VERSION="kamn.runtime.observability-endpoint-reason-taxonomy.v1"
REASON_CODES_CSV="runtime_observability_endpoint_readiness_progress_stalled,runtime_observability_stream_parity_bypass_detected,ci_local_observability_endpoint_budget_boundary_exceeded"

start_epoch="$(date +%s)"

pushd "$ROOT_DIR" >/dev/null
cargo test -p kamn-node functional_observability_endpoint_renders_stream_payload
cargo test -p kamn-node integration_runtime_observability_endpoint_serves_stream_path
cargo test -p kamn-node integration_runtime_observability_endpoint_serves_metrics_and_health_paths
cargo test -p kamn-node functional_observability_endpoint_projects_readiness_reason_code_parity_across_endpoint_surfaces
cargo test -p kamn-node integration_runtime_observability_endpoint_returns_not_found_for_unknown_path
cargo test -p kamn-node integration_runtime_observability_endpoint_returns_not_found_for_malformed_request_method
cargo test -p kamn-node integration_runtime_observability_endpoint_fails_closed_on_idle_timeout
popd >/dev/null

if ! grep -q "Runtime Endpoint Stream Contract (Issue #3047)" "$OBSERVABILITY_DOC"; then
  echo "expected observability doc to include runtime endpoint stream contract section" >&2
  exit 1
fi
if ! grep -q "validate_runtime_observability_endpoint_live.sh" "$OBSERVABILITY_DOC"; then
  echo "expected observability doc to reference validate_runtime_observability_endpoint_live.sh" >&2
  exit 1
fi
if ! grep -q "test_validate_runtime_observability_endpoint_live.sh" "$OBSERVABILITY_DOC"; then
  echo "expected observability doc to reference test_validate_runtime_observability_endpoint_live.sh" >&2
  exit 1
fi
if ! grep -q "Task #3047, Subtask #3048" "$ROADMAP_DOC"; then
  echo "expected roadmap to include Task #3047, Subtask #3048 marker" >&2
  exit 1
fi
if ! grep -q "scripts/runtime/validate_runtime_observability_endpoint_live.sh" "$ROADMAP_DOC"; then
  echo "expected roadmap to reference runtime observability endpoint live validation lane command" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "runtime observability endpoint live validation exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

report_json="$(mktemp)"
cat >"$report_json" <<JSON
{
  "schema_version": "kamn.runtime.observability-endpoint-live-validation.v1",
  "status": "pass",
  "final_decision": "GO",
  "runtime_observability_stream_contract_status": "verified",
  "endpoint_readiness_status": "verified",
  "stream_parity_status": "verified",
  "unknown_path_contract_status": "verified",
  "malformed_input_contract_status": "verified",
  "timeout_contract_status": "verified",
  "reason_taxonomy_version": "${REASON_TAXONOMY_VERSION}",
  "reason_codes_csv": "${REASON_CODES_CSV}",
  "ci_local_budget_boundary_status": "verified",
  "fail_closed_status": "verified",
  "docs_contract_status": "verified",
  "fail_closed_reason_code": "observability_endpoint_not_found",
  "fail_closed_reason_codes_csv": "observability_endpoint_not_found,observability_endpoint_malformed_request,observability_endpoint_idle_timeout",
  "performance_budget_status": "verified",
  "elapsed_seconds": ${elapsed_seconds},
  "max_seconds": ${max_seconds}
}
JSON

if [[ -n "$output_json" ]]; then
  cp "$report_json" "$output_json"
fi
rm -f "$report_json"

echo "status=pass"
echo "final_decision=GO"
echo "runtime_observability_stream_contract_status=verified"
echo "endpoint_readiness_status=verified"
echo "stream_parity_status=verified"
echo "unknown_path_contract_status=verified"
echo "malformed_input_contract_status=verified"
echo "timeout_contract_status=verified"
echo "reason_taxonomy_version=${REASON_TAXONOMY_VERSION}"
echo "reason_codes_csv=${REASON_CODES_CSV}"
echo "ci_local_budget_boundary_status=verified"
echo "fail_closed_status=verified"
echo "docs_contract_status=verified"
echo "fail_closed_reason_code=observability_endpoint_not_found"
echo "fail_closed_reason_codes_csv=observability_endpoint_not_found,observability_endpoint_malformed_request,observability_endpoint_idle_timeout"
echo "performance_budget_status=verified"
