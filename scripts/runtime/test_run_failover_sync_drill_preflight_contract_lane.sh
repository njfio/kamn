#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PREFLIGHT_LANE="$ROOT_DIR/scripts/runtime/run_failover_sync_drill_preflight_contract_lane.sh"
SHARED_CONTRACT="$ROOT_DIR/scripts/runtime/failover_sync_drill_preflight_contract_lane_contract.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/runtime_failover_sync_drill_preflight_contract_lane.json"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"
TMP_REPORT="$(mktemp)"
TMP_DIR="$(mktemp -d)"
trap 'rm -f "$TMP_REPORT"; rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$PREFLIGHT_LANE" ]; then
  echo "expected failover/sync preflight contract lane script to be executable" >&2
  exit 1
fi

if [ ! -x "$SHARED_CONTRACT" ]; then
  echo "expected failover/sync preflight shared contract module to be executable" >&2
  exit 1
fi

lane_output="$(bash "$PREFLIGHT_LANE" --output-json "$TMP_REPORT")"
if ! printf '%s\n' "$lane_output" | grep -q "failover/sync preflight contract lane tests passed."; then
  echo "expected failover/sync preflight contract lane success marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text())
if payload.get("schema_version") != "kamn.runtime.failover-sync-drill-report.v1":
    raise SystemExit("unexpected failover/sync preflight report schema")
if payload.get("lane") != "preflight":
    raise SystemExit("expected preflight lane report")
if payload.get("status") != "pass":
    raise SystemExit("expected preflight lane to pass")
if payload.get("failover_promotion_gate_status") != "verified":
    raise SystemExit("expected failover_promotion_gate_status=verified")
if payload.get("live_node_drift_parity_status") != "verified":
    raise SystemExit("expected live_node_drift_parity_status=verified")
if payload.get("ci_local_promotion_budget_boundary_status") != "verified":
    raise SystemExit("expected ci_local_promotion_budget_boundary_status=verified")
if payload.get("failover_readiness_reason_taxonomy_version") != "kamn.runtime.failover-readiness-reason-taxonomy.v1":
    raise SystemExit("expected deterministic failover_readiness_reason_taxonomy_version marker")
if payload.get("failover_readiness_reason_codes_csv") != "failover_readiness_progress_stalled,live_node_drift_marker_parity_mismatch,ci_local_promotion_budget_boundary_exceeded":
    raise SystemExit("expected deterministic failover_readiness_reason_codes_csv marker")
if payload.get("drift_taxonomy_mapping_status") != "verified":
    raise SystemExit("expected drift_taxonomy_mapping_status=verified")
if payload.get("runbook_marker_parity_status") != "verified":
    raise SystemExit("expected runbook_marker_parity_status=verified")
if payload.get("drift_taxonomy_runbook_reason_taxonomy_version") != "kamn.runtime.failover-drift-taxonomy-runbook-reason-taxonomy.v1":
    raise SystemExit("expected deterministic drift_taxonomy_runbook_reason_taxonomy_version marker")
if payload.get("drift_taxonomy_runbook_reason_codes_csv") != "drift_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch":
    raise SystemExit("expected deterministic drift_taxonomy_runbook_reason_codes_csv marker")
PY

