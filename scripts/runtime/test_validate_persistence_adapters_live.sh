#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_persistence_adapters_live.sh"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected persistence adapter live validation script to be executable" >&2
  exit 1
fi

validation_output="$(bash "$VALIDATION_SCRIPT" --output-json "$TMP_REPORT")"
if ! printf '%s\n' "$validation_output" | grep -q '^status=pass$'; then
  echo "expected persistence adapter live validation pass status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^final_decision=GO$'; then
  echo "expected persistence adapter live validation GO decision marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^content_persistence_status=verified$'; then
  echo "expected persistence adapter live validation content marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^did_duplicate_detection_status=verified$'; then
  echo "expected persistence adapter live validation did marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^restart_recovery_status=verified$'; then
  echo "expected persistence adapter live validation restart marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^corruption_fail_closed_status=verified$'; then
  echo "expected persistence adapter live validation corruption marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^incompatible_schema_fail_closed_status=verified$'; then
  echo "expected persistence adapter live validation incompatible-schema marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^fail_closed_status=verified$'; then
  echo "expected persistence adapter live validation fail-closed marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^evidence_bundle_status=verified$'; then
  echo "expected persistence adapter live validation evidence marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^execution_scope=local-scheduled$'; then
  echo "expected persistence adapter live validation execution scope marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^performance_budget_status=verified$'; then
  echo "expected persistence adapter live validation performance marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^persistence_gate_reason_taxonomy_version=kamn.runtime.persistence-gate-reason-taxonomy.v1$'; then
  echo "expected persistence adapter live validation reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^persistence_gate_reason_codes_csv=content_storage_corrupt_payload_rejected,did_registry_corrupt_payload_rejected,task_operation_snapshot_schema_mismatch_rejected,durable_guard_snapshot_schema_mismatch_rejected,channel_snapshot_corrupt_payload_rejected,channel_snapshot_schema_mismatch_rejected,message_lifecycle_snapshot_corrupt_payload_rejected,message_lifecycle_snapshot_schema_mismatch_rejected,runtime_snapshot_corrupt_payload_rejected,runtime_snapshot_state_version_regression_rejected,persistence_evidence_tamper_detected,persistence_evidence_freshness_window_exceeded,persistence_evidence_incomplete,persistence_ci_smoke_local_heavy_boundary_violation$'; then
  echo "expected persistence adapter live validation reason codes csv marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^persistence_tamper_freshness_drift_fail_closed_status=verified$'; then
  echo "expected persistence adapter live validation tamper/freshness marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^persistence_evidence_completeness_status=verified$'; then
  echo "expected persistence adapter live validation completeness marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^persistence_ci_smoke_local_heavy_boundary_status=verified$'; then
  echo "expected persistence adapter live validation ci/local boundary marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^persistence_ci_smoke_lane_cost_profile=low$'; then
  echo "expected persistence adapter live validation ci smoke cost profile marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^persistence_local_heavy_execution_mode=opt_in$'; then
  echo "expected persistence adapter live validation local-heavy execution marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.persistence.adapters-live-validation.v1":
    raise SystemExit("unexpected persistence adapter live validation schema")
if payload.get("status") != "pass":
    raise SystemExit("expected persistence adapter live validation status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected persistence adapter live validation final_decision=GO")
if payload.get("content_persistence_status") != "verified":
    raise SystemExit("expected content_persistence_status=verified")
if payload.get("did_duplicate_detection_status") != "verified":
    raise SystemExit("expected did_duplicate_detection_status=verified")
if payload.get("restart_recovery_status") != "verified":
    raise SystemExit("expected restart_recovery_status=verified")
if payload.get("corruption_fail_closed_status") != "verified":
    raise SystemExit("expected corruption_fail_closed_status=verified")
if payload.get("incompatible_schema_fail_closed_status") != "verified":
    raise SystemExit("expected incompatible_schema_fail_closed_status=verified")
if payload.get("fail_closed_status") != "verified":
    raise SystemExit("expected fail_closed_status=verified")
if payload.get("evidence_bundle_status") != "verified":
    raise SystemExit("expected evidence_bundle_status=verified")
