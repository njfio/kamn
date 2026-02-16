#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_block_reconciliation_partition_rejoin_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_block_reconciliation_partition_rejoin_live_policy.sh"
TMP_REPORT="$(mktemp)"
TMP_POLICY="$(mktemp)"
TMP_TAMPERED="$(mktemp)"
TMP_TAMPERED_TRANSPORT="$(mktemp)"
TMP_TAMPERED_RECOVERY="$(mktemp)"
TMP_TAMPERED_TAXONOMY="$(mktemp)"
TMP_TAMPERED_CONSISTENCY="$(mktemp)"
TMP_TAMPERED_TRANSPORT_EVIDENCE="$(mktemp)"
TMP_TAMPERED_REASON_CODES_CSV="$(mktemp)"
trap 'rm -f "$TMP_REPORT" "$TMP_POLICY" "$TMP_TAMPERED" "$TMP_TAMPERED_TRANSPORT" "$TMP_TAMPERED_RECOVERY" "$TMP_TAMPERED_TAXONOMY" "$TMP_TAMPERED_CONSISTENCY" "$TMP_TAMPERED_TRANSPORT_EVIDENCE" "$TMP_TAMPERED_REASON_CODES_CSV"' EXIT

if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected block reconciliation partition/rejoin validation script to be executable" >&2
  exit 1
fi
if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected block reconciliation partition/rejoin policy checker script to be executable" >&2
  exit 1
fi

bash "$VALIDATION_SCRIPT" --mode dry-run --ci-fast-gate PASS --output-json "$TMP_REPORT" >/dev/null

policy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$TMP_REPORT" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_POLICY"
)"
if ! printf '%s\n' "$policy_output" | grep -q '^status=ok$'; then
  echo "expected block reconciliation partition/rejoin policy checker status=ok marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^final_decision=GO$'; then
  echo "expected block reconciliation partition/rejoin policy checker final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^block_reconciliation_partition_rejoin_policy_status=verified$'; then
  echo "expected block reconciliation partition/rejoin policy checker status marker" >&2
  exit 1
fi

python3 - "$TMP_POLICY" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.block-reconciliation-partition-rejoin-live-policy-report.v1":
    raise SystemExit("unexpected block reconciliation partition/rejoin policy schema")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected block reconciliation partition/rejoin policy final_decision=GO")
if payload.get("block_reconciliation_partition_rejoin_policy_status") != "verified":
    raise SystemExit("expected block_reconciliation_partition_rejoin_policy_status=verified")
if payload.get("reconciliation_reason_taxonomy_version") != "kamn.runtime.block-reconciliation-partition-rejoin-reason-taxonomy.v1":
    raise SystemExit("expected deterministic reconciliation_reason_taxonomy_version marker")
if payload.get("reconciliation_reason_codes_csv") != "reconciliation_partition_transition_failed,reconciliation_rejoin_transition_failed,reconciliation_publish_drop_recovery_failed,reconciliation_peer_churn_recovery_failed,reconciliation_split_head_unresolved,reconciliation_replay_instability,reconciliation_fixture_contract_failed,reconciliation_unclassified_scenario_failed,reconciliation_runtime_budget_exceeded,reconciliation_ci_fast_gate_failed":
    raise SystemExit("expected deterministic reconciliation_reason_codes_csv marker")
if payload.get("transport_evidence_schema_version") != "kamn.runtime.libp2p-transport-transition-evidence.v1":
    raise SystemExit("expected deterministic transport_evidence_schema_version marker")
if payload.get("transport_evidence_normalization_status") != "verified":
    raise SystemExit("expected deterministic transport_evidence_normalization_status=verified")
if payload.get("transport_evidence_source_contract_status") != "verified":
    raise SystemExit("expected deterministic transport_evidence_source_contract_status=verified")
if payload.get("reconciliation_consistency_reason_taxonomy_version") != "kamn.runtime.snapshot-wal-consistency-reason-taxonomy.v1":
    raise SystemExit("expected deterministic reconciliation_consistency_reason_taxonomy_version marker")
if payload.get("reconciliation_consistency_reason_codes_csv") != "snapshot_wal_lineage_diverged,snapshot_wal_checkpoint_stale,consistency_classification_mismatch":
    raise SystemExit("expected deterministic reconciliation_consistency_reason_codes_csv marker")
PY

cp "$TMP_REPORT" "$TMP_TAMPERED"
python3 - "$TMP_TAMPERED" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["fast_gate_exclusion_status"] = "mismatch-marker"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$TMP_TAMPERED" \
    --expected-final-decision GO \
    --ci-fast-gate PASS 2>&1
)"
tampered_code=$?
set -e
if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered block reconciliation partition/rejoin report to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q 'block_reconciliation_partition_rejoin_policy_fast_gate_exclusion_mismatch'; then
  echo "expected deterministic fail-closed reason for tampered block reconciliation partition/rejoin report" >&2
  exit 1
fi

python3 - "$tampered_output" <<'PY'
import sys

output = sys.argv[1]
failed_checks = ""
for line in output.splitlines():
    if line.startswith("failed_checks="):
        failed_checks = line.split("=", 1)[1]
        break
reason_codes = [entry for entry in failed_checks.split(",") if entry]
if "block_reconciliation_partition_rejoin_policy_fast_gate_exclusion_mismatch" not in reason_codes:
    raise SystemExit("expected parser to recover deterministic block reconciliation partition/rejoin reason code")