policy_report="$TMP_DIR/failover-sync-preflight-policy.json"
policy_output="$(
  bash "$SHARED_CONTRACT" check-policy \
    --report-file "$TMP_REPORT" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$policy_report"
)"
if ! printf '%s\n' "$policy_output" | grep -q '^status=ok$'; then
  echo "expected failover/sync preflight policy checker status=ok marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^final_decision=GO$'; then
  echo "expected failover/sync preflight policy checker final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^failover_sync_drift_policy_status=verified$'; then
  echo "expected failover/sync preflight policy checker status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^reason_taxonomy_version=kamn.runtime.failover-readiness-reason-taxonomy.v1$'; then
  echo "expected deterministic failover/sync preflight policy reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^reason_codes_csv=failover_readiness_progress_stalled,live_node_drift_marker_parity_mismatch,ci_local_promotion_budget_boundary_exceeded$'; then
  echo "expected deterministic failover/sync preflight policy reason codes marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^drift_taxonomy_reason_taxonomy_version=kamn.runtime.failover-drift-taxonomy-runbook-reason-taxonomy.v1$'; then
  echo "expected deterministic failover/sync preflight drift taxonomy reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^drift_taxonomy_reason_codes_csv=drift_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch$'; then
  echo "expected deterministic failover/sync preflight drift taxonomy reason codes marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^promotion_decision_reason_mapping_status=verified$'; then
  echo "expected failover/sync preflight promotion decision reason mapping status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^promotion_decision_reason_taxonomy_version=kamn.runtime.failover-promotion-decision-reason-taxonomy.v1$'; then
  echo "expected deterministic failover/sync preflight promotion decision reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^promotion_decision_reason_codes_csv=failover_readiness_progress_stalled,live_node_drift_marker_parity_mismatch,ci_local_promotion_budget_boundary_exceeded,drift_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch,ci_fast_gate_failed,failover_sync_drift_policy_expected_decision_mismatch,failover_sync_drift_policy_violation$'; then
  echo "expected deterministic failover/sync preflight promotion decision reason codes marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^promotion_decision_reason_code=none$'; then
  echo "expected deterministic failover/sync preflight promotion decision reason code marker" >&2
  exit 1
fi

python3 - "$policy_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.failover-sync-drill-preflight-policy-report.v1":
    raise SystemExit("unexpected failover/sync preflight policy report schema")
if payload.get("status") != "pass":
    raise SystemExit("expected failover/sync preflight policy status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected failover/sync preflight policy final_decision=GO")
if payload.get("failover_sync_drift_policy_status") != "verified":
    raise SystemExit("expected failover/sync preflight policy status marker")
if payload.get("reason_codes") != ["none"]:
    raise SystemExit("expected failover/sync preflight policy success reason code ['none']")
if payload.get("reason_taxonomy_version") != "kamn.runtime.failover-readiness-reason-taxonomy.v1":
    raise SystemExit("expected deterministic failover/sync preflight reason taxonomy marker")
if payload.get("reason_codes_csv") != "failover_readiness_progress_stalled,live_node_drift_marker_parity_mismatch,ci_local_promotion_budget_boundary_exceeded":
    raise SystemExit("expected deterministic failover/sync preflight reason codes marker")
if payload.get("drift_taxonomy_reason_taxonomy_version") != "kamn.runtime.failover-drift-taxonomy-runbook-reason-taxonomy.v1":
    raise SystemExit("expected deterministic failover/sync preflight drift taxonomy reason taxonomy marker")
if payload.get("drift_taxonomy_reason_codes_csv") != "drift_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch":
    raise SystemExit("expected deterministic failover/sync preflight drift taxonomy reason codes marker")
if payload.get("promotion_decision_reason_mapping_status") != "verified":
    raise SystemExit("expected failover/sync preflight promotion decision reason mapping status marker")
if payload.get("promotion_decision_reason_taxonomy_version") != "kamn.runtime.failover-promotion-decision-reason-taxonomy.v1":
    raise SystemExit("expected deterministic failover/sync preflight promotion decision reason taxonomy marker")
if payload.get("promotion_decision_reason_codes_csv") != "failover_readiness_progress_stalled,live_node_drift_marker_parity_mismatch,ci_local_promotion_budget_boundary_exceeded,drift_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch,ci_fast_gate_failed,failover_sync_drift_policy_expected_decision_mismatch,failover_sync_drift_policy_violation":
    raise SystemExit("expected deterministic failover/sync preflight promotion decision reason codes marker")
if payload.get("promotion_decision_reason_code") != "none":
    raise SystemExit("expected deterministic failover/sync preflight promotion decision reason code marker")
PY

convergence_report="$TMP_DIR/failover-sync-preflight-convergence.json"
convergence_output="$(
  bash "$SHARED_CONTRACT" check-evidence-convergence \
    --report-file "$TMP_REPORT" \
    --policy-file "$policy_report" \
    --output-json "$convergence_report"
)"
if ! printf '%s\n' "$convergence_output" | grep -q '^status=ok$'; then
  echo "expected failover/sync preflight convergence checker status=ok marker" >&2
  exit 1
