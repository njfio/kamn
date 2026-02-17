#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
POLICY_SCRIPT="$ROOT_DIR/scripts/runtime/check_local_full_stack_integration_live_policy.sh"
RUNBOOK_DOC="$ROOT_DIR/docs/deploy/kolme_devnet_ops.md"
TMP_REPORT="$(mktemp)"
TMP_POLICY="$(mktemp)"
TMP_TAMPERED="$(mktemp)"
TMP_RUNBOOK_TAXONOMY_DRIFT="$(mktemp)"
TMP_RUNBOOK_PARITY_DRIFT="$(mktemp)"
trap 'rm -f "$TMP_REPORT" "$TMP_POLICY" "$TMP_TAMPERED" "$TMP_RUNBOOK_TAXONOMY_DRIFT" "$TMP_RUNBOOK_PARITY_DRIFT"' EXIT

EXPECTED_RUNTIME_PHASE_REASON_TAXONOMY_VERSION="kamn.runtime.phase-module-extraction-parity-reason-taxonomy.v1"
EXPECTED_RUNTIME_PHASE_REASON_CODES_CSV="runtime_phase_module_parity_drift_detected,runtime_extraction_evidence_output_unstable,ci_local_runtime_phase_parity_budget_boundary_exceeded"
EXPECTED_RUNTIME_MODULE_BOUNDARY_REASON_CODES_CSV="runtime_orchestration_dispatch_boundary_drift_detected,runtime_daemon_phase_boundary_drift_detected,runtime_kolme_live_boundary_drift_detected,ci_local_runtime_module_boundary_budget_boundary_exceeded"
EXPECTED_RUNBOOK_REASON_TAXONOMY_VERSION="kamn.runtime.local-full-stack-harness-runbook-reason-taxonomy.v1"
EXPECTED_RUNBOOK_REASON_CODES_CSV="local_full_stack_harness_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch"
RUNBOOK_TAXONOMY_REASON_CODE="local_full_stack_harness_taxonomy_mapping_drift_detected"
RUNBOOK_MARKER_PARITY_REASON_CODE="runbook_marker_parity_mismatch"

if [ ! -x "$POLICY_SCRIPT" ]; then
  echo "expected local full-stack integration policy script to be executable" >&2
  exit 1
fi
if [ ! -f "$RUNBOOK_DOC" ]; then
  echo "expected local full-stack integration runbook doc to exist" >&2
  exit 1
fi

bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$TMP_REPORT" <<'JSON'
{
  "schema_version": "kamn.runtime.local-full-stack-integration-live-report.v1",
  "status": "pass",
  "final_decision": "GO",
  "lane_mode": "dry-run",
  "ci_fast_gate": "PASS",
  "ci_fast_gate_eligibility": "eligible",
  "fast_gate_exclusion_status": "verified",
  "fast_gate_exclusion_reason_code": "local_full_stack_integration_run_mode_excluded_from_fast_gate",
  "scenario_matrix_status": "verified",
  "full_runtime_status": "verified",
  "native_libp2p_convergence_status": "planned",
  "libp2p_runtime_transport_mode": "libp2p_process_isolated_convergence",
  "libp2p_native_provider_marker": "p2p-live-libp2p-provider:native",
  "libp2p_fallback_marker_blocklist": [
    "p2p-in-memory-transport-fallback",
    "p2p-live-libp2p-provider:contract-only"
  ],
  "libp2p_fallback_markers_detected": [],
  "libp2p_provider_marker_contract_status": "verified",
  "libp2p_process_isolation_status": "planned",
  "libp2p_two_node_process_isolated_status": "planned",
  "libp2p_three_node_process_isolated_status": "planned",
  "libp2p_convergence_report_schema_version": "kamn.runtime.libp2p-convergence-process-isolated-live-report.v1",
  "libp2p_convergence_policy_schema_version": "kamn.runtime.libp2p-convergence-process-isolated-live-policy-report.v1",
  "evidence_bundle_status": "verified",
  "transport_convergence_status": "planned",
  "signer_provenance_status": "planned",
  "runtime_commit_submission_status": "planned",
  "runtime_commit_finality_status": "planned",
  "runtime_provider_contract_status": "planned",
  "runtime_provider_client_contract": "KolmeRuntimeCommitLiveProvider",
  "runtime_signing_profile": "kolme-fork-secp256k1-v1",
  "runtime_signer_attestation_schema_version": "kamn.kolme.runtime-signer-attestation.v1",
  "kolme_local_prerequisite_status": "planned",
  "kolme_local_only_enforced_status": "planned",
  "kolme_integration_mode_status": "planned",
  "kolme_integration_policy_status": "planned",
  "kolme_checkout_path": "/tmp/kolme_fork",
  "kolme_expected_remote_url": "https://github.com/njfio/kolme_fork.git",
  "kolme_expected_ref": "refs/heads/main",
  "kolme_base_url": "http://127.0.0.1:3000",
  "kolme_fork_chain_version": "v0.15.2",
  "kolme_integration_report_schema_version": "kamn.kolme.local-kamn-live-runtime-integration-summary.v1",
  "kolme_integration_policy_schema_version": "kamn.kolme.local-kamn-live-runtime-integration-policy-report.v1",
  "combined_reason_taxonomy_version": "kamn.runtime.local-full-stack-integration-reason-taxonomy.v1",
  "runtime_phase_parity_reason_taxonomy_version": "kamn.runtime.phase-module-extraction-parity-reason-taxonomy.v1",
  "runtime_phase_parity_reason_codes_csv": "runtime_phase_module_parity_drift_detected,runtime_extraction_evidence_output_unstable,ci_local_runtime_phase_parity_budget_boundary_exceeded",
  "runtime_phase_parity_evidence_outputs_csv": "runtime_phase_module_parity_status,runtime_extraction_evidence_output_status,ci_local_runtime_phase_parity_budget_boundary_status",
  "runtime_phase_module_parity_status": "verified",
  "runtime_extraction_evidence_output_status": "verified",
  "ci_local_runtime_phase_parity_budget_boundary_status": "verified",
  "runtime_module_boundary_parity_reason_taxonomy_version": "kamn.runtime.module-boundary-parity-reason-taxonomy.v1",
  "runtime_module_boundary_parity_reason_codes_csv": "runtime_orchestration_dispatch_boundary_drift_detected,runtime_daemon_phase_boundary_drift_detected,runtime_kolme_live_boundary_drift_detected,ci_local_runtime_module_boundary_budget_boundary_exceeded",
  "runtime_module_boundary_evidence_outputs_csv": "runtime_module_boundary_parity_status,runtime_module_boundary_evidence_status,ci_local_runtime_module_boundary_budget_boundary_status",
  "runtime_orchestration_dispatch_boundary_status": "verified",
  "runtime_daemon_phase_boundary_status": "verified",
  "runtime_kolme_live_boundary_status": "verified",
  "runtime_module_boundary_parity_status": "verified",
  "runtime_module_boundary_evidence_status": "verified",
  "ci_local_runtime_module_boundary_budget_boundary_status": "verified",
  "runtime_module_boundary_reason_codes_value": "none",
  "combined_transport_reason_codes": ["fork_choice_stale_block_height"],
  "combined_kolme_runtime_reason_code": "not_run",
  "kolme_runtime_commit_failure_taxonomy_version": "v1",
  "kolme_runtime_commit_failure_taxonomy": "not_run",
  "kolme_fixture_profile": "real-node-non-synthetic-v1",
  "kolme_fixture_profile_version": "v1",
  "kolme_fixture_profile_status": "planned",
  "run_mode_command_status": "dry_run_no_commands_executed",
  "run_mode_command_count": 0,
  "reason_code": "dry_run_no_commands_executed",
  "local_heavy_runtime_budget_status": "verified",
  "elapsed_seconds": 1,
  "max_seconds": 120,
  "command_max_seconds": 60,
  "artifact_paths": {}
}
JSON

