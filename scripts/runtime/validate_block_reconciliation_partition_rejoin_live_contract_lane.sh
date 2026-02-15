#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_block_reconciliation_partition_rejoin_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_block_reconciliation_partition_rejoin_live_policy.sh"
STRATEGY_DOC="$ROOT_DIR/docs/ci/strategy.md"

output_json=""
policy_output_json=""
max_seconds="${KAMN_BLOCK_RECONCILIATION_PARTITION_REJOIN_LIVE_CONTRACT_MAX_SECONDS:-240}"
ci_fast_gate="PASS"
mode="dry-run"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-json)
      output_json="${2:-}"
      shift 2
      ;;
    --policy-output-json)
      policy_output_json="${2:-}"
      shift 2
      ;;
    --max-seconds)
      max_seconds="${2:-}"
      shift 2
      ;;
    --ci-fast-gate)
      ci_fast_gate="${2:-}"
      shift 2
      ;;
    --mode)
      mode="${2:-}"
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
if [[ "$ci_fast_gate" != "PASS" && "$ci_fast_gate" != "FAIL" ]]; then
  echo "ci-fast-gate must be PASS or FAIL" >&2
  exit 1
fi
if [[ "$mode" != "dry-run" && "$mode" != "run" ]]; then
  echo "mode must be dry-run or run" >&2
  exit 1
fi
for required_exec in "$VALIDATION_SCRIPT" "$POLICY_CHECKER"; do
  if [ ! -x "$required_exec" ]; then
    echo "expected required executable script '$required_exec'" >&2
    exit 1
  fi
done
if [ ! -f "$STRATEGY_DOC" ]; then
  echo "expected required documentation file '$STRATEGY_DOC'" >&2
  exit 1
fi

start_epoch="$(date +%s)"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

summary_report="$TMP_DIR/block-reconciliation-partition-rejoin-live-summary.json"
policy_report="$TMP_DIR/block-reconciliation-partition-rejoin-live-policy.json"
tampered_report="$TMP_DIR/block-reconciliation-partition-rejoin-live-summary.tampered.json"

validation_output="$(
  bash "$VALIDATION_SCRIPT" \
    --mode "$mode" \
    --max-seconds "$max_seconds" \
    --ci-fast-gate "$ci_fast_gate" \
    --output-json "$summary_report"
)"
if ! printf '%s\n' "$validation_output" | grep -q '^status=pass$'; then
  echo "expected block reconciliation partition/rejoin validation status=pass marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^final_decision=GO$'; then
  echo "expected block reconciliation partition/rejoin validation final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^fast_gate_exclusion_status=verified$'; then
  echo "expected block reconciliation partition/rejoin validation fast-gate exclusion marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^block_reconciliation_partition_status=verified$'; then
  echo "expected block reconciliation partition marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^block_reconciliation_rejoin_status=verified$'; then
  echo "expected block reconciliation rejoin marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^canonical_convergence_status=verified$'; then
  echo "expected canonical convergence marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^runtime_transport_mode=libp2p_transport_fed$'; then
  echo "expected runtime transport mode marker for block reconciliation partition/rejoin validation" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^reconciliation_reason_taxonomy_status=verified$'; then
  echo "expected reconciliation reason taxonomy status marker for block reconciliation partition/rejoin validation" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^snapshot_wal_reconciliation_status=verified$'; then
  echo "expected snapshot-vs-wal reconciliation status marker for block reconciliation partition/rejoin validation" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^consistency_classification_status=verified$'; then
  echo "expected consistency classification status marker for block reconciliation partition/rejoin validation" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^reconciliation_consistency_reason_taxonomy_version=kamn.runtime.snapshot-wal-consistency-reason-taxonomy.v1$'; then
  echo "expected reconciliation consistency reason taxonomy version marker for block reconciliation partition/rejoin validation" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^reconciliation_consistency_reason_codes_csv=snapshot_wal_lineage_diverged,snapshot_wal_checkpoint_stale,consistency_classification_mismatch$'; then
  echo "expected reconciliation consistency reason taxonomy csv marker for block reconciliation partition/rejoin validation" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^reconciliation_reason_codes=none$'; then
  echo "expected deterministic reconciliation reason-code matrix marker for block reconciliation partition/rejoin validation" >&2
  exit 1