fi
if ! printf '%s\n' "$convergence_output" | grep -q '^final_decision=GO$'; then
  echo "expected failover/sync preflight convergence checker final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$convergence_output" | grep -q '^evidence_convergence_status=verified$'; then
  echo "expected failover/sync preflight convergence checker evidence status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$convergence_output" | grep -q '^promotion_decision_reason_mapping_status=verified$'; then
  echo "expected failover/sync preflight convergence checker promotion mapping status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$convergence_output" | grep -q '^reason_taxonomy_version=kamn.runtime.failover-evidence-convergence-reason-taxonomy.v1$'; then
  echo "expected deterministic failover/sync preflight convergence reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$convergence_output" | grep -q '^reason_codes_csv=failover_evidence_link_missing,failover_evidence_payload_tamper_detected,promotion_decision_reason_mapping_mismatch$'; then
  echo "expected deterministic failover/sync preflight convergence reason codes marker" >&2
  exit 1
fi
if ! printf '%s\n' "$convergence_output" | grep -q '^reason_codes_value=none$'; then
  echo "expected failover/sync preflight convergence checker reason_codes_value=none marker" >&2
  exit 1
fi

python3 - "$convergence_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.failover-sync-drill-preflight-convergence-report.v1":
    raise SystemExit("unexpected failover/sync preflight convergence report schema")
if payload.get("status") != "pass":
    raise SystemExit("expected failover/sync preflight convergence report status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected failover/sync preflight convergence report final_decision=GO")
if payload.get("evidence_convergence_status") != "verified":
    raise SystemExit("expected failover/sync preflight convergence evidence status marker")
if payload.get("promotion_decision_reason_mapping_status") != "verified":
    raise SystemExit("expected failover/sync preflight convergence promotion mapping status marker")
if payload.get("reason_taxonomy_version") != "kamn.runtime.failover-evidence-convergence-reason-taxonomy.v1":
    raise SystemExit("expected deterministic failover/sync preflight convergence reason taxonomy marker")
if payload.get("reason_codes_csv") != "failover_evidence_link_missing,failover_evidence_payload_tamper_detected,promotion_decision_reason_mapping_mismatch":
    raise SystemExit("expected deterministic failover/sync preflight convergence reason codes marker")
if payload.get("reason_codes") != ["none"]:
    raise SystemExit("expected failover/sync preflight convergence success reason code ['none']")
PY

missing_link_policy="$TMP_DIR/failover-sync-preflight-policy.missing-link.json"
cp "$policy_report" "$missing_link_policy"
python3 - "$missing_link_policy" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload.pop("report_file", None)
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
missing_link_output_first="$(
  bash "$SHARED_CONTRACT" check-evidence-convergence \
    --report-file "$TMP_REPORT" \
    --policy-file "$missing_link_policy" \
    --output-json "$TMP_DIR/failover-sync-preflight-convergence.missing-link.first.json" 2>&1
)"
missing_link_code_first=$?
missing_link_output_second="$(
  bash "$SHARED_CONTRACT" check-evidence-convergence \
    --report-file "$TMP_REPORT" \
    --policy-file "$missing_link_policy" \
    --output-json "$TMP_DIR/failover-sync-preflight-convergence.missing-link.second.json" 2>&1
)"
missing_link_code_second=$?
set -e
if [ "$missing_link_code_first" -eq 0 ] || [ "$missing_link_code_second" -eq 0 ]; then
  echo "expected failover/sync preflight convergence checker to reject missing evidence link marker deterministically" >&2
  exit 1
fi
if ! printf '%s\n' "$missing_link_output_first" | grep -q 'failover_evidence_link_missing:report_file'; then
  echo "expected deterministic failover/sync preflight missing evidence link reason output on first run" >&2
  exit 1
fi
if ! printf '%s\n' "$missing_link_output_second" | grep -q 'failover_evidence_link_missing:report_file'; then
  echo "expected deterministic failover/sync preflight missing evidence link reason output on second run" >&2
  exit 1
fi

python3 - \
  "$TMP_DIR/failover-sync-preflight-convergence.missing-link.first.json" \
  "$TMP_DIR/failover-sync-preflight-convergence.missing-link.second.json" <<'PY'