policy_output="$(
  bash "$POLICY_SCRIPT" \
    --report-file "$TMP_REPORT" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --runbook-file "$RUNBOOK_DOC" \
    --output-json "$TMP_POLICY"
)"
if ! printf '%s\n' "$policy_output" | grep -q '^status=ok$'; then
  echo "expected local full-stack integration policy status=ok marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^final_decision=GO$'; then
  echo "expected local full-stack integration policy final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^local_full_stack_integration_policy_status=verified$'; then
  echo "expected local full-stack integration policy status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^reason_taxonomy_version=kamn.runtime.phase-module-extraction-parity-reason-taxonomy.v1$'; then
  echo "expected local full-stack integration policy reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^reason_codes_csv=runtime_phase_module_parity_drift_detected,runtime_extraction_evidence_output_unstable,ci_local_runtime_phase_parity_budget_boundary_exceeded$'; then
  echo "expected local full-stack integration policy reason codes marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^reason_codes_value=none$'; then
  echo "expected local full-stack integration policy normalized reason_codes_value marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^runtime_module_boundary_reason_taxonomy_version=kamn.runtime.module-boundary-parity-reason-taxonomy.v1$'; then
  echo "expected local full-stack integration policy runtime module boundary reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^runtime_module_boundary_reason_codes_csv=runtime_orchestration_dispatch_boundary_drift_detected,runtime_daemon_phase_boundary_drift_detected,runtime_kolme_live_boundary_drift_detected,ci_local_runtime_module_boundary_budget_boundary_exceeded$'; then
  echo "expected local full-stack integration policy runtime module boundary reason codes marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^runtime_module_boundary_reason_codes_value=none$'; then
  echo "expected local full-stack integration policy runtime module boundary normalized reason marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^runtime_phase_parity_evidence_outputs_csv=runtime_phase_module_parity_status,runtime_extraction_evidence_output_status,ci_local_runtime_phase_parity_budget_boundary_status$'; then
  echo "expected local full-stack integration policy parity evidence output normalization marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^runtime_module_boundary_evidence_outputs_csv=runtime_module_boundary_parity_status,runtime_module_boundary_evidence_status,ci_local_runtime_module_boundary_budget_boundary_status$'; then
  echo "expected local full-stack integration policy runtime module boundary evidence output normalization marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^local_full_stack_harness_runbook_marker_parity_status=verified$'; then
  echo "expected local full-stack integration runbook marker parity status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q "^local_full_stack_harness_runbook_reason_taxonomy_version=${EXPECTED_RUNBOOK_REASON_TAXONOMY_VERSION}$"; then
  echo "expected local full-stack integration runbook reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q "^local_full_stack_harness_runbook_reason_codes_csv=${EXPECTED_RUNBOOK_REASON_CODES_CSV}$"; then
  echo "expected local full-stack integration runbook reason codes marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^local_full_stack_harness_runbook_reason_code=none$'; then
  echo "expected local full-stack integration runbook normalized reason marker" >&2
  exit 1
fi

python3 - "$TMP_POLICY" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.local-full-stack-integration-live-policy-report.v1":
    raise SystemExit("unexpected local full-stack integration policy schema")
if payload.get("status") != "ok":
    raise SystemExit("expected local full-stack integration policy status=ok")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected local full-stack integration policy final_decision=GO")
if payload.get("local_full_stack_integration_policy_status") != "verified":
    raise SystemExit("expected local_full_stack_integration_policy_status=verified")
if payload.get("reason_taxonomy_version") != "kamn.runtime.phase-module-extraction-parity-reason-taxonomy.v1":
    raise SystemExit("expected deterministic runtime phase extraction parity reason taxonomy marker")
if payload.get("reason_codes_csv") != "runtime_phase_module_parity_drift_detected,runtime_extraction_evidence_output_unstable,ci_local_runtime_phase_parity_budget_boundary_exceeded":
    raise SystemExit("expected deterministic runtime phase extraction parity reason codes marker")
if payload.get("reason_codes_value") != "none":
    raise SystemExit("expected deterministic reason_codes_value=none marker")
if payload.get("runtime_phase_parity_evidence_outputs_csv") != "runtime_phase_module_parity_status,runtime_extraction_evidence_output_status,ci_local_runtime_phase_parity_budget_boundary_status":
    raise SystemExit("expected deterministic runtime phase parity evidence output normalization marker")
