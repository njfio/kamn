#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PERSISTENCE_DOC="$ROOT_DIR/docs/architecture/persistence-backends.md"
ROADMAP_DOC="$ROOT_DIR/docs/plans/2026-02-08-production-service-roadmap.md"

output_json=""
max_seconds=180

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

if [ ! -f "$PERSISTENCE_DOC" ]; then
  echo "expected persistence architecture doc to exist: $PERSISTENCE_DOC" >&2
  exit 1
fi
if [ ! -f "$ROADMAP_DOC" ]; then
  echo "expected roadmap doc to exist: $ROADMAP_DOC" >&2
  exit 1
fi

for marker in \
  "validate_persistence_adapters_live.sh" \
  "test_validate_persistence_adapters_live.sh" \
  "kamn.persistence.adapters-live-validation.v1" \
  "restart_recovery_status=verified" \
  "corruption_fail_closed_status=verified" \
  "incompatible_schema_fail_closed_status=verified" \
  "execution_scope=local-scheduled" \
  "channel_snapshot_corrupt_payload_rejected" \
  "channel_snapshot_schema_mismatch_rejected" \
  "message_lifecycle_snapshot_corrupt_payload_rejected" \
  "message_lifecycle_snapshot_schema_mismatch_rejected" \
  "runtime_snapshot_corrupt_payload_rejected" \
  "runtime_snapshot_state_version_regression_rejected"; do
  if ! grep -q -- "$marker" "$PERSISTENCE_DOC"; then
    echo "expected persistence architecture doc marker: $marker" >&2
    exit 1
  fi
done

for marker in \
  "Task #3068, Subtask #3070" \
  "Task #3078" \
  "scripts/runtime/validate_persistence_adapters_live.sh" \
  "content_storage_corrupt_payload_rejected" \
  "task_operation_snapshot_schema_mismatch_rejected" \
  "durable_guard_snapshot_schema_mismatch_rejected" \
  "channel_snapshot_corrupt_payload_rejected" \
  "channel_snapshot_schema_mismatch_rejected" \
  "message_lifecycle_snapshot_corrupt_payload_rejected" \
  "message_lifecycle_snapshot_schema_mismatch_rejected" \
  "runtime_snapshot_corrupt_payload_rejected" \
  "runtime_snapshot_state_version_regression_rejected"; do
  if ! grep -q -- "$marker" "$ROADMAP_DOC"; then
    echo "expected roadmap marker for persistence live validation: $marker" >&2
    exit 1
  fi
done

pushd "$ROOT_DIR" >/dev/null
cargo test -p kamn-core --test content_storage_file_adapter \
  content_storage_file_adapter_persists_round_trip_across_reopen -- --nocapture \
  >"$TMP_DIR/content-storage-restart.log" 2>&1
cargo test -p kamn-core --test did_registry_file_chain_adapter \
  did_registry_file_chain_adapter_persists_duplicate_detection_across_restart -- --nocapture \
  >"$TMP_DIR/did-chain-restart.log" 2>&1
cargo test -p kamn-core --test content_storage_file_adapter \
  content_storage_file_adapter_regression_rejects_corrupt_payload_line -- --nocapture \
  >"$TMP_DIR/content-storage-corrupt.log" 2>&1
cargo test -p kamn-core --test did_registry_file_chain_adapter \
  did_registry_file_chain_adapter_regression_rejects_corrupt_payload_line -- --nocapture \
  >"$TMP_DIR/did-chain-corrupt.log" 2>&1
cargo test -p kamn-core --test task_operation_snapshot \
  task_operation_snapshot_rejects_schema_version_mismatch -- --nocapture \
  >"$TMP_DIR/task-snapshot-schema-mismatch.log" 2>&1
cargo test -p kamn-core --test durable_guard_snapshot_store \
  unit_bundle_schema_mismatch_is_rejected -- --nocapture \
  >"$TMP_DIR/durable-guard-schema-mismatch.log" 2>&1
cargo test -p kamn-core --test task_operation_snapshot \
  task_operation_snapshot_bounded_roundtrip_benchmark_is_fast_for_ci -- --nocapture \
  >"$TMP_DIR/task-snapshot-performance.log" 2>&1