import json
import pathlib
import sys

first_payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
second_payload = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
first_reasons = first_payload.get("reason_codes")
second_reasons = second_payload.get("reason_codes")
if not first_reasons:
    raise SystemExit("expected failover/sync preflight missing-link convergence report to include non-empty reason codes")
if first_reasons != second_reasons:
    raise SystemExit("expected deterministic failover/sync preflight convergence reason-code ordering across repeated missing-link checks")
if "failover_evidence_link_missing:report_file" not in first_reasons:
    raise SystemExit("expected failover_evidence_link_missing:report_file reason code in failover/sync preflight missing-link convergence reports")
PY

tampered_mapping_policy="$TMP_DIR/failover-sync-preflight-policy.tampered-mapping.json"
cp "$policy_report" "$tampered_mapping_policy"
python3 - "$tampered_mapping_policy" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["promotion_decision_reason_code"] = "live_node_drift_marker_parity_mismatch"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_mapping_output="$(
  bash "$SHARED_CONTRACT" check-evidence-convergence \
    --report-file "$TMP_REPORT" \
    --policy-file "$tampered_mapping_policy" \
    --output-json "$TMP_DIR/failover-sync-preflight-convergence.tampered-mapping.json" 2>&1
)"
tampered_mapping_code=$?
set -e
if [ "$tampered_mapping_code" -eq 0 ]; then
  echo "expected failover/sync preflight convergence checker to reject tampered promotion decision reason mapping" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_mapping_output" | grep -q 'promotion_decision_reason_mapping_mismatch'; then
  echo "expected deterministic failover/sync preflight promotion reason mapping mismatch marker" >&2
  exit 1
fi

tampered_payload_policy="$TMP_DIR/failover-sync-preflight-policy.tampered-payload.json"
cp "$policy_report" "$tampered_payload_policy"
python3 - "$tampered_payload_policy" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["reason_codes"] = "none"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_payload_output="$(
  bash "$SHARED_CONTRACT" check-evidence-convergence \
    --report-file "$TMP_REPORT" \
    --policy-file "$tampered_payload_policy" \
    --output-json "$TMP_DIR/failover-sync-preflight-convergence.tampered-payload.json" 2>&1
)"
tampered_payload_code=$?
set -e
if [ "$tampered_payload_code" -eq 0 ]; then
  echo "expected failover/sync preflight convergence checker to reject tampered convergence payload" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_payload_output" | grep -q 'failover_evidence_payload_tamper_detected'; then
  echo "expected deterministic failover/sync preflight convergence payload tamper marker" >&2
  exit 1
fi

missing_marker_report="$TMP_DIR/failover-sync-preflight-summary.missing-marker.json"
cp "$TMP_REPORT" "$missing_marker_report"
python3 - "$missing_marker_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload.pop("live_node_drift_parity_status", None)
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
missing_marker_output="$(
  bash "$SHARED_CONTRACT" check-policy \
    --report-file "$missing_marker_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/failover-sync-preflight-policy.missing-marker.json" 2>&1
)"
missing_marker_code=$?
set -e
if [ "$missing_marker_code" -eq 0 ]; then
  echo "expected missing live-node drift marker report to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$missing_marker_output" | grep -q 'missing required report fields: live_node_drift_parity_status'; then
  echo "expected deterministic missing marker reason output for failover/sync preflight policy checker" >&2
  exit 1
fi

drift_marker_report="$TMP_DIR/failover-sync-preflight-summary.marker-drift.json"
cp "$TMP_REPORT" "$drift_marker_report"
python3 - "$drift_marker_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["live_node_drift_parity_status"] = "drifted"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
drift_marker_output_first="$(
  bash "$SHARED_CONTRACT" check-policy \
    --report-file "$drift_marker_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/failover-sync-preflight-policy.marker-drift.first.json" 2>&1
)"
drift_marker_code_first=$?
drift_marker_output_second="$(
  bash "$SHARED_CONTRACT" check-policy \
    --report-file "$drift_marker_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/failover-sync-preflight-policy.marker-drift.second.json" 2>&1
)"
drift_marker_code_second=$?
set -e
if [ "$drift_marker_code_first" -eq 0 ] || [ "$drift_marker_code_second" -eq 0 ]; then
  echo "expected live-node drift parity mismatch report to fail policy checker deterministically" >&2
  exit 1