fi

policy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$summary_report" \
    --expected-final-decision GO \
    --ci-fast-gate "$ci_fast_gate" \
    --output-json "$policy_report"
)"
if ! printf '%s\n' "$policy_output" | grep -q '^status=ok$'; then
  echo "expected block reconciliation partition/rejoin policy status=ok marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^final_decision=GO$'; then
  echo "expected block reconciliation partition/rejoin policy final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^block_reconciliation_partition_rejoin_policy_status=verified$'; then
  echo "expected block reconciliation partition/rejoin policy status marker" >&2
  exit 1
fi

cp "$summary_report" "$tampered_report"
python3 - "$tampered_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["fast_gate_exclusion_status"] = "tampered"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_policy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_report" \
    --expected-final-decision GO \
    --ci-fast-gate "$ci_fast_gate" \
    --output-json "$TMP_DIR/block-reconciliation-partition-rejoin-live-policy.tampered.json" 2>&1
)"
tampered_policy_code=$?
set -e
if [ "$tampered_policy_code" -eq 0 ]; then
  echo "expected tampered block reconciliation partition/rejoin report to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_policy_output" | grep -q 'block_reconciliation_partition_rejoin_policy_fast_gate_exclusion_mismatch'; then
  echo "expected deterministic fail-closed reason for tampered block reconciliation partition/rejoin report" >&2
  exit 1
fi

for required_ref in \
  "validate_block_reconciliation_partition_rejoin_live.sh" \
  "check_block_reconciliation_partition_rejoin_live_policy.sh" \
  "validate_block_reconciliation_partition_rejoin_live_contract_lane.sh" \
  "test_validate_block_reconciliation_partition_rejoin_live.sh" \
  "test_check_block_reconciliation_partition_rejoin_live_policy.sh" \
  "test_validate_block_reconciliation_partition_rejoin_live_contract_lane.sh"; do
  if ! grep -q "$required_ref" "$STRATEGY_DOC"; then
    echo "expected CI strategy docs to reference $required_ref" >&2
    exit 1
  fi
done
if ! grep -q "block reconciliation partition/rejoin run-mode commands remain excluded from ci-fast-gate and ci-tools fast mode." "$STRATEGY_DOC"; then
  echo "expected CI strategy docs to include block reconciliation partition/rejoin run-mode exclusion marker" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "block reconciliation partition/rejoin contract lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

lane_report="$TMP_DIR/block-reconciliation-partition-rejoin-live-contract-lane-report.json"
python3 - "$summary_report" "$policy_report" "$lane_report" "$elapsed_seconds" "$max_seconds" "$mode" <<'PY'
import json
import pathlib
import sys

summary_report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
policy_report = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
lane_report_file = pathlib.Path(sys.argv[3])
elapsed_seconds = int(sys.argv[4])
max_seconds = int(sys.argv[5])
mode = sys.argv[6]

if summary_report.get("schema_version") != "kamn.runtime.block-reconciliation-partition-rejoin-live-report.v1":
    raise SystemExit("unexpected block reconciliation partition/rejoin summary schema")
if policy_report.get("schema_version") != "kamn.runtime.block-reconciliation-partition-rejoin-live-policy-report.v1":
    raise SystemExit("unexpected block reconciliation partition/rejoin policy schema")
if summary_report.get("final_decision") != "GO":
    raise SystemExit("expected block reconciliation partition/rejoin summary final_decision=GO")
if policy_report.get("final_decision") != "GO":
    raise SystemExit("expected block reconciliation partition/rejoin policy final_decision=GO")
if summary_report.get("runtime_transport_mode") != "libp2p_transport_fed":
    raise SystemExit("expected summary runtime_transport_mode=libp2p_transport_fed")
