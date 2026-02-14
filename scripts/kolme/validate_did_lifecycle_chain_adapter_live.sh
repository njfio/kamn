#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_did_lifecycle_chain_adapter_contract_lane.sh"

output_json=""
max_seconds=240

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

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

start_epoch="$(date +%s)"
contract_report="$TMP_DIR/did-lifecycle-chain-contract-report.json"
run_output="$({
  bash "$RUNNER" --max-seconds 180 --output-json "$contract_report"
} 2>&1)"

if ! printf '%s\n' "$run_output" | grep -q '^status=pass$'; then
  echo "expected did lifecycle contract pass marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^final_decision=GO$'; then
  echo "expected did lifecycle contract GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^lifecycle_chain_contract_status=verified$'; then
  echo "expected did lifecycle contract marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^duplicate_retry_status=verified$'; then
  echo "expected duplicate retry marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^conflict_fail_closed_status=verified$'; then
  echo "expected conflict fail-closed marker" >&2
  exit 1
fi

docs_output_file="$TMP_DIR/did-lifecycle-docs-validation.out"
set +e
(
  cd "$ROOT_DIR"
  cargo test -p kamn-core --test did_registry_transactions_docs -- \
    regression_doc_marks_lifecycle_chain_submission_conflict_guard
) >"$docs_output_file" 2>&1
docs_code=$?
set -e
if [ "$docs_code" -ne 0 ]; then
  cat "$docs_output_file" >&2
  echo "did lifecycle docs validation failed" >&2
  exit 1
fi

set +e
fail_closed_output="$({
  cd "$ROOT_DIR"
  cargo test -p kamn-core --test did_registry_transactions -- \
    regression_lifecycle_chain_submission_rejects_conflicting_same_nonce_payload
} 2>&1)"
fail_closed_code=$?
set -e
if [ "$fail_closed_code" -ne 0 ]; then
  printf '%s\n' "$fail_closed_output" >&2
  echo "expected conflict fail-closed drill to pass" >&2
  exit 1
fi
if ! printf '%s\n' "$fail_closed_output" | grep -q '1 passed; 0 failed'; then
  printf '%s\n' "$fail_closed_output" >&2
  echo "expected conflict fail-closed pass-count marker" >&2
  exit 1
fi

set +e
performance_output="$({
  cd "$ROOT_DIR"
  cargo test -p kamn-core --test did_registry_transactions -- \
    performance_lifecycle_mutation_contract_lane_stays_within_budget
} 2>&1)"
performance_code=$?
set -e
if [ "$performance_code" -ne 0 ]; then
  printf '%s\n' "$performance_output" >&2
  echo "expected lifecycle performance drill to pass" >&2
  exit 1
fi
if ! printf '%s\n' "$performance_output" | grep -q '1 passed; 0 failed'; then
  printf '%s\n' "$performance_output" >&2
  echo "expected performance pass-count marker" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "did lifecycle chain adapter live validation exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

report_json="$TMP_DIR/did-lifecycle-chain-live-validation-report.json"
cat >"$report_json" <<JSON
{
  "schema_version": "kamn.kolme.did-lifecycle-chain.live-validation.v1",
  "status": "pass",
  "final_decision": "GO",
  "did_lifecycle_contract_status": "verified",
  "evidence_bundle_status": "verified",
  "docs_contract_status": "verified",
  "fail_closed_status": "verified",
  "fail_closed_reason_code": "did_registry_submission_key_conflict",
  "performance_budget_status": "verified",
  "elapsed_seconds": ${elapsed_seconds}
}
JSON

if [[ -n "$output_json" ]]; then
  cp "$report_json" "$output_json"
fi

echo "status=pass"
echo "final_decision=GO"
echo "did_lifecycle_contract_status=verified"
echo "evidence_bundle_status=verified"
echo "docs_contract_status=verified"
echo "fail_closed_status=verified"
echo "fail_closed_reason_code=did_registry_submission_key_conflict"
echo "performance_budget_status=verified"