fi
if ! printf '%s\n' "$drift_marker_output_first" | grep -q 'live_node_drift_marker_parity_mismatch'; then
  echo "expected deterministic live-node drift parity mismatch reason output on first run" >&2
  exit 1
fi
if ! printf '%s\n' "$drift_marker_output_second" | grep -q 'live_node_drift_marker_parity_mismatch'; then
  echo "expected deterministic live-node drift parity mismatch reason output on second run" >&2
  exit 1
fi

python3 - \
  "$TMP_DIR/failover-sync-preflight-policy.marker-drift.first.json" \
  "$TMP_DIR/failover-sync-preflight-policy.marker-drift.second.json" <<'PY'
import json
import pathlib
import sys

first_payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
second_payload = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
first_reasons = first_payload.get("reason_codes")
second_reasons = second_payload.get("reason_codes")
if not first_reasons:
    raise SystemExit("expected failover/sync preflight marker-drift policy report to include non-empty reason codes")
if first_reasons != second_reasons:
    raise SystemExit("expected deterministic failover/sync preflight reason-code ordering across repeated marker-drift checks")
if "live_node_drift_marker_parity_mismatch" not in first_reasons:
    raise SystemExit("expected live_node_drift_marker_parity_mismatch reason code in failover/sync preflight marker-drift policy reports")
PY

taxonomy_drift_report="$TMP_DIR/failover-sync-preflight-summary.taxonomy-drift.json"
cp "$TMP_REPORT" "$taxonomy_drift_report"
python3 - "$taxonomy_drift_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["drift_taxonomy_mapping_status"] = "drifted"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
taxonomy_drift_output_first="$(
  bash "$SHARED_CONTRACT" check-policy \
    --report-file "$taxonomy_drift_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/failover-sync-preflight-policy.taxonomy-drift.first.json" 2>&1
)"
taxonomy_drift_code_first=$?
taxonomy_drift_output_second="$(
  bash "$SHARED_CONTRACT" check-policy \
    --report-file "$taxonomy_drift_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/failover-sync-preflight-policy.taxonomy-drift.second.json" 2>&1
)"
taxonomy_drift_code_second=$?
set -e
if [ "$taxonomy_drift_code_first" -eq 0 ] || [ "$taxonomy_drift_code_second" -eq 0 ]; then
  echo "expected drift taxonomy mapping drift report to fail policy checker deterministically" >&2
  exit 1
fi
if ! printf '%s\n' "$taxonomy_drift_output_first" | grep -q 'drift_taxonomy_mapping_drift_detected'; then
  echo "expected deterministic drift taxonomy mapping drift reason output on first run" >&2
  exit 1
fi
if ! printf '%s\n' "$taxonomy_drift_output_second" | grep -q 'drift_taxonomy_mapping_drift_detected'; then
  echo "expected deterministic drift taxonomy mapping drift reason output on second run" >&2
  exit 1
fi

python3 - \
  "$TMP_DIR/failover-sync-preflight-policy.taxonomy-drift.first.json" \
  "$TMP_DIR/failover-sync-preflight-policy.taxonomy-drift.second.json" <<'PY'
import json
import pathlib
import sys

first_payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
second_payload = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
first_reasons = first_payload.get("reason_codes")
second_reasons = second_payload.get("reason_codes")
if not first_reasons:
    raise SystemExit("expected failover/sync preflight taxonomy-drift policy report to include non-empty reason codes")
if first_reasons != second_reasons:
    raise SystemExit("expected deterministic failover/sync preflight reason-code ordering across repeated taxonomy-drift checks")
if "drift_taxonomy_mapping_drift_detected" not in first_reasons:
    raise SystemExit("expected drift_taxonomy_mapping_drift_detected reason code in failover/sync preflight taxonomy-drift policy reports")
PY

runbook_divergence_file="$TMP_DIR/kolme_devnet_ops.marker-divergence.md"
cat > "$runbook_divergence_file" <<'EOF'
# marker divergence fixture