if payload.get("runtime_module_boundary_reason_taxonomy_version") != "kamn.runtime.module-boundary-parity-reason-taxonomy.v1":
    raise SystemExit("expected deterministic runtime module boundary reason taxonomy marker")
if payload.get("runtime_module_boundary_reason_codes_csv") != "runtime_orchestration_dispatch_boundary_drift_detected,runtime_daemon_phase_boundary_drift_detected,runtime_kolme_live_boundary_drift_detected,ci_local_runtime_module_boundary_budget_boundary_exceeded":
    raise SystemExit("expected deterministic runtime module boundary reason codes marker")
if payload.get("runtime_module_boundary_reason_codes_value") != "none":
    raise SystemExit("expected deterministic runtime module boundary reason_codes_value=none marker")
if payload.get("runtime_module_boundary_evidence_outputs_csv") != "runtime_module_boundary_parity_status,runtime_module_boundary_evidence_status,ci_local_runtime_module_boundary_budget_boundary_status":
    raise SystemExit("expected deterministic runtime module boundary evidence output normalization marker")
if payload.get("local_full_stack_harness_runbook_marker_parity_status") != "verified":
    raise SystemExit("expected deterministic runbook marker parity status marker")
if payload.get("local_full_stack_harness_runbook_reason_taxonomy_version") != "kamn.runtime.local-full-stack-harness-runbook-reason-taxonomy.v1":
    raise SystemExit("expected deterministic runbook reason taxonomy marker")
if payload.get("local_full_stack_harness_runbook_reason_codes_csv") != "local_full_stack_harness_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch":
    raise SystemExit("expected deterministic runbook reason codes marker")
if payload.get("local_full_stack_harness_runbook_reason_code") != "none":
    raise SystemExit("expected deterministic runbook reason marker normalization")
PY

cp "$TMP_REPORT" "$TMP_TAMPERED"
python3 - "$TMP_TAMPERED" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["runtime_commit_finality_status"] = "missing"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_output="$(
  bash "$POLICY_SCRIPT" \
    --report-file "$TMP_TAMPERED" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_POLICY" 2>&1
)"
tampered_code=$?
set -e
if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered local full-stack integration report to fail policy check" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q 'local_full_stack_integration_policy_runtime_commit_finality_status_mismatch'; then
  echo "expected deterministic reason marker for tampered runtime commit finality status" >&2
  exit 1
fi

cp "$TMP_REPORT" "$TMP_TAMPERED"
python3 - "$TMP_TAMPERED" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["kolme_local_prerequisite_status"] = "missing"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_kolme_marker_output="$(
  bash "$POLICY_SCRIPT" \
    --report-file "$TMP_TAMPERED" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_POLICY" 2>&1
)"
tampered_kolme_marker_code=$?
set -e
if [ "$tampered_kolme_marker_code" -eq 0 ]; then
  echo "expected tampered Kolme local prerequisite marker to fail policy check" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_kolme_marker_output" | grep -q 'local_full_stack_integration_policy_kolme_local_prerequisite_status_mismatch'; then
  echo "expected deterministic reason marker for tampered Kolme local prerequisite status" >&2
  exit 1
fi

cp "$TMP_REPORT" "$TMP_TAMPERED"
python3 - "$TMP_TAMPERED" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["libp2p_three_node_process_isolated_status"] = "missing"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_three_node_marker_output="$(
  bash "$POLICY_SCRIPT" \
    --report-file "$TMP_TAMPERED" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_POLICY" 2>&1
)"
tampered_three_node_marker_code=$?
set -e
if [ "$tampered_three_node_marker_code" -eq 0 ]; then
  echo "expected tampered three-node process-isolated marker to fail policy check" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_three_node_marker_output" | grep -q 'local_full_stack_integration_policy_libp2p_three_node_process_isolated_status_mismatch'; then
  echo "expected deterministic reason marker for tampered three-node process-isolated status" >&2
  exit 1
fi