PY

cp "$TMP_REPORT" "$TMP_TAMPERED_TRANSPORT"
python3 - "$TMP_TAMPERED_TRANSPORT" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["runtime_transport_mode"] = "in_memory_simulation"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_transport_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$TMP_TAMPERED_TRANSPORT" \
    --expected-final-decision GO \
    --ci-fast-gate PASS 2>&1
)"
tampered_transport_code=$?
set -e
if [ "$tampered_transport_code" -eq 0 ]; then
  echo "expected transport-mode tampered block reconciliation partition/rejoin report to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_transport_output" | grep -q 'block_reconciliation_partition_rejoin_policy_transport_mode_mismatch'; then
  echo "expected deterministic transport-mode mismatch reason for block reconciliation partition/rejoin report" >&2
  exit 1
fi

cp "$TMP_REPORT" "$TMP_TAMPERED_TRANSPORT_EVIDENCE"
python3 - "$TMP_TAMPERED_TRANSPORT_EVIDENCE" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["transport_evidence_normalization_status"] = "tampered"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_transport_evidence_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$TMP_TAMPERED_TRANSPORT_EVIDENCE" \
    --expected-final-decision GO \
    --ci-fast-gate PASS 2>&1
)"
tampered_transport_evidence_code=$?
set -e
if [ "$tampered_transport_evidence_code" -eq 0 ]; then
  echo "expected transport-evidence tampered block reconciliation partition/rejoin report to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_transport_evidence_output" | grep -q 'block_reconciliation_partition_rejoin_policy_transport_evidence_normalization_status_mismatch'; then
  echo "expected deterministic transport-evidence mismatch reason for block reconciliation partition/rejoin report" >&2
  exit 1
fi

cp "$TMP_REPORT" "$TMP_TAMPERED_RECOVERY"
python3 - "$TMP_TAMPERED_RECOVERY" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["head_alignment_status"] = "drifted"
payload["reconciliation_reason_codes"] = ["reconciliation_split_head_unresolved"]
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_recovery_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$TMP_TAMPERED_RECOVERY" \
    --expected-final-decision GO \
    --ci-fast-gate PASS 2>&1
)"
tampered_recovery_code=$?
set -e
if [ "$tampered_recovery_code" -eq 0 ]; then
  echo "expected recovery-criteria tampered block reconciliation partition/rejoin report to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_recovery_output" | grep -q 'block_reconciliation_partition_rejoin_policy_head_alignment_status_mismatch'; then
  echo "expected deterministic head-alignment mismatch reason for block reconciliation partition/rejoin report" >&2
  exit 1
fi

cp "$TMP_REPORT" "$TMP_TAMPERED_TAXONOMY"
python3 - "$TMP_TAMPERED_TAXONOMY" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["reconciliation_consistency_reason_taxonomy_version"] = "tampered-taxonomy"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_taxonomy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$TMP_TAMPERED_TAXONOMY" \
    --expected-final-decision GO \
    --ci-fast-gate PASS 2>&1
)"
tampered_taxonomy_code=$?
set -e
if [ "$tampered_taxonomy_code" -eq 0 ]; then
  echo "expected consistency-taxonomy tampered block reconciliation partition/rejoin report to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_taxonomy_output" | grep -q 'block_reconciliation_partition_rejoin_policy_reconciliation_consistency_reason_taxonomy_version_mismatch'; then
  echo "expected deterministic consistency-taxonomy mismatch reason for block reconciliation partition/rejoin report" >&2
  exit 1
fi

cp "$TMP_REPORT" "$TMP_TAMPERED_CONSISTENCY"
python3 - "$TMP_TAMPERED_CONSISTENCY" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["consistency_classification_status"] = "drifted"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_consistency_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$TMP_TAMPERED_CONSISTENCY" \
    --expected-final-decision GO \
    --ci-fast-gate PASS 2>&1
)"
tampered_consistency_code=$?
set -e
if [ "$tampered_consistency_code" -eq 0 ]; then
  echo "expected consistency-classification tampered block reconciliation partition/rejoin report to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_consistency_output" | grep -q 'block_reconciliation_partition_rejoin_policy_consistency_classification_status_mismatch'; then
  echo "expected deterministic consistency-classification mismatch reason for block reconciliation partition/rejoin report" >&2
  exit 1
fi

cp "$TMP_REPORT" "$TMP_TAMPERED_REASON_CODES_CSV"
python3 - "$TMP_TAMPERED_REASON_CODES_CSV" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["reconciliation_reason_codes_csv"] = "tampered-reasons"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_reason_codes_csv_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$TMP_TAMPERED_REASON_CODES_CSV" \
    --expected-final-decision GO \
    --ci-fast-gate PASS 2>&1
)"
tampered_reason_codes_csv_code=$?
set -e
if [ "$tampered_reason_codes_csv_code" -eq 0 ]; then
  echo "expected reconciliation-reason-csv tampered block reconciliation partition/rejoin report to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_reason_codes_csv_output" | grep -q 'block_reconciliation_partition_rejoin_policy_reconciliation_reason_codes_csv_mismatch'; then
  echo "expected deterministic reconciliation-reason-csv mismatch reason for block reconciliation partition/rejoin report" >&2
  exit 1
fi

echo "block reconciliation partition/rejoin live policy tests passed."
