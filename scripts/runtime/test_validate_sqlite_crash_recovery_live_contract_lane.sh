#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/runtime/validate_sqlite_crash_recovery_live_contract_lane.sh"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_sqlite_crash_recovery_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_sqlite_crash_recovery_live_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected sqlite crash-recovery contract lane script to be executable" >&2
  exit 1
fi
if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected sqlite crash-recovery validation script to be executable" >&2
  exit 1
fi
if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected sqlite crash-recovery policy checker script to be executable" >&2
  exit 1
fi

lane_report="$TMP_DIR/sqlite-crash-recovery-contract-lane-report.json"
policy_report="$TMP_DIR/sqlite-crash-recovery-policy-report.json"

lane_output="$(
  bash "$CONTRACT_LANE" \
    --mode dry-run \
    --max-seconds 240 \
    --ci-fast-gate PASS \
    --output-json "$lane_report" \
    --policy-output-json "$policy_report"
)"
if ! printf '%s\n' "$lane_output" | grep -q '^status=pass$'; then
  echo "expected sqlite crash-recovery contract lane status=pass marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^final_decision=GO$'; then
  echo "expected sqlite crash-recovery contract lane final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^lane_mode=dry-run$'; then
  echo "expected sqlite crash-recovery contract lane mode marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^wal_append_status=verified$'; then
  echo "expected sqlite crash-recovery contract lane wal-append marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^wal_checkpoint_status=verified$'; then
  echo "expected sqlite crash-recovery contract lane wal-checkpoint marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^wal_durability_reason_taxonomy_version=kamn.runtime.wal-durability-reason-taxonomy.v1$'; then
  echo "expected sqlite crash-recovery contract lane wal-durability reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^wal_durability_reason_codes_csv=wal_append_rejected,wal_checkpoint_skipped,wal_replay_incomplete$'; then
  echo "expected sqlite crash-recovery contract lane wal-durability reason taxonomy csv marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^historical_query_index_status=verified$'; then
  echo "expected sqlite crash-recovery contract lane historical-query index marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^historical_query_latency_budget_status=verified$'; then
  echo "expected sqlite crash-recovery contract lane historical-query latency budget marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^historical_query_reason_taxonomy_version=kamn.runtime.historical-query-reason-taxonomy.v1$'; then
  echo "expected sqlite crash-recovery contract lane historical-query reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^historical_query_reason_codes_csv=historical_query_index_drift,historical_query_latency_budget_exceeded,historical_query_consistency_mismatch$'; then
  echo "expected sqlite crash-recovery contract lane historical-query reason taxonomy csv marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^crash_recovery_promotion_gate_status=verified$'; then
  echo "expected sqlite crash-recovery contract lane promotion-gate marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^audit_trail_parity_status=verified$'; then
  echo "expected sqlite crash-recovery contract lane audit-trail parity marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^ci_local_promotion_budget_boundary_status=verified$'; then
  echo "expected sqlite crash-recovery contract lane ci-local promotion budget boundary marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^durability_governance_reason_taxonomy_version=kamn.runtime.durability-governance-reason-taxonomy.v1$'; then
  echo "expected sqlite crash-recovery contract lane durability-governance reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^durability_governance_reason_codes_csv=crash_recovery_promotion_stalled,audit_trail_parity_mismatch,ci_local_promotion_budget_boundary_exceeded$'; then
  echo "expected sqlite crash-recovery contract lane durability-governance reason taxonomy csv marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^sqlite_crash_recovery_policy_status=verified$'; then
  echo "expected sqlite crash-recovery contract lane policy status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^sqlite_crash_recovery_contract_status=verified$'; then
  echo "expected sqlite crash-recovery contract lane status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^fail_closed_reason_code=sqlite_crash_recovery_policy_fast_gate_exclusion_mismatch$'; then
  echo "expected sqlite crash-recovery contract lane fail-closed reason marker" >&2
  exit 1
fi

python3 - "$lane_report" "$policy_report" <<'PY'
import json
import pathlib
import sys

lane_payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if lane_payload.get("schema_version") != "kamn.runtime.sqlite-crash-recovery-live-contract-lane-report.v1":
    raise SystemExit("unexpected sqlite crash-recovery contract lane report schema")
if lane_payload.get("status") != "pass":
    raise SystemExit("expected contract lane status=pass")
if lane_payload.get("final_decision") != "GO":
    raise SystemExit("expected contract lane final_decision=GO")
if lane_payload.get("wal_append_status") != "verified":
    raise SystemExit("expected wal_append_status=verified")
if lane_payload.get("wal_checkpoint_status") != "verified":
    raise SystemExit("expected wal_checkpoint_status=verified")
if lane_payload.get("wal_durability_reason_taxonomy_version") != "kamn.runtime.wal-durability-reason-taxonomy.v1":
    raise SystemExit("expected deterministic wal_durability_reason_taxonomy_version marker")
