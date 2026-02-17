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

KOLME_DOC="$ROOT_DIR/docs/foundation/kolme-runtime-commit-client.md"
ROADMAP_DOC="$ROOT_DIR/docs/plans/2026-02-08-production-service-roadmap.md"

start_epoch="$(date +%s)"

pushd "$ROOT_DIR" >/dev/null
cargo test -p kamn-node integration_kolme_live_nonce_resolver_retries_unavailable_then_succeeds
cargo test -p kamn-node functional_kolme_live_nonce_retry_emits_structured_retry_marker
cargo test -p kamn-node regression_kolme_live_nonce_resolver_rejects_malformed_response
popd >/dev/null

if ! grep -q "validate_nonce_retry_live.sh" "$KOLME_DOC"; then
  echo "expected Kolme runtime commit doc to reference validate_nonce_retry_live.sh" >&2
  exit 1
fi
if ! grep -q "test_validate_nonce_retry_live.sh" "$KOLME_DOC"; then
  echo "expected Kolme runtime commit doc to reference test_validate_nonce_retry_live.sh" >&2
  exit 1
fi
if ! grep -q "kolme.live.nonce.retry" "$KOLME_DOC"; then
  echo "expected Kolme runtime commit doc to reference kolme.live.nonce.retry marker" >&2
  exit 1
fi
if ! grep -q "Task #3042, Subtask #3043" "$ROADMAP_DOC"; then
  echo "expected roadmap to include Task #3042, Subtask #3043 marker" >&2
  exit 1
fi
if ! grep -q "scripts/runtime/validate_nonce_retry_live.sh" "$ROADMAP_DOC"; then
  echo "expected roadmap to reference nonce retry live validation lane command" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "nonce retry live validation exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

report_json="$(mktemp)"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$report_json" <<JSON
{
  "schema_version": "kamn.runtime.nonce-retry-live-validation.v1",
  "status": "pass",
  "final_decision": "GO",
  "nonce_retry_contract_status": "verified",
  "nonce_malformed_fail_closed_status": "verified",
  "docs_contract_status": "verified",
  "fail_closed_reason_code": "nonce_response_malformed",
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
echo "nonce_retry_contract_status=verified"
echo "nonce_malformed_fail_closed_status=verified"
echo "docs_contract_status=verified"
echo "fail_closed_reason_code=nonce_response_malformed"
echo "performance_budget_status=verified"