cp "$TMP_REPORT" "$TMP_TAMPERED"
python3 - "$TMP_TAMPERED" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["elapsed_seconds"] = 130
payload["max_seconds"] = 120
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_runtime_budget_output="$(
  bash "$POLICY_SCRIPT" \
    --report-file "$TMP_TAMPERED" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_POLICY" 2>&1
)"
tampered_runtime_budget_code=$?
set -e
if [ "$tampered_runtime_budget_code" -eq 0 ]; then
  echo "expected runtime budget exceedance to fail local full-stack integration policy check" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_runtime_budget_output" | grep -q 'local_full_stack_integration_policy_runtime_budget_exceeded'; then
  echo "expected deterministic reason marker for runtime budget exceedance" >&2
  exit 1
fi

cp "$TMP_REPORT" "$TMP_TAMPERED"
python3 - "$TMP_TAMPERED" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["combined_reason_taxonomy_version"] = "v0"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_reason_taxonomy_output="$(
  bash "$POLICY_SCRIPT" \
    --report-file "$TMP_TAMPERED" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_POLICY" 2>&1
)"
tampered_reason_taxonomy_code=$?
set -e
if [ "$tampered_reason_taxonomy_code" -eq 0 ]; then
  echo "expected tampered combined reason taxonomy version to fail policy check" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_reason_taxonomy_output" | grep -q 'local_full_stack_integration_policy_reason_taxonomy_version_mismatch'; then
  echo "expected deterministic reason marker for tampered combined reason taxonomy version" >&2
  exit 1
fi

cp "$TMP_REPORT" "$TMP_TAMPERED"
python3 - "$TMP_TAMPERED" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["runtime_phase_parity_reason_taxonomy_version"] = "v0"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_runtime_phase_reason_taxonomy_output="$(
  bash "$POLICY_SCRIPT" \
    --report-file "$TMP_TAMPERED" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_POLICY" 2>&1
)"
tampered_runtime_phase_reason_taxonomy_code=$?
set -e
if [ "$tampered_runtime_phase_reason_taxonomy_code" -eq 0 ]; then
  echo "expected runtime phase parity reason taxonomy drift to fail local full-stack integration policy check" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_runtime_phase_reason_taxonomy_output" | grep -q 'local_full_stack_integration_policy_runtime_phase_parity_reason_taxonomy_version_mismatch'; then
  echo "expected deterministic reason marker for runtime phase parity reason taxonomy drift" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_runtime_phase_reason_taxonomy_output" | grep -q '^reason_codes_value=runtime_extraction_evidence_output_unstable$'; then
  echo "expected normalized reason mapping for runtime phase parity reason taxonomy drift" >&2
  exit 1
fi

cp "$TMP_REPORT" "$TMP_TAMPERED"
python3 - "$TMP_TAMPERED" "$EXPECTED_RUNTIME_PHASE_REASON_CODES_CSV" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
expected = sys.argv[2]
payload = json.loads(path.read_text(encoding="utf-8"))
payload["runtime_phase_parity_reason_codes_csv"] = expected.split(",")[0]
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_runtime_phase_reason_codes_output="$(
  bash "$POLICY_SCRIPT" \
    --report-file "$TMP_TAMPERED" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_POLICY" 2>&1
)"
tampered_runtime_phase_reason_codes_code=$?
set -e
if [ "$tampered_runtime_phase_reason_codes_code" -eq 0 ]; then
  echo "expected runtime phase parity reason code drift to fail local full-stack integration policy check" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_runtime_phase_reason_codes_output" | grep -q 'local_full_stack_integration_policy_runtime_phase_parity_reason_codes_csv_mismatch'; then
  echo "expected deterministic reason marker for runtime phase parity reason code drift" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_runtime_phase_reason_codes_output" | grep -q '^reason_codes_value=runtime_extraction_evidence_output_unstable$'; then
  echo "expected normalized reason mapping for runtime phase parity reason code drift" >&2
  exit 1
fi