This fixture intentionally omits required failover drift taxonomy marker declarations.
EOF

set +e
runbook_divergence_output="$(
  bash "$SHARED_CONTRACT" check-policy \
    --report-file "$TMP_REPORT" \
    --runbook-file "$runbook_divergence_file" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/failover-sync-preflight-policy.runbook-divergence.json" 2>&1
)"
runbook_divergence_code=$?
set -e
if [ "$runbook_divergence_code" -eq 0 ]; then
  echo "expected runbook marker divergence fixture to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$runbook_divergence_output" | grep -q 'runbook_marker_parity_mismatch'; then
  echo "expected deterministic runbook marker parity mismatch reason output for failover/sync preflight policy checker" >&2
  exit 1
fi

set +e
over_budget_output="$(
  bash "$PREFLIGHT_LANE" \
    --skip-suite \
    --simulate-delay-seconds 1 \
    --max-seconds 0 \
    --output-json "$TMP_REPORT" 2>&1
)"
over_budget_code=$?
set -e

if [ "$over_budget_code" -eq 0 ]; then
  echo "expected failover/sync preflight budget guard to fail over-budget run" >&2
  exit 1
fi

# Regression: #788
if ! printf '%s\n' "$over_budget_output" | grep -q "exceeded runtime budget"; then
  echo "expected failover/sync preflight budget overrun signal" >&2
  exit 1
fi

set +e
drift_output="$(
  bash "$PREFLIGHT_LANE" \
    --skip-suite \
    --simulate-live-node-drift \
    --max-seconds 5 \
    --output-json "$TMP_REPORT" 2>&1
)"
drift_code=$?
set -e

if [ "$drift_code" -eq 0 ]; then
  echo "expected failover/sync preflight live-node drift simulation to fail closed" >&2
  exit 1
fi

if ! printf '%s\n' "$drift_output" | grep -q "live_node_drift_marker_parity_mismatch"; then
  echo "expected failover/sync preflight live-node drift mismatch reason marker" >&2
  exit 1
fi

set +e
stall_output="$(
  bash "$PREFLIGHT_LANE" \
    --skip-suite \
    --simulate-failover-stall \
    --max-seconds 5 \
    --output-json "$TMP_REPORT" 2>&1
)"
stall_code=$?
set -e

if [ "$stall_code" -eq 0 ]; then
  echo "expected failover/sync preflight failover stall simulation to fail closed" >&2
  exit 1
fi

if ! printf '%s\n' "$stall_output" | grep -q "failover_readiness_progress_stalled"; then
  echo "expected failover/sync preflight failover stall reason marker" >&2
  exit 1
fi

set +e
ci_boundary_output="$(
  bash "$PREFLIGHT_LANE" \
    --skip-suite \
    --simulate-delay-seconds 1 \
    --max-seconds 5 \
    --ci-local-promotion-max-seconds 0 \
    --output-json "$TMP_REPORT" 2>&1
)"
ci_boundary_code=$?
set -e

if [ "$ci_boundary_code" -eq 0 ]; then
  echo "expected failover/sync preflight ci-local promotion boundary to fail closed" >&2
  exit 1
fi

if ! printf '%s\n' "$ci_boundary_output" | grep -q "ci_local_promotion_budget_boundary_exceeded"; then
  echo "expected failover/sync preflight ci-local promotion boundary reason marker" >&2
  exit 1
fi

if [ ! -L "$PREFLIGHT_LANE" ]; then
  echo "expected failover/sync preflight wrapper to be a dispatcher symlink" >&2
  exit 1
fi

if [ "$(readlink "$PREFLIGHT_LANE")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected failover/sync preflight wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$PREFLIGHT_LANE")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected failover/sync preflight wrapper to resolve runtime manifest via dispatcher" >&2
  exit 1
fi

if ! grep -Fq "failover_sync_drill_preflight_contract_lane_contract.sh" "$MANIFEST_FILE"; then
  echo "expected failover/sync preflight manifest to dispatch shared contract module" >&2
  exit 1
fi

echo "failover/sync preflight contract lane script tests passed."
