#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/runtime/validate_block_reconciliation_partition_rejoin_live_contract_lane.sh"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_block_reconciliation_partition_rejoin_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_block_reconciliation_partition_rejoin_live_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected block reconciliation partition/rejoin contract lane script to be executable" >&2
  exit 1
fi
if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected block reconciliation partition/rejoin validation script to be executable" >&2
  exit 1
fi
if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected block reconciliation partition/rejoin policy checker script to be executable" >&2
  exit 1
fi

lane_report="$TMP_DIR/block-reconciliation-partition-rejoin-contract-lane-report.json"
policy_report="$TMP_DIR/block-reconciliation-partition-rejoin-policy-report.json"

lane_output="$(
  bash "$CONTRACT_LANE" \
    --mode dry-run \
    --max-seconds 240 \
    --ci-fast-gate PASS \
    --output-json "$lane_report" \
    --policy-output-json "$policy_report"
)"
if ! printf '%s\n' "$lane_output" | grep -q '^status=pass$'; then
  echo "expected block reconciliation partition/rejoin contract lane status=pass marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^final_decision=GO$'; then
  echo "expected block reconciliation partition/rejoin contract lane final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^lane_mode=dry-run$'; then
  echo "expected block reconciliation partition/rejoin contract lane mode marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^block_reconciliation_partition_rejoin_policy_status=verified$'; then
  echo "expected block reconciliation partition/rejoin contract lane policy status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^block_reconciliation_partition_rejoin_contract_status=verified$'; then
  echo "expected block reconciliation partition/rejoin contract lane status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^runtime_transport_mode_status=verified$'; then
  echo "expected block reconciliation partition/rejoin contract lane runtime transport mode marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^transport_evidence_normalization_status=verified$'; then
  echo "expected block reconciliation partition/rejoin contract lane transport evidence normalization marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^transport_evidence_schema_version=kamn.runtime.libp2p-transport-transition-evidence.v1$'; then
  echo "expected block reconciliation partition/rejoin contract lane transport evidence schema marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^reconciliation_reason_taxonomy_status=verified$'; then
  echo "expected block reconciliation partition/rejoin contract lane reconciliation taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^reconciliation_reason_taxonomy_version=kamn.runtime.block-reconciliation-partition-rejoin-reason-taxonomy.v1$'; then
  echo "expected block reconciliation partition/rejoin contract lane reconciliation taxonomy version marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^reconciliation_reason_codes_csv=reconciliation_partition_transition_failed,reconciliation_rejoin_transition_failed,reconciliation_publish_drop_recovery_failed,reconciliation_peer_churn_recovery_failed,reconciliation_split_head_unresolved,reconciliation_replay_instability,reconciliation_fixture_contract_failed,reconciliation_unclassified_scenario_failed,reconciliation_runtime_budget_exceeded,reconciliation_ci_fast_gate_failed$'; then
  echo "expected block reconciliation partition/rejoin contract lane reconciliation taxonomy csv marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^snapshot_wal_reconciliation_status=verified$'; then
  echo "expected block reconciliation partition/rejoin contract lane snapshot-vs-wal reconciliation marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^consistency_classification_status=verified$'; then
  echo "expected block reconciliation partition/rejoin contract lane consistency classification marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^reconciliation_consistency_reason_taxonomy_version=kamn.runtime.snapshot-wal-consistency-reason-taxonomy.v1$'; then
  echo "expected block reconciliation partition/rejoin contract lane consistency taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^reconciliation_consistency_reason_codes_csv=snapshot_wal_lineage_diverged,snapshot_wal_checkpoint_stale,consistency_classification_mismatch$'; then
  echo "expected block reconciliation partition/rejoin contract lane consistency taxonomy csv marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^fail_closed_reason_code=block_reconciliation_partition_rejoin_policy_fast_gate_exclusion_mismatch$'; then
  echo "expected block reconciliation partition/rejoin contract lane fail-closed reason marker" >&2
  exit 1
fi

python3 - "$lane_report" "$policy_report" <<'PY'
import json
import pathlib
import sys

lane_payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if lane_payload.get("schema_version") != "kamn.runtime.block-reconciliation-partition-rejoin-live-contract-lane-report.v1":
    raise SystemExit("unexpected block reconciliation partition/rejoin contract lane report schema")
if lane_payload.get("status") != "pass":
    raise SystemExit("expected contract lane status=pass")
if lane_payload.get("final_decision") != "GO":
    raise SystemExit("expected contract lane final_decision=GO")
if lane_payload.get("block_reconciliation_partition_rejoin_policy_status") != "verified":
    raise SystemExit("expected block_reconciliation_partition_rejoin_policy_status=verified")
if lane_payload.get("block_reconciliation_partition_rejoin_contract_status") != "verified":
    raise SystemExit("expected block_reconciliation_partition_rejoin_contract_status=verified")
if lane_payload.get("docs_contract_status") != "verified":
    raise SystemExit("expected docs_contract_status=verified")
if lane_payload.get("runtime_transport_mode_status") != "verified":
    raise SystemExit("expected runtime_transport_mode_status=verified")
if lane_payload.get("transport_evidence_normalization_status") != "verified":
    raise SystemExit("expected transport_evidence_normalization_status=verified")