cp "$TMP_REPORT" "$TMP_TAMPERED"
python3 - "$TMP_TAMPERED" "$EXPECTED_RUNTIME_MODULE_BOUNDARY_REASON_CODES_CSV" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
expected = sys.argv[2]
payload = json.loads(path.read_text(encoding="utf-8"))
payload["runtime_module_boundary_parity_reason_codes_csv"] = expected.split(",")[0]
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_runtime_module_boundary_reason_codes_output="$(
  bash "$POLICY_SCRIPT" \
    --report-file "$TMP_TAMPERED" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_POLICY" 2>&1
)"
tampered_runtime_module_boundary_reason_codes_code=$?
set -e
if [ "$tampered_runtime_module_boundary_reason_codes_code" -eq 0 ]; then
  echo "expected runtime module boundary reason code drift to fail local full-stack integration policy check" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_runtime_module_boundary_reason_codes_output" | grep -q 'local_full_stack_integration_policy_runtime_module_boundary_parity_reason_codes_csv_mismatch'; then
  echo "expected deterministic reason marker for runtime module boundary reason code drift" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_runtime_module_boundary_reason_codes_output" | grep -q '^runtime_module_boundary_reason_codes_value=runtime_orchestration_dispatch_boundary_drift_detected$'; then
  echo "expected normalized runtime module boundary reason mapping for reason code drift" >&2
  exit 1
fi

cp "$TMP_REPORT" "$TMP_TAMPERED"
python3 - "$TMP_TAMPERED" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["libp2p_fallback_markers_detected"] = ["p2p-in-memory-transport-fallback"]
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_fallback_marker_output="$(
  bash "$POLICY_SCRIPT" \
    --report-file "$TMP_TAMPERED" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_POLICY" 2>&1
)"
tampered_fallback_marker_code=$?
set -e
if [ "$tampered_fallback_marker_code" -eq 0 ]; then
  echo "expected fallback marker drift to fail local full-stack integration policy check" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_fallback_marker_output" | grep -q 'local_full_stack_integration_policy_libp2p_fallback_markers_detected'; then
  echo "expected deterministic reason marker for libp2p fallback marker drift" >&2
  exit 1
fi

cp "$TMP_REPORT" "$TMP_TAMPERED"
python3 - "$TMP_TAMPERED" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["runtime_phase_module_parity_status"] = "drifted"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_phase_parity_output="$(
  bash "$POLICY_SCRIPT" \
    --report-file "$TMP_TAMPERED" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_POLICY" 2>&1
)"
tampered_phase_parity_code=$?
set -e
if [ "$tampered_phase_parity_code" -eq 0 ]; then
  echo "expected runtime phase module parity drift to fail local full-stack integration policy check" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_phase_parity_output" | grep -q 'runtime_phase_module_parity_drift_detected'; then
  echo "expected deterministic runtime phase module parity drift reason marker" >&2
  exit 1
fi

cp "$TMP_REPORT" "$TMP_TAMPERED"
python3 - "$TMP_TAMPERED" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["runtime_orchestration_dispatch_boundary_status"] = "drifted"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_module_boundary_output="$(
  bash "$POLICY_SCRIPT" \
    --report-file "$TMP_TAMPERED" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_POLICY" 2>&1
)"
tampered_module_boundary_code=$?
set -e
if [ "$tampered_module_boundary_code" -eq 0 ]; then
  echo "expected runtime module boundary drift to fail local full-stack integration policy check" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_module_boundary_output" | grep -q 'runtime_orchestration_dispatch_boundary_drift_detected'; then
  echo "expected deterministic runtime module boundary drift reason marker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_module_boundary_output" | grep -q '^runtime_module_boundary_reason_codes_value=runtime_orchestration_dispatch_boundary_drift_detected$'; then
  echo "expected normalized runtime module boundary reason mapping for dispatch boundary drift" >&2
  exit 1
fi

cp "$TMP_REPORT" "$TMP_TAMPERED"
python3 - "$TMP_TAMPERED" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["runtime_extraction_evidence_output_status"] = "drifted"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_extraction_evidence_output="$(
  bash "$POLICY_SCRIPT" \
    --report-file "$TMP_TAMPERED" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_POLICY" 2>&1
)"
tampered_extraction_evidence_code=$?
set -e
if [ "$tampered_extraction_evidence_code" -eq 0 ]; then
  echo "expected runtime extraction evidence output drift to fail local full-stack integration policy check" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_extraction_evidence_output" | grep -q 'runtime_extraction_evidence_output_unstable'; then
  echo "expected deterministic runtime extraction evidence output drift reason marker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_extraction_evidence_output" | grep -q '^reason_codes_value=runtime_extraction_evidence_output_unstable$'; then
  echo "expected normalized reason_codes_value mapping for runtime extraction evidence output drift" >&2
  exit 1
