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

ARCH_DOC="$ROOT_DIR/docs/architecture/block-pipeline.md"
ROADMAP_DOC="$ROOT_DIR/docs/plans/2026-02-08-production-service-roadmap.md"

start_epoch="$(date +%s)"

pushd "$ROOT_DIR" >/dev/null
cargo test -p kamn-core --test block_pipeline
cargo test -p kamn-core --test block_pipeline_docs
cargo test -p kamn-core --test block_pipeline regression_block_pipeline_rejects_payload_digest_mismatch_before_commit -- --exact
popd >/dev/null

if ! grep -q "validate_block_pipeline_live.sh" "$ARCH_DOC"; then
  echo "expected architecture doc to reference validate_block_pipeline_live.sh" >&2
  exit 1
fi
if ! grep -q "test_validate_block_pipeline_live.sh" "$ARCH_DOC"; then
  echo "expected architecture doc to reference test_validate_block_pipeline_live.sh" >&2
  exit 1
fi
if ! grep -q "Phase 3.2 live validation delivered" "$ROADMAP_DOC"; then
  echo "expected roadmap to include Phase 3.2 live validation status marker" >&2
  exit 1
fi
if ! grep -q "scripts/runtime/validate_block_pipeline_live.sh" "$ROADMAP_DOC"; then
  echo "expected roadmap to reference block pipeline live validation lane command" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "block pipeline live validation exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

report_json="$(mktemp)"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$report_json" <<JSON
{
  "schema_version": "kamn.runtime.block-pipeline-live-validation.v1",
  "status": "pass",
  "final_decision": "GO",
  "block_pipeline_contract_status": "verified",
  "evidence_bundle_status": "verified",
  "docs_contract_status": "verified",
  "fail_closed_status": "verified",
  "fail_closed_reason_code": "block_pipeline_payload_digest_mismatch",
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
echo "block_pipeline_contract_status=verified"
echo "evidence_bundle_status=verified"
echo "docs_contract_status=verified"
echo "fail_closed_status=verified"
echo "fail_closed_reason_code=block_pipeline_payload_digest_mismatch"
echo "performance_budget_status=verified"