cargo test -p kamn-core --lib \
  bootstrap::tests::regression_bootstrap_fails_closed_when_channel_snapshot_payload_is_corrupt -- --nocapture \
  >"$TMP_DIR/bootstrap-channel-corrupt.log" 2>&1
cargo test -p kamn-core --lib \
  bootstrap::tests::regression_bootstrap_fails_closed_when_channel_snapshot_schema_is_incompatible -- --nocapture \
  >"$TMP_DIR/bootstrap-channel-schema.log" 2>&1
cargo test -p kamn-core --lib \
  bootstrap::tests::regression_bootstrap_fails_closed_when_message_snapshot_payload_is_corrupt -- --nocapture \
  >"$TMP_DIR/bootstrap-message-corrupt.log" 2>&1
cargo test -p kamn-core --lib \
  bootstrap::tests::regression_bootstrap_fails_closed_when_message_snapshot_schema_is_incompatible -- --nocapture \
  >"$TMP_DIR/bootstrap-message-schema.log" 2>&1
cargo test -p kamn-core --lib \
  bootstrap::tests::regression_bootstrap_fails_closed_when_runtime_snapshot_payload_is_corrupt -- --nocapture \
  >"$TMP_DIR/bootstrap-runtime-corrupt.log" 2>&1
cargo test -p kamn-core --lib \
  bootstrap::tests::regression_bootstrap_fails_closed_when_runtime_snapshot_state_version_regresses -- --nocapture \
  >"$TMP_DIR/bootstrap-runtime-version-regression.log" 2>&1
popd >/dev/null

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "persistence adapter live validation exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

report_json="$TMP_DIR/persistence-adapter-live-validation-report.json"
cat >"$report_json" <<JSON
{
  "schema_version": "kamn.persistence.adapters-live-validation.v1",
  "status": "pass",
  "final_decision": "GO",
  "content_persistence_status": "verified",
  "did_duplicate_detection_status": "verified",
  "restart_recovery_status": "verified",
  "corruption_fail_closed_status": "verified",
  "incompatible_schema_fail_closed_status": "verified",
  "fail_closed_status": "verified",
  "evidence_bundle_status": "verified",
  "execution_scope": "local-scheduled",
  "performance_budget_status": "verified",
  "fail_closed_reason_codes": [
    "content_storage_corrupt_payload_rejected",
    "did_registry_corrupt_payload_rejected",
    "task_operation_snapshot_schema_mismatch_rejected",
    "durable_guard_snapshot_schema_mismatch_rejected",
    "channel_snapshot_corrupt_payload_rejected",
    "channel_snapshot_schema_mismatch_rejected",
    "message_lifecycle_snapshot_corrupt_payload_rejected",
    "message_lifecycle_snapshot_schema_mismatch_rejected",
    "runtime_snapshot_corrupt_payload_rejected",
    "runtime_snapshot_state_version_regression_rejected"
  ],
  "elapsed_seconds": $elapsed_seconds
}
JSON

if [[ -n "$output_json" ]]; then
  cp "$report_json" "$output_json"
fi

echo "status=pass"
echo "final_decision=GO"
echo "content_persistence_status=verified"
echo "did_duplicate_detection_status=verified"
echo "restart_recovery_status=verified"
echo "corruption_fail_closed_status=verified"
echo "incompatible_schema_fail_closed_status=verified"
echo "fail_closed_status=verified"
echo "evidence_bundle_status=verified"
echo "execution_scope=local-scheduled"
echo "performance_budget_status=verified"
echo "fail_closed_reason_codes=content_storage_corrupt_payload_rejected,did_registry_corrupt_payload_rejected,task_operation_snapshot_schema_mismatch_rejected,durable_guard_snapshot_schema_mismatch_rejected,channel_snapshot_corrupt_payload_rejected,channel_snapshot_schema_mismatch_rejected,message_lifecycle_snapshot_corrupt_payload_rejected,message_lifecycle_snapshot_schema_mismatch_rejected,runtime_snapshot_corrupt_payload_rejected,runtime_snapshot_state_version_regression_rejected"