fi

cp "$TMP_REPORT" "$TMP_TAMPERED"
python3 - "$TMP_TAMPERED" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["max_seconds"] = 241
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_phase_budget_output="$(
  bash "$POLICY_SCRIPT" \
    --report-file "$TMP_TAMPERED" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_POLICY" 2>&1
)"
tampered_phase_budget_code=$?
set -e
if [ "$tampered_phase_budget_code" -eq 0 ]; then
  echo "expected runtime phase parity budget overrun to fail local full-stack integration policy check" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_phase_budget_output" | grep -q 'ci_local_runtime_phase_parity_budget_boundary_exceeded'; then
  echo "expected deterministic runtime phase parity budget boundary reason marker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_phase_budget_output" | grep -q 'ci_local_runtime_module_boundary_budget_boundary_exceeded'; then
  echo "expected deterministic runtime module boundary budget boundary reason marker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_phase_budget_output" | grep -q '^runtime_module_boundary_reason_codes_value=ci_local_runtime_module_boundary_budget_boundary_exceeded$'; then
  echo "expected normalized runtime module boundary budget reason mapping marker" >&2
  exit 1
fi

cp "$RUNBOOK_DOC" "$TMP_RUNBOOK_TAXONOMY_DRIFT"
python3 - "$TMP_RUNBOOK_TAXONOMY_DRIFT" <<'PY'
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = path.read_text(encoding="utf-8")
payload = payload.replace(
    "combined_reason_taxonomy_version=kamn.runtime.local-full-stack-integration-reason-taxonomy.v1",
    "combined_reason_taxonomy_version=v0",
)
path.write_text(payload, encoding="utf-8")
PY

set +e
runbook_taxonomy_drift_output="$(
  bash "$POLICY_SCRIPT" \
    --report-file "$TMP_REPORT" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --runbook-file "$TMP_RUNBOOK_TAXONOMY_DRIFT" \
    --output-json "$TMP_POLICY" 2>&1
)"
runbook_taxonomy_drift_code=$?
set -e
if [ "$runbook_taxonomy_drift_code" -eq 0 ]; then
  echo "expected runbook taxonomy drift to fail local full-stack integration policy check" >&2
  exit 1
fi
if ! printf '%s\n' "$runbook_taxonomy_drift_output" | grep -q "$RUNBOOK_TAXONOMY_REASON_CODE"; then
  echo "expected deterministic runbook taxonomy drift reason marker" >&2
  exit 1
fi

cp "$RUNBOOK_DOC" "$TMP_RUNBOOK_PARITY_DRIFT"
python3 - "$TMP_RUNBOOK_PARITY_DRIFT" <<'PY'
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = path.read_text(encoding="utf-8")
payload = payload.replace(
    "local_full_stack_harness_runbook_reason_codes_csv=local_full_stack_harness_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch",
    "",
    1,
)
path.write_text(payload, encoding="utf-8")
PY

set +e
runbook_marker_parity_output="$(
  bash "$POLICY_SCRIPT" \
    --report-file "$TMP_REPORT" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --runbook-file "$TMP_RUNBOOK_PARITY_DRIFT" \
    --output-json "$TMP_POLICY" 2>&1
)"
runbook_marker_parity_code=$?
set -e
if [ "$runbook_marker_parity_code" -eq 0 ]; then
  echo "expected runbook marker parity drift to fail local full-stack integration policy check" >&2
  exit 1
fi
if ! printf '%s\n' "$runbook_marker_parity_output" | grep -q "$RUNBOOK_MARKER_PARITY_REASON_CODE"; then
  echo "expected deterministic runbook marker parity reason marker" >&2
  exit 1
fi

echo "local full-stack integration policy checker tests passed."