if lane_payload.get("transport_evidence_schema_version") != "kamn.runtime.libp2p-transport-transition-evidence.v1":
    raise SystemExit("expected deterministic transport_evidence_schema_version marker")
if lane_payload.get("reconciliation_reason_taxonomy_status") != "verified":
    raise SystemExit("expected reconciliation_reason_taxonomy_status=verified")
if lane_payload.get("reconciliation_reason_taxonomy_version") != "kamn.runtime.block-reconciliation-partition-rejoin-reason-taxonomy.v1":
    raise SystemExit("expected deterministic reconciliation_reason_taxonomy_version marker")
if lane_payload.get("reconciliation_reason_codes_csv") != "reconciliation_partition_transition_failed,reconciliation_rejoin_transition_failed,reconciliation_publish_drop_recovery_failed,reconciliation_peer_churn_recovery_failed,reconciliation_split_head_unresolved,reconciliation_replay_instability,reconciliation_fixture_contract_failed,reconciliation_unclassified_scenario_failed,reconciliation_runtime_budget_exceeded,reconciliation_ci_fast_gate_failed":
    raise SystemExit("expected deterministic reconciliation_reason_codes_csv marker")
if lane_payload.get("snapshot_wal_reconciliation_status") != "verified":
    raise SystemExit("expected snapshot_wal_reconciliation_status=verified")
if lane_payload.get("consistency_classification_status") != "verified":
    raise SystemExit("expected consistency_classification_status=verified")
if lane_payload.get("reconciliation_consistency_reason_taxonomy_version") != "kamn.runtime.snapshot-wal-consistency-reason-taxonomy.v1":
    raise SystemExit("expected deterministic reconciliation_consistency_reason_taxonomy_version marker")
if lane_payload.get("reconciliation_consistency_reason_codes_csv") != "snapshot_wal_lineage_diverged,snapshot_wal_checkpoint_stale,consistency_classification_mismatch":
    raise SystemExit("expected deterministic reconciliation_consistency_reason_codes_csv marker")
if lane_payload.get("performance_budget_status") != "verified":
    raise SystemExit("expected performance_budget_status=verified")

policy_payload = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
if policy_payload.get("schema_version") != "kamn.runtime.block-reconciliation-partition-rejoin-live-policy-report.v1":
    raise SystemExit("unexpected block reconciliation partition/rejoin policy report schema")
if policy_payload.get("final_decision") != "GO":
    raise SystemExit("expected policy final_decision=GO")
if policy_payload.get("block_reconciliation_partition_rejoin_policy_status") != "verified":
    raise SystemExit("expected block_reconciliation_partition_rejoin_policy_status=verified in policy report")
if policy_payload.get("transport_evidence_schema_version") != "kamn.runtime.libp2p-transport-transition-evidence.v1":
    raise SystemExit("expected deterministic transport_evidence_schema_version marker in policy report")
if policy_payload.get("transport_evidence_normalization_status") != "verified":
    raise SystemExit("expected deterministic transport_evidence_normalization_status marker in policy report")
if policy_payload.get("transport_evidence_source_contract_status") != "verified":
    raise SystemExit("expected deterministic transport_evidence_source_contract_status marker in policy report")
if policy_payload.get("reconciliation_reason_taxonomy_version") != "kamn.runtime.block-reconciliation-partition-rejoin-reason-taxonomy.v1":
    raise SystemExit("expected deterministic reconciliation_reason_taxonomy_version marker in policy report")
if policy_payload.get("reconciliation_reason_codes_csv") != "reconciliation_partition_transition_failed,reconciliation_rejoin_transition_failed,reconciliation_publish_drop_recovery_failed,reconciliation_peer_churn_recovery_failed,reconciliation_split_head_unresolved,reconciliation_replay_instability,reconciliation_fixture_contract_failed,reconciliation_unclassified_scenario_failed,reconciliation_runtime_budget_exceeded,reconciliation_ci_fast_gate_failed":
    raise SystemExit("expected deterministic reconciliation_reason_codes_csv marker in policy report")
if policy_payload.get("reconciliation_consistency_reason_taxonomy_version") != "kamn.runtime.snapshot-wal-consistency-reason-taxonomy.v1":
    raise SystemExit("expected deterministic reconciliation_consistency_reason_taxonomy_version marker in policy report")
if policy_payload.get("reconciliation_consistency_reason_codes_csv") != "snapshot_wal_lineage_diverged,snapshot_wal_checkpoint_stale,consistency_classification_mismatch":
    raise SystemExit("expected deterministic reconciliation_consistency_reason_codes_csv marker in policy report")
PY

if ! grep -q "check_block_reconciliation_partition_rejoin_live_policy.sh" "$CONTRACT_LANE"; then
  echo "expected block reconciliation partition/rejoin contract lane to compose policy checker" >&2
  exit 1
fi
if ! grep -q "validate_block_reconciliation_partition_rejoin_live.sh" "$CONTRACT_LANE"; then
  echo "expected block reconciliation partition/rejoin contract lane to compose validation lane" >&2
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
  echo "expected block reconciliation partition/rejoin contract lane to reject invalid ci-fast-gate value" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_ci_fast_gate_output" | grep -q 'ci-fast-gate must be PASS or FAIL'; then
  echo "expected deterministic invalid ci-fast-gate marker for block reconciliation partition/rejoin contract lane" >&2
  exit 1
fi

echo "block reconciliation partition/rejoin contract lane tests passed."