if payload.get("execution_scope") != "local-scheduled":
    raise SystemExit("expected execution_scope=local-scheduled")
if payload.get("performance_budget_status") != "verified":
    raise SystemExit("expected performance_budget_status=verified")
if payload.get("persistence_gate_reason_taxonomy_version") != "kamn.runtime.persistence-gate-reason-taxonomy.v1":
    raise SystemExit("expected persistence_gate_reason_taxonomy_version contract marker")
if payload.get("persistence_gate_reason_codes_csv") != "content_storage_corrupt_payload_rejected,did_registry_corrupt_payload_rejected,task_operation_snapshot_schema_mismatch_rejected,durable_guard_snapshot_schema_mismatch_rejected,channel_snapshot_corrupt_payload_rejected,channel_snapshot_schema_mismatch_rejected,message_lifecycle_snapshot_corrupt_payload_rejected,message_lifecycle_snapshot_schema_mismatch_rejected,runtime_snapshot_corrupt_payload_rejected,runtime_snapshot_state_version_regression_rejected,persistence_evidence_tamper_detected,persistence_evidence_freshness_window_exceeded,persistence_evidence_incomplete,persistence_ci_smoke_local_heavy_boundary_violation":
    raise SystemExit("expected persistence_gate_reason_codes_csv contract marker")
if payload.get("persistence_tamper_freshness_drift_fail_closed_status") != "verified":
    raise SystemExit("expected persistence_tamper_freshness_drift_fail_closed_status=verified")
if payload.get("persistence_evidence_completeness_status") != "verified":
    raise SystemExit("expected persistence_evidence_completeness_status=verified")
if payload.get("persistence_ci_smoke_local_heavy_boundary_status") != "verified":
    raise SystemExit("expected persistence_ci_smoke_local_heavy_boundary_status=verified")
if payload.get("persistence_ci_smoke_lane_cost_profile") != "low":
    raise SystemExit("expected persistence_ci_smoke_lane_cost_profile=low")
if payload.get("persistence_local_heavy_execution_mode") != "opt_in":
    raise SystemExit("expected persistence_local_heavy_execution_mode=opt_in")
reason_codes = payload.get("fail_closed_reason_codes")
if reason_codes != [
    "content_storage_corrupt_payload_rejected",
    "did_registry_corrupt_payload_rejected",
    "task_operation_snapshot_schema_mismatch_rejected",
    "durable_guard_snapshot_schema_mismatch_rejected",
    "channel_snapshot_corrupt_payload_rejected",
    "channel_snapshot_schema_mismatch_rejected",
    "message_lifecycle_snapshot_corrupt_payload_rejected",
    "message_lifecycle_snapshot_schema_mismatch_rejected",
    "runtime_snapshot_corrupt_payload_rejected",
    "runtime_snapshot_state_version_regression_rejected",
    "persistence_evidence_tamper_detected",
    "persistence_evidence_freshness_window_exceeded",
    "persistence_evidence_incomplete",
    "persistence_ci_smoke_local_heavy_boundary_violation",
]:
    raise SystemExit("expected deterministic fail_closed_reason_codes contract list")
PY

set +e
invalid_budget_output="$(
  bash "$VALIDATION_SCRIPT" \
    --max-seconds invalid 2>&1
)"
invalid_budget_code=$?
set -e
if [ "$invalid_budget_code" -eq 0 ]; then
  echo "expected persistence adapter live validation script to reject invalid max-seconds" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_budget_output" | grep -q "max-seconds must be an integer"; then
  echo "expected deterministic invalid max-seconds marker" >&2
  exit 1
fi

set +e
zero_budget_output="$(
  bash "$VALIDATION_SCRIPT" \
    --max-seconds 0 2>&1
)"
zero_budget_code=$?
set -e
if [ "$zero_budget_code" -eq 0 ]; then
  echo "expected persistence adapter live validation script to reject zero max-seconds" >&2
  exit 1
fi
if ! printf '%s\n' "$zero_budget_output" | grep -q "max-seconds must be greater than zero"; then
  echo "expected deterministic zero max-seconds marker" >&2
  exit 1
fi

echo "persistence adapter live validation tests passed."