if summary_report.get("transport_state_transition_status") != "verified":
    raise SystemExit("expected summary transport_state_transition_status=verified")
if summary_report.get("reconciliation_reason_taxonomy_status") != "verified":
    raise SystemExit("expected summary reconciliation_reason_taxonomy_status=verified")
if summary_report.get("reconciliation_reason_codes") != ["none"]:
    raise SystemExit("expected summary reconciliation_reason_codes=['none']")
if summary_report.get("snapshot_wal_reconciliation_status") != "verified":
    raise SystemExit("expected summary snapshot_wal_reconciliation_status=verified")
if summary_report.get("consistency_classification_status") != "verified":
    raise SystemExit("expected summary consistency_classification_status=verified")
if summary_report.get("reconciliation_consistency_reason_taxonomy_version") != "kamn.runtime.snapshot-wal-consistency-reason-taxonomy.v1":
    raise SystemExit("expected summary reconciliation_consistency_reason_taxonomy_version marker")
if summary_report.get("reconciliation_consistency_reason_codes_csv") != "snapshot_wal_lineage_diverged,snapshot_wal_checkpoint_stale,consistency_classification_mismatch":
    raise SystemExit("expected summary reconciliation_consistency_reason_codes_csv marker")

lane_report = {
    "schema_version": "kamn.runtime.block-reconciliation-partition-rejoin-live-contract-lane-report.v1",
    "status": "pass",
    "final_decision": "GO",
    "lane_mode": mode,
    "block_reconciliation_partition_rejoin_contract_status": "verified",
    "block_reconciliation_partition_rejoin_policy_status": policy_report.get(
        "block_reconciliation_partition_rejoin_policy_status"
    ),
    "docs_contract_status": "verified",
    "runtime_transport_mode_status": "verified",
    "reconciliation_reason_taxonomy_status": "verified",
    "snapshot_wal_reconciliation_status": summary_report.get(
        "snapshot_wal_reconciliation_status"
    ),
    "consistency_classification_status": summary_report.get(
        "consistency_classification_status"
    ),
    "reconciliation_consistency_reason_taxonomy_version": summary_report.get(
        "reconciliation_consistency_reason_taxonomy_version"
    ),
    "reconciliation_consistency_reason_codes_csv": summary_report.get(
        "reconciliation_consistency_reason_codes_csv"
    ),
    "fail_closed_status": "verified",
    "fail_closed_reason_code": "block_reconciliation_partition_rejoin_policy_fast_gate_exclusion_mismatch",
    "performance_budget_status": "verified",
    "elapsed_seconds": elapsed_seconds,
    "max_seconds": max_seconds,
}
lane_report_file.write_text(json.dumps(lane_report, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

if [[ -n "$output_json" ]]; then
  cp "$lane_report" "$output_json"
fi
if [[ -n "$policy_output_json" ]]; then
  cp "$policy_report" "$policy_output_json"
fi

echo "status=pass"
echo "final_decision=GO"
echo "lane_mode=$mode"
echo "block_reconciliation_partition_rejoin_contract_status=verified"
echo "block_reconciliation_partition_rejoin_policy_status=verified"
echo "docs_contract_status=verified"
echo "runtime_transport_mode_status=verified"
echo "reconciliation_reason_taxonomy_status=verified"
echo "snapshot_wal_reconciliation_status=verified"
echo "consistency_classification_status=verified"
echo "reconciliation_consistency_reason_taxonomy_version=kamn.runtime.snapshot-wal-consistency-reason-taxonomy.v1"
echo "reconciliation_consistency_reason_codes_csv=snapshot_wal_lineage_diverged,snapshot_wal_checkpoint_stale,consistency_classification_mismatch"
echo "fail_closed_status=verified"
echo "fail_closed_reason_code=block_reconciliation_partition_rejoin_policy_fast_gate_exclusion_mismatch"
echo "performance_budget_status=verified"