if lane_payload.get("wal_durability_reason_codes_csv") != "wal_append_rejected,wal_checkpoint_skipped,wal_replay_incomplete":
    raise SystemExit("expected deterministic wal_durability_reason_codes_csv marker")
if lane_payload.get("historical_query_index_status") != "verified":
    raise SystemExit("expected historical_query_index_status=verified")
if lane_payload.get("historical_query_latency_budget_status") != "verified":
    raise SystemExit("expected historical_query_latency_budget_status=verified")
if lane_payload.get("historical_query_reason_taxonomy_version") != "kamn.runtime.historical-query-reason-taxonomy.v1":
    raise SystemExit("expected deterministic historical_query_reason_taxonomy_version marker")
if lane_payload.get("historical_query_reason_codes_csv") != "historical_query_index_drift,historical_query_latency_budget_exceeded,historical_query_consistency_mismatch":
    raise SystemExit("expected deterministic historical_query_reason_codes_csv marker")
if lane_payload.get("crash_recovery_promotion_gate_status") != "verified":
    raise SystemExit("expected crash_recovery_promotion_gate_status=verified")
if lane_payload.get("audit_trail_parity_status") != "verified":
    raise SystemExit("expected audit_trail_parity_status=verified")
if lane_payload.get("ci_local_promotion_budget_boundary_status") != "verified":
    raise SystemExit("expected ci_local_promotion_budget_boundary_status=verified")
if lane_payload.get("durability_governance_reason_taxonomy_version") != "kamn.runtime.durability-governance-reason-taxonomy.v1":
    raise SystemExit("expected deterministic durability_governance_reason_taxonomy_version marker")
if lane_payload.get("durability_governance_reason_codes_csv") != "crash_recovery_promotion_stalled,audit_trail_parity_mismatch,ci_local_promotion_budget_boundary_exceeded":
    raise SystemExit("expected deterministic durability_governance_reason_codes_csv marker")
if lane_payload.get("sqlite_crash_recovery_policy_status") != "verified":
    raise SystemExit("expected sqlite_crash_recovery_policy_status=verified")
if lane_payload.get("sqlite_crash_recovery_contract_status") != "verified":
    raise SystemExit("expected sqlite_crash_recovery_contract_status=verified")
if lane_payload.get("docs_contract_status") != "verified":
    raise SystemExit("expected docs_contract_status=verified")
if lane_payload.get("performance_budget_status") != "verified":
    raise SystemExit("expected performance_budget_status=verified")

policy_payload = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
if policy_payload.get("schema_version") != "kamn.runtime.sqlite-crash-recovery-live-policy-report.v1":
    raise SystemExit("unexpected sqlite crash-recovery policy report schema")
if policy_payload.get("final_decision") != "GO":
    raise SystemExit("expected policy final_decision=GO")
if policy_payload.get("sqlite_crash_recovery_policy_status") != "verified":
    raise SystemExit("expected sqlite_crash_recovery_policy_status=verified in policy report")
if policy_payload.get("historical_query_reason_taxonomy_version") != "kamn.runtime.historical-query-reason-taxonomy.v1":
    raise SystemExit("expected deterministic historical_query_reason_taxonomy_version marker in policy report")
if policy_payload.get("historical_query_reason_codes_csv") != "historical_query_index_drift,historical_query_latency_budget_exceeded,historical_query_consistency_mismatch":
    raise SystemExit("expected deterministic historical_query_reason_codes_csv marker in policy report")
if policy_payload.get("durability_governance_reason_taxonomy_version") != "kamn.runtime.durability-governance-reason-taxonomy.v1":
    raise SystemExit("expected deterministic durability_governance_reason_taxonomy_version marker in policy report")
if policy_payload.get("durability_governance_reason_codes_csv") != "crash_recovery_promotion_stalled,audit_trail_parity_mismatch,ci_local_promotion_budget_boundary_exceeded":
    raise SystemExit("expected deterministic durability_governance_reason_codes_csv marker in policy report")
PY

if ! grep -q "check_sqlite_crash_recovery_live_policy.sh" "$CONTRACT_LANE"; then
  echo "expected sqlite crash-recovery contract lane to compose policy checker" >&2
  exit 1
fi
if ! grep -q "validate_sqlite_crash_recovery_live.sh" "$CONTRACT_LANE"; then
  echo "expected sqlite crash-recovery contract lane to compose validation lane" >&2
  exit 1
fi

set +e
invalid_ci_fast_gate_output="$(
  bash "$CONTRACT_LANE" \
    --mode dry-run \
    --ci-fast-gate MAYBE 2>&1
)"
invalid_ci_fast_gate_code=$?
set -e
if [ "$invalid_ci_fast_gate_code" -eq 0 ]; then
  echo "expected sqlite crash-recovery contract lane to reject invalid ci-fast-gate value" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_ci_fast_gate_output" | grep -q 'ci-fast-gate must be PASS or FAIL'; then
  echo "expected deterministic invalid ci-fast-gate marker for sqlite crash-recovery contract lane" >&2
  exit 1
fi

echo "sqlite crash-recovery contract lane tests passed."
