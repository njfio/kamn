#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
POLICY_CHECKER="$ROOT_DIR/scripts/kolme/check_runtime_commit_replay_policy.py"
GONOGO_DOC="$ROOT_DIR/docs/foundation/release-gonogo-checklist.md"
DEVNET_PLAN_DOC="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected runtime commit replay policy checker to be executable" >&2
  exit 1
fi

if [ ! -f "$GONOGO_DOC" ] || [ ! -f "$DEVNET_PLAN_DOC" ]; then
  echo "expected release go/no-go and Kolme devnet docs to exist" >&2
  exit 1
fi

start_epoch="$(date +%s)"

cargo test -p kamn-core --test kolme_runtime_commit_client \
  functional_adapter_maps_transport_provider_and_finality_failures_to_typed_errors
cargo test -p kamn-core --test kolme_runtime_commit_client \
  integration_runtime_pipeline_accepts_adapter_backed_final_receipts
cargo test -p kamn-core --test kolme_runtime_commit_client \
  regression_adapter_path_keeps_receipt_provider_mismatch_fail_closed

set +e
provider_mismatch_output="$(
  python3 "$POLICY_CHECKER" \
    --operation-id "op-adapter-no-go-provider-mismatch-001" \
    --idempotency-key "kolme-runtime-commit:op-adapter-no-go-provider-mismatch-001:state:agent:7:12" \
    --receipt-provider "kolme-remote" \
    --expected-receipt-provider "kolme-local" \
    --receipt-commit-id "kolme-commit:op-adapter-no-go-provider-mismatch-001:agent:7:12" \
    --expected-receipt-commit-id "kolme-commit:op-adapter-no-go-provider-mismatch-001:agent:7:12" \
    --nonce-monotonic true \
    --replay-detected false \
    --payload-hash-match true \
    --receipt-finality FINAL \
    --ci-fast-gate PASS \
    --output-json "$TMP_REPORT" 2>&1
)"
provider_mismatch_code=$?
set -e
if [ "$provider_mismatch_code" -eq 0 ]; then
  echo "expected provider mismatch policy case to fail closed" >&2
  exit 1
fi
if ! printf '%s\n' "$provider_mismatch_output" | grep -q '^final_decision=NO-GO$'; then
  echo "expected provider mismatch policy case to produce NO-GO" >&2
  exit 1
fi
if ! printf '%s\n' "$provider_mismatch_output" | grep -q 'receipt_provider_mismatch'; then
  echo "expected provider mismatch policy case to emit receipt_provider_mismatch reason code" >&2
  exit 1
fi

set +e
non_final_output="$(
  python3 "$POLICY_CHECKER" \
    --operation-id "op-adapter-no-go-non-final-001" \
    --idempotency-key "kolme-runtime-commit:op-adapter-no-go-non-final-001:state:agent:8:12" \
    --receipt-provider "kolme-local" \
    --expected-receipt-provider "kolme-local" \
    --receipt-commit-id "kolme-commit:op-adapter-no-go-non-final-001:agent:8:12" \
    --expected-receipt-commit-id "kolme-commit:op-adapter-no-go-non-final-001:agent:8:12" \
    --nonce-monotonic true \
    --replay-detected false \
    --payload-hash-match true \
    --receipt-finality PENDING \
    --ci-fast-gate PASS \
    --output-json "$TMP_REPORT" 2>&1
)"
non_final_code=$?
set -e
if [ "$non_final_code" -eq 0 ]; then
  echo "expected non-final receipt policy case to fail closed" >&2
  exit 1
fi
if ! printf '%s\n' "$non_final_output" | grep -q '^final_decision=NO-GO$'; then
  echo "expected non-final receipt policy case to produce NO-GO" >&2
  exit 1
fi
if ! printf '%s\n' "$non_final_output" | grep -q 'receipt_not_final'; then
  echo "expected non-final receipt policy case to emit receipt_not_final reason code" >&2
  exit 1
fi

if ! grep -q "run_runtime_commit_adapter_contract_lane.sh" "$GONOGO_DOC"; then
  echo "expected release go/no-go doc to reference adapter runtime commit contract lane command" >&2
  exit 1
fi
if ! grep -q "run_runtime_commit_adapter_contract_lane.sh" "$DEVNET_PLAN_DOC"; then
  echo "expected Kolme devnet plan doc to reference adapter runtime commit contract lane command" >&2
  exit 1
fi
if ! grep -q "receipt_provider_mismatch" "$GONOGO_DOC"; then
  echo "expected release go/no-go doc to reference receipt_provider_mismatch reason code" >&2
  exit 1
fi
if ! grep -q "receipt_not_final" "$GONOGO_DOC"; then
  echo "expected release go/no-go doc to reference receipt_not_final reason code" >&2
  exit 1
fi
if ! grep -q "Regression: #980" "$GONOGO_DOC"; then
  echo "expected release go/no-go doc to include adapter replay/finality regression marker" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt 60 ]; then
  echo "Kolme runtime commit adapter contract lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

echo "Kolme runtime commit adapter contract lane tests passed."
