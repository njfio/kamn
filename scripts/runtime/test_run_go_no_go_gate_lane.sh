#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LANE_SCRIPT="$ROOT_DIR/scripts/runtime/run_go_no_go_gate_lane.sh"
LANE_IMPL_SCRIPT="$ROOT_DIR/scripts/runtime/run_go_no_go_gate_lane_impl.sh"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/runtime_go_no_go_gate_lane.json"
RELEASE_MANIFEST_FILE="$ROOT_DIR/scripts/runtime/release_evidence_manifest.json"
EXEC_DISPATCHER="$ROOT_DIR/scripts/lib/exec_dispatch.sh"
EXEC_REGISTRY="$ROOT_DIR/scripts/lib/exec_registry.json"
TMP_DIR="$(mktemp -d)"
TMP_REPORT="$TMP_DIR/go-no-go-gate-report.json"
TMP_FAULT_REPORT="$TMP_DIR/go-no-go-gate-fault-report.json"
TMP_FALLBACK_MARKER_FAULT_REPORT="$TMP_DIR/go-no-go-gate-fallback-marker-fault-report.json"
TMP_READINESS_MARKER_FAULT_REPORT="$TMP_DIR/go-no-go-gate-readiness-marker-fault-report.json"
TMP_WARN_REPORT="$TMP_DIR/go-no-go-gate-warn-report.json"
TMP_RUN_REPORT="$TMP_DIR/go-no-go-gate-run-mode-report.json"
TMP_WAIVER_REPORT="$TMP_DIR/go-no-go-gate-waiver-report.json"
TMP_MANIFEST_FAIL_REPORT="$TMP_DIR/go-no-go-gate-manifest-fail-report.json"
TMP_CONVERGENCE_MISSING_REPORT="$TMP_DIR/go-no-go-gate-missing-convergence-report.json"
TMP_CONVERGENCE_TAMPER_REPORT="$TMP_DIR/go-no-go-gate-tampered-convergence-marker-report.json"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$LANE_SCRIPT" ]; then
  echo "expected go/no-go gate lane script to be executable" >&2
  exit 1
fi
if [ ! -x "$LANE_IMPL_SCRIPT" ]; then
  echo "expected go/no-go gate lane implementation script to be executable" >&2
  exit 1
fi
if [ ! -x "$DISPATCHER" ]; then
  echo "expected shared non-Kolme dispatcher to be executable" >&2
  exit 1
fi
if [ ! -x "$EXEC_DISPATCHER" ]; then
  echo "expected shared exec dispatcher to be executable" >&2
  exit 1
fi
if [ ! -f "$EXEC_REGISTRY" ]; then
  echo "expected exec wrapper registry to exist" >&2
  exit 1
fi
if [ ! -L "$LANE_SCRIPT" ]; then
  echo "expected go/no-go gate lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi
if [ "$(readlink "$LANE_SCRIPT")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected go/no-go gate lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$LANE_SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected go/no-go gate lane wrapper to resolve runtime manifest via dispatcher" >&2
  exit 1
fi

if ! grep -q 'run_go_no_go_gate_lane_impl.sh' "$MANIFEST_FILE"; then
  echo "expected go/no-go gate lane manifest to dispatch implementation module" >&2
  exit 1
fi
if [ ! -L "$LANE_IMPL_SCRIPT" ]; then
  echo "expected go/no-go gate lane implementation wrapper to be a symlink" >&2
  exit 1
fi
if [ "$(readlink -f "$LANE_IMPL_SCRIPT")" != "$(readlink -f "$EXEC_DISPATCHER")" ]; then
  echo "expected go/no-go gate lane implementation wrapper to resolve to shared exec dispatcher" >&2
  exit 1
fi

python3 - "$EXEC_REGISTRY" <<'PY'
import json
import sys
from pathlib import Path

registry = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
entry = registry.get("entries", {}).get("scripts/runtime/run_go_no_go_gate_lane_impl.sh")
if not isinstance(entry, dict):
    raise SystemExit("expected registry entry for go/no-go gate lane implementation wrapper")
if entry.get("interpreter") != "python3":
    raise SystemExit("expected python3 interpreter for go/no-go gate lane implementation wrapper")
if entry.get("target") != "scripts/runtime/go_no_go_gate_lane_contract.py":
    raise SystemExit("expected go/no-go gate lane implementation wrapper target in exec registry")
if entry.get("args_prefix") != []:
    raise SystemExit("expected empty args_prefix for go/no-go gate lane implementation wrapper")
if entry.get("passthrough") is not True:
    raise SystemExit("expected passthrough=true for go/no-go gate lane implementation wrapper")
PY
if [ ! -f "$RELEASE_MANIFEST_FILE" ]; then
  echo "expected release evidence manifest file for go/no-go gate lane" >&2
  exit 1
fi

python3 - "$ROOT_DIR" <<'PY'
import importlib.util
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
module_path = root / "scripts/runtime/go_no_go_gate_lane_contract.py"
spec = importlib.util.spec_from_file_location("go_no_go_gate_lane_contract", module_path)
module = importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(module)

artifact_inventory = [
    {"artifact_id": "go_no_go_evidence", "status": "dry_run_pending"},
]

# Unit: missing readiness marker fails closed.
policy_outcome, final_decision, status, reason_codes = module._evaluate_go_no_go_policy(
    artifact_inventory,
    {"go_no_go_evidence_status": ""},
    [],
    "dry-run",
    module.NATIVE_LIBP2P_PROVIDER_MARKER,
    list(module.LIBP2P_FALLBACK_MARKER_BLOCKLIST),
    [],
    "verified",
)
if (policy_outcome, final_decision, status) != ("FAIL", "NO-GO", "fail"):
    raise SystemExit("expected missing readiness marker unit check to fail closed")
if "gate_required_artifact_status_mismatch:go_no_go_evidence" not in reason_codes:
    raise SystemExit("expected deterministic readiness marker mismatch reason in unit check")

# Unit: runtime budget overflow fails closed.
policy_outcome, final_decision, status, reason_codes = module._evaluate_go_no_go_policy(
    artifact_inventory,
    {"go_no_go_evidence_status": "dry_run_pending"},
    [module.RUNTIME_BUDGET_EXCEEDED_REASON],
    "dry-run",
    module.NATIVE_LIBP2P_PROVIDER_MARKER,
    list(module.LIBP2P_FALLBACK_MARKER_BLOCKLIST),
    [],
    "verified",
)
if (policy_outcome, final_decision, status) != ("FAIL", "NO-GO", "fail"):
    raise SystemExit("expected runtime budget unit check to fail closed")
if reason_codes != [module.RUNTIME_BUDGET_EXCEEDED_REASON]:
    raise SystemExit("expected deterministic runtime budget reason code in unit check")
PY

lane_output="$(
  bash "$LANE_SCRIPT" \
    --mode dry-run \
    --max-seconds 120 \
    --output-json "$TMP_REPORT"
)"
if ! printf '%s\n' "$lane_output" | grep -q '^status=pass$'; then
  echo "expected go/no-go gate lane pass status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^policy_outcome=PASS$'; then
  echo "expected go/no-go gate lane PASS policy-outcome marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^final_decision=GO$'; then
  echo "expected go/no-go gate lane GO decision marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^go_no_go_evidence_status=dry_run_pending$'; then
  echo "expected go/no-go gate lane dry-run evidence status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^rollback_readiness_status=dry_run_pending$'; then
  echo "expected go/no-go gate lane dry-run rollback status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^dr_readiness_status=dry_run_pending$'; then
  echo "expected go/no-go gate lane dry-run dr status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^local_full_stack_integration_status=dry_run_pending$'; then
  echo "expected go/no-go gate lane dry-run local full-stack integration status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^local_full_runtime_convergence_status=dry_run_pending$'; then
  echo "expected go/no-go gate lane dry-run local full-runtime convergence status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^transport_fault_matrix_status=dry_run_pending$'; then
  echo "expected go/no-go gate lane dry-run transport fault-matrix status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^policy_evaluator_status=verified$'; then
  echo "expected go/no-go gate lane policy evaluator status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^reason_taxonomy_version=kamn.runtime.go-no-go-gate-reason-taxonomy.v1$'; then
  echo "expected go/no-go gate lane reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^promotion_evidence_convergence_status=verified$'; then
  echo "expected go/no-go gate lane promotion evidence convergence status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^promotion_evidence_reason_taxonomy_version=kamn.runtime.go-no-go-gate-evidence-convergence-reason-taxonomy.v1$'; then
  echo "expected go/no-go gate lane promotion evidence reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^promotion_evidence_reason_codes_csv=promotion_evidence_link_missing,promotion_evidence_payload_tamper_detected,promotion_decision_reason_mapping_mismatch$'; then
  echo "expected go/no-go gate lane promotion evidence reason codes marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^promotion_evidence_reason_code=none$'; then
  echo "expected go/no-go gate lane promotion evidence reason marker normalization" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^promotion_decision_reason_mapping_status=verified$'; then
  echo "expected go/no-go gate lane promotion decision mapping status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^promotion_decision_reason_taxonomy_version=kamn.runtime.go-no-go-gate-promotion-decision-reason-taxonomy.v1$'; then
  echo "expected go/no-go gate lane promotion decision reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^promotion_decision_reason_codes_csv=release_manifest_missing_required_artifact,release_manifest_success_marker_mismatch,gate_required_artifact_status_mismatch,gate_decision_fault_injection_triggered,runtime_budget_exceeded,gate_policy_unknown_reason_code,gate_policy_native_libp2p_provider_marker_mismatch,gate_policy_libp2p_fallback_marker_blocklist_mismatch,gate_policy_libp2p_fallback_markers_detected,gate_policy_native_libp2p_provider_marker_contract_status_mismatch$'; then
  echo "expected go/no-go gate lane promotion decision reason codes marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^promotion_decision_reason_code=none$'; then
  echo "expected go/no-go gate lane promotion decision reason marker normalization" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^combined_reason_taxonomy_version=kamn.runtime.local-full-stack-integration-reason-taxonomy.v1$'; then
  echo "expected go/no-go gate lane combined reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^combined_transport_reason_codes=fork_choice_stale_block_height$'; then
  echo "expected go/no-go gate lane combined transport reason marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^combined_kolme_runtime_reason_code=not_run$'; then
  echo "expected go/no-go gate lane combined Kolme reason marker in dry-run mode" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^kolme_runtime_commit_failure_taxonomy_version=v1$'; then
  echo "expected go/no-go gate lane Kolme failure taxonomy version marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^kolme_runtime_commit_failure_taxonomy=not_run$'; then
  echo "expected go/no-go gate lane Kolme failure taxonomy marker in dry-run mode" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^kolme_fixture_profile=real-node-non-synthetic-v1$'; then
  echo "expected go/no-go gate lane Kolme fixture profile marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^kolme_fixture_profile_version=v1$'; then
  echo "expected go/no-go gate lane Kolme fixture profile version marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^kolme_fixture_profile_status=planned$'; then
  echo "expected go/no-go gate lane Kolme fixture profile status marker in dry-run mode" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^combined_lane_marker_contract_status=verified$'; then
  echo "expected go/no-go gate lane combined marker contract status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^native_libp2p_provider_marker=p2p-live-libp2p-provider:native$'; then
  echo "expected go/no-go gate lane native libp2p provider marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^libp2p_fallback_marker_blocklist=p2p-in-memory-transport-fallback,p2p-live-libp2p-provider:contract-only$'; then
  echo "expected go/no-go gate lane fallback marker blocklist" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^libp2p_fallback_markers_detected=none$'; then
  echo "expected go/no-go gate lane empty fallback marker detection marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^native_libp2p_provider_marker_contract_status=verified$'; then
  echo "expected go/no-go gate lane provider marker contract status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^lane_mode=dry-run$'; then
  echo "expected go/no-go gate lane dry-run mode marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^run_mode_command_status=dry_run_no_commands_executed$'; then
  echo "expected go/no-go gate lane dry-run command status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^run_mode_command_count=0$'; then
  echo "expected go/no-go gate lane dry-run command count marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^ci_fast_gate_eligible=true$'; then
  echo "expected go/no-go gate lane dry-run fast-gate eligibility marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^ci_fast_gate_scope=ci-fast-gate$'; then
  echo "expected go/no-go gate lane dry-run fast-gate scope marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^fast_gate_exclusion_status=verified$'; then
  echo "expected go/no-go gate lane fast-gate exclusion status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^fast_gate_exclusion_reason_code=go_no_go_gate_run_mode_excluded_from_fast_gate$'; then
  echo "expected go/no-go gate lane fast-gate exclusion reason marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^waiver_status=none$'; then
  echo "expected go/no-go gate lane baseline waiver status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^waived_reason_codes=none$'; then
  echo "expected go/no-go gate lane baseline waived reason marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.go-no-go-gate-report.v1":
    raise SystemExit("unexpected go/no-go gate report schema")
if payload.get("status") != "pass":
    raise SystemExit("expected go/no-go gate report status=pass")
if payload.get("policy_outcome") != "PASS":
    raise SystemExit("expected go/no-go gate report policy_outcome=PASS")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected go/no-go gate report final_decision=GO")
if payload.get("fault_profile") != "none":
    raise SystemExit("expected go/no-go gate report fault_profile=none")
if payload.get("reason_taxonomy_version") != "kamn.runtime.go-no-go-gate-reason-taxonomy.v1":
    raise SystemExit("expected go/no-go gate report reason taxonomy version marker")
if payload.get("promotion_evidence_convergence_status") != "verified":
    raise SystemExit("expected promotion_evidence_convergence_status=verified")
if payload.get("promotion_evidence_reason_taxonomy_version") != "kamn.runtime.go-no-go-gate-evidence-convergence-reason-taxonomy.v1":
    raise SystemExit("expected promotion_evidence_reason_taxonomy_version marker")
if payload.get("promotion_evidence_reason_codes_csv") != "promotion_evidence_link_missing,promotion_evidence_payload_tamper_detected,promotion_decision_reason_mapping_mismatch":
    raise SystemExit("expected promotion_evidence_reason_codes_csv marker")
if payload.get("promotion_evidence_reason_code") != "none":
    raise SystemExit("expected promotion_evidence_reason_code=none in baseline")
if payload.get("promotion_decision_reason_mapping_status") != "verified":
    raise SystemExit("expected promotion_decision_reason_mapping_status=verified")
if payload.get("promotion_decision_reason_taxonomy_version") != "kamn.runtime.go-no-go-gate-promotion-decision-reason-taxonomy.v1":
    raise SystemExit("expected promotion_decision_reason_taxonomy_version marker")
if payload.get("promotion_decision_reason_codes_csv") != "release_manifest_missing_required_artifact,release_manifest_success_marker_mismatch,gate_required_artifact_status_mismatch,gate_decision_fault_injection_triggered,runtime_budget_exceeded,gate_policy_unknown_reason_code,gate_policy_native_libp2p_provider_marker_mismatch,gate_policy_libp2p_fallback_marker_blocklist_mismatch,gate_policy_libp2p_fallback_markers_detected,gate_policy_native_libp2p_provider_marker_contract_status_mismatch":
    raise SystemExit("expected promotion_decision_reason_codes_csv marker")
if payload.get("promotion_decision_reason_code") != "none":
    raise SystemExit("expected promotion_decision_reason_code=none in baseline")
if payload.get("go_no_go_evidence_status") != "dry_run_pending":
    raise SystemExit("expected go_no_go_evidence_status=dry_run_pending")
if payload.get("rollback_readiness_status") != "dry_run_pending":
    raise SystemExit("expected rollback_readiness_status=dry_run_pending")
if payload.get("dr_readiness_status") != "dry_run_pending":
    raise SystemExit("expected dr_readiness_status=dry_run_pending")
if payload.get("local_full_stack_integration_status") != "dry_run_pending":
    raise SystemExit("expected local_full_stack_integration_status=dry_run_pending")
if payload.get("local_full_runtime_convergence_status") != "dry_run_pending":
    raise SystemExit("expected local_full_runtime_convergence_status=dry_run_pending")
if payload.get("transport_fault_matrix_status") != "dry_run_pending":
    raise SystemExit("expected transport_fault_matrix_status=dry_run_pending")
if payload.get("policy_evaluator_status") != "verified":
    raise SystemExit("expected policy_evaluator_status=verified")
if payload.get("manifest_schema_version") != "kamn.runtime.release-evidence-manifest.v1":
    raise SystemExit("expected manifest_schema_version marker in go/no-go gate report")
if payload.get("manifest_registry_status") != "verified":
    raise SystemExit("expected manifest_registry_status=verified")
if payload.get("combined_reason_taxonomy_version") != "kamn.runtime.local-full-stack-integration-reason-taxonomy.v1":
    raise SystemExit("expected combined_reason_taxonomy_version marker")
if payload.get("combined_transport_reason_codes") != ["fork_choice_stale_block_height"]:
    raise SystemExit("expected combined_transport_reason_codes marker")
if payload.get("combined_kolme_runtime_reason_code") != "not_run":
    raise SystemExit("expected combined_kolme_runtime_reason_code=not_run in dry-run mode")
if payload.get("kolme_runtime_commit_failure_taxonomy_version") != "v1":
    raise SystemExit("expected kolme_runtime_commit_failure_taxonomy_version marker")
if payload.get("kolme_runtime_commit_failure_taxonomy") != "not_run":
    raise SystemExit("expected kolme_runtime_commit_failure_taxonomy=not_run in dry-run mode")
if payload.get("kolme_fixture_profile") != "real-node-non-synthetic-v1":
    raise SystemExit("expected kolme_fixture_profile marker")
if payload.get("kolme_fixture_profile_version") != "v1":
    raise SystemExit("expected kolme_fixture_profile_version marker")
if payload.get("kolme_fixture_profile_status") != "planned":
    raise SystemExit("expected kolme_fixture_profile_status=planned in dry-run mode")
if payload.get("combined_lane_marker_contract_status") != "verified":
    raise SystemExit("expected combined_lane_marker_contract_status=verified")
if payload.get("native_libp2p_provider_marker") != "p2p-live-libp2p-provider:native":
    raise SystemExit("expected native_libp2p_provider_marker in dry-run go/no-go gate report")
if payload.get("libp2p_fallback_marker_blocklist") != [
    "p2p-in-memory-transport-fallback",
    "p2p-live-libp2p-provider:contract-only",
]:
    raise SystemExit("expected libp2p_fallback_marker_blocklist in dry-run go/no-go gate report")
if payload.get("libp2p_fallback_markers_detected") != []:
    raise SystemExit("expected no fallback markers in dry-run go/no-go gate report")
if payload.get("native_libp2p_provider_marker_contract_status") != "verified":
    raise SystemExit(
        "expected native_libp2p_provider_marker_contract_status=verified in dry-run go/no-go gate report"
    )
inventory = payload.get("artifact_inventory")
if not isinstance(inventory, list) or len(inventory) != 6:
    raise SystemExit("expected deterministic artifact inventory list with six required entries")
required_ids = payload.get("required_artifact_ids")
if not isinstance(required_ids, list) or sorted(required_ids) != sorted(
    [
        "go_no_go_evidence",
        "rollback_readiness",
        "dr_readiness",
        "local_full_stack_integration",
        "local_full_runtime_convergence",
        "transport_fault_matrix",
    ]
):
    raise SystemExit(
        "expected required_artifact_ids to include local_full_stack_integration, local_full_runtime_convergence, and transport_fault_matrix"
    )
for entry in inventory:
    if not isinstance(entry, dict):
        raise SystemExit("artifact inventory entry must be an object")
    if entry.get("status") != "dry_run_pending":
        raise SystemExit("expected every baseline artifact inventory entry status=dry_run_pending")
if payload.get("reason_codes") != []:
    raise SystemExit("expected empty reason_codes for baseline go/no-go gate run")
if payload.get("observed_reason_codes") != []:
    raise SystemExit("expected empty observed_reason_codes for baseline go/no-go gate run")
if payload.get("lane_mode") != "dry-run":
    raise SystemExit("expected lane_mode=dry-run in baseline go/no-go gate run")
if payload.get("run_mode_command_status") != "dry_run_no_commands_executed":
    raise SystemExit("expected run_mode_command_status=dry_run_no_commands_executed for baseline go/no-go gate run")
if payload.get("run_mode_command_count") != 0:
    raise SystemExit("expected run_mode_command_count=0 for baseline go/no-go gate run")
if payload.get("ci_fast_gate_eligible") is not True:
    raise SystemExit("expected ci_fast_gate_eligible=true for baseline go/no-go gate run")
if payload.get("ci_fast_gate_scope") != "ci-fast-gate":
    raise SystemExit("expected ci_fast_gate_scope=ci-fast-gate for baseline go/no-go gate run")
if payload.get("fast_gate_exclusion_status") != "verified":
    raise SystemExit("expected fast_gate_exclusion_status=verified for baseline go/no-go gate run")
if payload.get("fast_gate_exclusion_reason_code") != "go_no_go_gate_run_mode_excluded_from_fast_gate":
    raise SystemExit("expected deterministic fast-gate exclusion reason marker for baseline go/no-go gate run")
if payload.get("waiver_status") != "none":
    raise SystemExit("expected waiver_status=none for baseline go/no-go gate run")
if payload.get("waived_reason_codes") != []:
    raise SystemExit("expected empty waived_reason_codes for baseline go/no-go gate run")
PY

run_mode_output="$(
  KAMN_GONOGO_GATE_LOCAL_OPT_IN=1 bash "$LANE_SCRIPT" \
    --mode run \
    --max-seconds 120 \
    --output-json "$TMP_RUN_REPORT"
)"
if ! printf '%s\n' "$run_mode_output" | grep -q '^lane_mode=run$'; then
  echo "expected go/no-go gate lane run-mode marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_mode_output" | grep -q '^run_mode_command_status=executed$'; then
  echo "expected go/no-go gate lane run-mode command status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_mode_output" | grep -q '^go_no_go_evidence_status=verified$'; then
  echo "expected go/no-go gate lane run-mode evidence status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_mode_output" | grep -q '^rollback_readiness_status=verified$'; then
  echo "expected go/no-go gate lane run-mode rollback status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_mode_output" | grep -q '^dr_readiness_status=verified$'; then
  echo "expected go/no-go gate lane run-mode dr status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_mode_output" | grep -q '^local_full_stack_integration_status=verified$'; then
  echo "expected go/no-go gate lane run-mode local full-stack integration status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_mode_output" | grep -q '^local_full_runtime_convergence_status=verified$'; then
  echo "expected go/no-go gate lane run-mode local full-runtime convergence status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_mode_output" | grep -q '^transport_fault_matrix_status=verified$'; then
  echo "expected go/no-go gate lane run-mode transport fault-matrix status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_mode_output" | grep -q '^combined_reason_taxonomy_version=kamn.runtime.local-full-stack-integration-reason-taxonomy.v1$'; then
  echo "expected go/no-go gate lane run-mode combined reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_mode_output" | grep -q '^combined_transport_reason_codes=fork_choice_stale_block_height$'; then
  echo "expected go/no-go gate lane run-mode combined transport reason marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_mode_output" | grep -q '^combined_kolme_runtime_reason_code=not_run$'; then
  echo "expected go/no-go gate lane run-mode combined Kolme reason marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_mode_output" | grep -q '^kolme_runtime_commit_failure_taxonomy_version=v1$'; then
  echo "expected go/no-go gate lane run-mode Kolme failure taxonomy version marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_mode_output" | grep -q '^kolme_runtime_commit_failure_taxonomy=not_run$'; then
  echo "expected go/no-go gate lane run-mode Kolme failure taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_mode_output" | grep -q '^kolme_fixture_profile=real-node-non-synthetic-v1$'; then
  echo "expected go/no-go gate lane run-mode Kolme fixture profile marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_mode_output" | grep -q '^kolme_fixture_profile_version=v1$'; then
  echo "expected go/no-go gate lane run-mode Kolme fixture profile version marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_mode_output" | grep -q '^kolme_fixture_profile_status=planned$'; then
  echo "expected go/no-go gate lane run-mode Kolme fixture profile status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_mode_output" | grep -q '^combined_lane_marker_contract_status=verified$'; then
  echo "expected go/no-go gate lane run-mode combined marker contract status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_mode_output" | grep -q '^native_libp2p_provider_marker=p2p-live-libp2p-provider:native$'; then
  echo "expected go/no-go gate lane run-mode native provider marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_mode_output" | grep -q '^libp2p_fallback_marker_blocklist=p2p-in-memory-transport-fallback,p2p-live-libp2p-provider:contract-only$'; then
  echo "expected go/no-go gate lane run-mode fallback marker blocklist" >&2
  exit 1
fi
if ! printf '%s\n' "$run_mode_output" | grep -q '^libp2p_fallback_markers_detected=none$'; then
  echo "expected go/no-go gate lane run-mode empty fallback marker detection marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_mode_output" | grep -q '^native_libp2p_provider_marker_contract_status=verified$'; then
  echo "expected go/no-go gate lane run-mode provider marker contract status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_mode_output" | grep -q '^ci_fast_gate_eligible=false$'; then
  echo "expected go/no-go gate lane run-mode fast-gate exclusion marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_mode_output" | grep -q '^ci_fast_gate_scope=local-only$'; then
  echo "expected go/no-go gate lane run-mode local-only scope marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_mode_output" | grep -q '^fast_gate_exclusion_status=verified$'; then
  echo "expected go/no-go gate lane run-mode fast-gate exclusion status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_mode_output" | grep -q '^reason_codes=none$'; then
  echo "expected go/no-go gate lane run-mode reason codes marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_mode_output" | grep -q '^promotion_evidence_convergence_status=verified$'; then
  echo "expected go/no-go gate lane run-mode promotion evidence convergence status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_mode_output" | grep -q '^promotion_evidence_reason_code=none$'; then
  echo "expected go/no-go gate lane run-mode promotion evidence normalized reason marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_mode_output" | grep -q '^promotion_decision_reason_mapping_status=verified$'; then
  echo "expected go/no-go gate lane run-mode promotion decision mapping status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_mode_output" | grep -q '^promotion_decision_reason_code=none$'; then
  echo "expected go/no-go gate lane run-mode promotion decision normalized reason marker" >&2
  exit 1
fi

python3 - "$TMP_RUN_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("lane_mode") != "run":
    raise SystemExit("expected lane_mode=run in run-mode go/no-go gate report")
if payload.get("run_mode_command_status") != "executed":
    raise SystemExit("expected run_mode_command_status=executed in run-mode go/no-go gate report")
if not isinstance(payload.get("run_mode_command_count"), int) or payload.get("run_mode_command_count") <= 0:
    raise SystemExit("expected positive run_mode_command_count in run-mode go/no-go gate report")
if payload.get("ci_fast_gate_eligible") is not False:
    raise SystemExit("expected ci_fast_gate_eligible=false in run-mode go/no-go gate report")
if payload.get("ci_fast_gate_scope") != "local-only":
    raise SystemExit("expected ci_fast_gate_scope=local-only in run-mode go/no-go gate report")
if payload.get("fast_gate_exclusion_status") != "verified":
    raise SystemExit("expected fast_gate_exclusion_status=verified in run-mode go/no-go gate report")
if payload.get("fast_gate_exclusion_reason_code") != "go_no_go_gate_run_mode_excluded_from_fast_gate":
    raise SystemExit("expected deterministic fast-gate exclusion reason marker in run-mode go/no-go gate report")
if payload.get("reason_codes") != []:
    raise SystemExit("expected empty reason_codes list for run-mode go/no-go gate report")
if payload.get("promotion_evidence_convergence_status") != "verified":
    raise SystemExit("expected promotion_evidence_convergence_status=verified in run-mode report")
if payload.get("promotion_evidence_reason_code") != "none":
    raise SystemExit("expected promotion_evidence_reason_code=none in run-mode report")
if payload.get("promotion_decision_reason_mapping_status") != "verified":
    raise SystemExit("expected promotion_decision_reason_mapping_status=verified in run-mode report")
if payload.get("promotion_decision_reason_code") != "none":
    raise SystemExit("expected promotion_decision_reason_code=none in run-mode report")
if payload.get("go_no_go_evidence_status") != "verified":
    raise SystemExit("expected go_no_go_evidence_status=verified in run-mode go/no-go gate report")
if payload.get("rollback_readiness_status") != "verified":
    raise SystemExit("expected rollback_readiness_status=verified in run-mode go/no-go gate report")
if payload.get("dr_readiness_status") != "verified":
    raise SystemExit("expected dr_readiness_status=verified in run-mode go/no-go gate report")
if payload.get("local_full_stack_integration_status") != "verified":
    raise SystemExit("expected local_full_stack_integration_status=verified in run-mode go/no-go gate report")
if payload.get("local_full_runtime_convergence_status") != "verified":
    raise SystemExit("expected local_full_runtime_convergence_status=verified in run-mode go/no-go gate report")
if payload.get("transport_fault_matrix_status") != "verified":
    raise SystemExit("expected transport_fault_matrix_status=verified in run-mode go/no-go gate report")
if payload.get("combined_reason_taxonomy_version") != "kamn.runtime.local-full-stack-integration-reason-taxonomy.v1":
    raise SystemExit("expected combined_reason_taxonomy_version marker in run-mode go/no-go gate report")
if payload.get("combined_transport_reason_codes") != ["fork_choice_stale_block_height"]:
    raise SystemExit("expected combined_transport_reason_codes marker in run-mode go/no-go gate report")
if payload.get("combined_kolme_runtime_reason_code") != "not_run":
    raise SystemExit("expected combined_kolme_runtime_reason_code=not_run in run-mode go/no-go gate report")
if payload.get("kolme_runtime_commit_failure_taxonomy_version") != "v1":
    raise SystemExit("expected kolme_runtime_commit_failure_taxonomy_version marker in run-mode go/no-go gate report")
if payload.get("kolme_runtime_commit_failure_taxonomy") != "not_run":
    raise SystemExit("expected kolme_runtime_commit_failure_taxonomy=not_run in run-mode go/no-go gate report")
if payload.get("kolme_fixture_profile") != "real-node-non-synthetic-v1":
    raise SystemExit("expected kolme_fixture_profile marker in run-mode go/no-go gate report")
if payload.get("kolme_fixture_profile_version") != "v1":
    raise SystemExit("expected kolme_fixture_profile_version marker in run-mode go/no-go gate report")
if payload.get("kolme_fixture_profile_status") != "planned":
    raise SystemExit("expected kolme_fixture_profile_status=planned in run-mode go/no-go gate report")
if payload.get("combined_lane_marker_contract_status") != "verified":
    raise SystemExit("expected combined_lane_marker_contract_status=verified in run-mode go/no-go gate report")
if payload.get("native_libp2p_provider_marker") != "p2p-live-libp2p-provider:native":
    raise SystemExit("expected native_libp2p_provider_marker in run-mode go/no-go gate report")
if payload.get("libp2p_fallback_marker_blocklist") != [
    "p2p-in-memory-transport-fallback",
    "p2p-live-libp2p-provider:contract-only",
]:
    raise SystemExit("expected libp2p_fallback_marker_blocklist in run-mode go/no-go gate report")
if payload.get("libp2p_fallback_markers_detected") != []:
    raise SystemExit("expected no fallback markers in run-mode go/no-go gate report")
if payload.get("native_libp2p_provider_marker_contract_status") != "verified":
    raise SystemExit(
        "expected native_libp2p_provider_marker_contract_status=verified in run-mode go/no-go gate report"
    )
required_ids = payload.get("required_artifact_ids")
if not isinstance(required_ids, list) or sorted(required_ids) != sorted(
    [
        "go_no_go_evidence",
        "rollback_readiness",
        "dr_readiness",
        "local_full_stack_integration",
        "local_full_runtime_convergence",
        "transport_fault_matrix",
    ]
):
    raise SystemExit(
        "expected run-mode required_artifact_ids to include local_full_stack_integration, local_full_runtime_convergence, and transport_fault_matrix"
    )
PY

set +e
missing_opt_in_output="$(
  bash "$LANE_SCRIPT" \
    --mode run \
    --max-seconds 120 2>&1
)"
missing_opt_in_code=$?
set -e
if [ "$missing_opt_in_code" -eq 0 ]; then
  echo "expected go/no-go gate lane run mode to require explicit local opt-in" >&2
  exit 1
fi
if ! printf '%s\n' "$missing_opt_in_output" | grep -q 'run mode requires KAMN_GONOGO_GATE_LOCAL_OPT_IN=1'; then
  echo "expected deterministic run-mode local opt-in marker for go/no-go gate lane" >&2
  exit 1
fi

set +e
fault_output="$(
  bash "$LANE_SCRIPT" \
    --fault-profile gate_decision \
    --max-seconds 120 \
    --output-json "$TMP_FAULT_REPORT" 2>&1
)"
fault_code=$?
set -e
if [ "$fault_code" -eq 0 ]; then
  echo "expected go/no-go gate lane gate_decision fault profile to fail closed" >&2
  exit 1
fi
if ! printf '%s\n' "$fault_output" | grep -q 'gate_decision_fault_injection_triggered'; then
  echo "expected go/no-go gate lane gate_decision fault reason marker" >&2
  exit 1
fi

set +e
fallback_marker_fault_output="$(
  bash "$LANE_SCRIPT" \
    --fault-profile libp2p_fallback_marker \
    --max-seconds 120 \
    --output-json "$TMP_FALLBACK_MARKER_FAULT_REPORT" 2>&1
)"
fallback_marker_fault_code=$?
set -e
if [ "$fallback_marker_fault_code" -eq 0 ]; then
  echo "expected go/no-go gate lane libp2p fallback marker fault profile to fail closed" >&2
  exit 1
fi
if ! printf '%s\n' "$fallback_marker_fault_output" | grep -q 'gate_policy_libp2p_fallback_markers_detected'; then
  echo "expected go/no-go gate lane fallback marker drift reason marker" >&2
  exit 1
fi

set +e
readiness_marker_fault_output="$(
  bash "$LANE_SCRIPT" \
    --fault-profile readiness_marker_missing \
    --max-seconds 120 \
    --output-json "$TMP_READINESS_MARKER_FAULT_REPORT" 2>&1
)"
readiness_marker_fault_code=$?
set -e
if [ "$readiness_marker_fault_code" -eq 0 ]; then
  echo "expected go/no-go gate lane readiness marker fault profile to fail closed" >&2
  exit 1
fi
if ! printf '%s\n' "$readiness_marker_fault_output" | grep -q 'gate_required_artifact_status_mismatch:go_no_go_evidence'; then
  echo "expected go/no-go gate lane readiness marker fault to report deterministic mismatch reason" >&2
  exit 1
fi

tampered_manifest="$TMP_DIR/release-evidence-manifest.missing-dr.json"
python3 - "$RELEASE_MANIFEST_FILE" "$tampered_manifest" <<'PY'
import json
import pathlib
import sys

source_path = pathlib.Path(sys.argv[1])
target_path = pathlib.Path(sys.argv[2])
payload = json.loads(source_path.read_text(encoding="utf-8"))
payload["required_artifacts"] = [
    artifact
    for artifact in payload.get("required_artifacts", [])
    if artifact.get("artifact_id") != "dr_readiness"
]
target_path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
manifest_fail_output="$(
  bash "$LANE_SCRIPT" \
    --manifest-file "$tampered_manifest" \
    --max-seconds 120 \
    --output-json "$TMP_MANIFEST_FAIL_REPORT" 2>&1
)"
manifest_fail_code=$?
set -e
if [ "$manifest_fail_code" -eq 0 ]; then
  echo "expected go/no-go gate lane to fail closed on tampered release evidence manifest" >&2
  exit 1
fi
if ! printf '%s\n' "$manifest_fail_output" | grep -q 'release_manifest_missing_required_artifact:dr_readiness'; then
  echo "expected deterministic missing-artifact reason marker for tampered release evidence manifest" >&2
  exit 1
fi

tampered_convergence_missing_manifest="$TMP_DIR/release-evidence-manifest.missing-local-full-runtime.json"
python3 - "$RELEASE_MANIFEST_FILE" "$tampered_convergence_missing_manifest" <<'PY'
import json
import pathlib
import sys

source_path = pathlib.Path(sys.argv[1])
target_path = pathlib.Path(sys.argv[2])
payload = json.loads(source_path.read_text(encoding="utf-8"))
payload["required_artifacts"] = [
    artifact
    for artifact in payload.get("required_artifacts", [])
    if artifact.get("artifact_id") != "local_full_runtime_convergence"
]
target_path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
missing_convergence_output="$(
  bash "$LANE_SCRIPT" \
    --manifest-file "$tampered_convergence_missing_manifest" \
    --max-seconds 120 \
    --output-json "$TMP_CONVERGENCE_MISSING_REPORT" 2>&1
)"
missing_convergence_code=$?
set -e
if [ "$missing_convergence_code" -eq 0 ]; then
  echo "expected go/no-go gate lane to fail closed when local full-runtime convergence evidence link is missing" >&2
  exit 1
fi
if ! printf '%s\n' "$missing_convergence_output" | grep -q 'release_manifest_missing_required_artifact:local_full_runtime_convergence'; then
  echo "expected deterministic missing-link reason marker for local full-runtime convergence evidence" >&2
  exit 1
fi

tampered_convergence_marker_manifest="$TMP_DIR/release-evidence-manifest.tampered-local-full-runtime-marker.json"
python3 - "$RELEASE_MANIFEST_FILE" "$tampered_convergence_marker_manifest" <<'PY'
import json
import pathlib
import sys

source_path = pathlib.Path(sys.argv[1])
target_path = pathlib.Path(sys.argv[2])
payload = json.loads(source_path.read_text(encoding="utf-8"))
for artifact in payload.get("required_artifacts", []):
    if artifact.get("artifact_id") == "local_full_runtime_convergence":
        artifact["expected_success_marker"] = "local_full_runtime_policy_status=tampered"
target_path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_convergence_marker_output="$(
  bash "$LANE_SCRIPT" \
    --manifest-file "$tampered_convergence_marker_manifest" \
    --max-seconds 120 \
    --output-json "$TMP_CONVERGENCE_TAMPER_REPORT" 2>&1
)"
tampered_convergence_marker_code=$?
set -e
if [ "$tampered_convergence_marker_code" -eq 0 ]; then
  echo "expected go/no-go gate lane to fail closed when local full-runtime convergence success marker is tampered" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_convergence_marker_output" | grep -q 'release_manifest_success_marker_mismatch:local_full_runtime_convergence'; then
  echo "expected deterministic tampered-marker reason for local full-runtime convergence evidence" >&2
  exit 1
fi

valid_waiver="$TMP_DIR/go-no-go-waiver.valid.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$valid_waiver" <<'JSON'
{
  "schema_version": "kamn.runtime.go-no-go-gate-waiver.v1",
  "scope": "runtime_go_no_go_gate_required_artifacts",
  "expires_on": "2099-12-31",
  "allowed_reason_codes": [
    "release_manifest_missing_required_artifact:dr_readiness"
  ]
}
JSON

waiver_output="$(
  bash "$LANE_SCRIPT" \
    --manifest-file "$tampered_manifest" \
    --waiver-file "$valid_waiver" \
    --max-seconds 120 \
    --output-json "$TMP_WAIVER_REPORT"
)"
if ! printf '%s\n' "$waiver_output" | grep -q '^status=warn$'; then
  echo "expected go/no-go gate waiver path status marker to be warn" >&2
  exit 1
fi
if ! printf '%s\n' "$waiver_output" | grep -q '^policy_outcome=WARN$'; then
  echo "expected go/no-go gate waiver path policy_outcome marker to be WARN" >&2
  exit 1
fi
if ! printf '%s\n' "$waiver_output" | grep -q '^final_decision=GO$'; then
  echo "expected go/no-go gate waiver path final decision marker to stay GO" >&2
  exit 1
fi
if ! printf '%s\n' "$waiver_output" | grep -q '^waiver_status=applied$'; then
  echo "expected go/no-go gate waiver path waiver_status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$waiver_output" | grep -q '^reason_codes=release_manifest_required_artifact_waiver_applied$'; then
  echo "expected go/no-go gate waiver path reason code marker" >&2
  exit 1
fi
if ! printf '%s\n' "$waiver_output" | grep -q '^waived_reason_codes=release_manifest_missing_required_artifact:dr_readiness$'; then
  echo "expected go/no-go gate waiver path waived reason marker" >&2
  exit 1
fi

python3 - "$TMP_WAIVER_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("status") != "warn":
    raise SystemExit("expected waiver report status=warn")
if payload.get("policy_outcome") != "WARN":
    raise SystemExit("expected waiver report policy_outcome=WARN")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected waiver report final_decision=GO")
if payload.get("waiver_status") != "applied":
    raise SystemExit("expected waiver_status=applied in waiver report")
if payload.get("reason_codes") != ["release_manifest_required_artifact_waiver_applied"]:
    raise SystemExit("expected waiver-applied reason code in waiver report")
if payload.get("waived_reason_codes") != ["release_manifest_missing_required_artifact:dr_readiness"]:
    raise SystemExit("expected waived reason code list in waiver report")
required_ids = payload.get("required_artifact_ids")
if not isinstance(required_ids, list) or "dr_readiness" in required_ids:
    raise SystemExit("expected waived missing artifact id to be absent from required_artifact_ids")
PY

expired_waiver="$TMP_DIR/go-no-go-waiver.expired.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$expired_waiver" <<'JSON'
{
  "schema_version": "kamn.runtime.go-no-go-gate-waiver.v1",
  "scope": "runtime_go_no_go_gate_required_artifacts",
  "expires_on": "2000-01-01",
  "allowed_reason_codes": [
    "release_manifest_missing_required_artifact:dr_readiness"
  ]
}
JSON

set +e
expired_waiver_output="$(
  bash "$LANE_SCRIPT" \
    --manifest-file "$tampered_manifest" \
    --waiver-file "$expired_waiver" \
    --max-seconds 120 2>&1
)"
expired_waiver_code=$?
set -e
if [ "$expired_waiver_code" -eq 0 ]; then
  echo "expected go/no-go gate lane to fail closed for expired waiver metadata" >&2
  exit 1
fi
if ! printf '%s\n' "$expired_waiver_output" | grep -q 'waiver_expired'; then
  echo "expected deterministic waiver_expired marker for go/no-go gate lane waiver validation" >&2
  exit 1
fi

scope_mismatch_waiver="$TMP_DIR/go-no-go-waiver.scope-mismatch.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$scope_mismatch_waiver" <<'JSON'
{
  "schema_version": "kamn.runtime.go-no-go-gate-waiver.v1",
  "scope": "runtime_go_no_go_gate_wrong_scope",
  "expires_on": "2099-12-31",
  "allowed_reason_codes": [
    "release_manifest_missing_required_artifact:dr_readiness"
  ]
}
JSON

set +e
scope_mismatch_waiver_output="$(
  bash "$LANE_SCRIPT" \
    --manifest-file "$tampered_manifest" \
    --waiver-file "$scope_mismatch_waiver" \
    --max-seconds 120 2>&1
)"
scope_mismatch_waiver_code=$?
set -e
if [ "$scope_mismatch_waiver_code" -eq 0 ]; then
  echo "expected go/no-go gate lane to fail closed for waiver scope mismatch" >&2
  exit 1
fi
if ! printf '%s\n' "$scope_mismatch_waiver_output" | grep -q 'waiver_scope_mismatch'; then
  echo "expected deterministic waiver_scope_mismatch marker for go/no-go gate lane waiver validation" >&2
  exit 1
fi

python3 - "$TMP_FAULT_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("status") != "fail":
    raise SystemExit("expected gate decision fault report status=fail")
if payload.get("final_decision") != "NO-GO":
    raise SystemExit("expected gate decision fault report final_decision=NO-GO")
if payload.get("fault_profile") != "gate_decision":
    raise SystemExit("expected gate decision fault report fault_profile=gate_decision")
reason_codes = payload.get("reason_codes", [])
if "gate_decision_fault_injection_triggered" not in reason_codes:
    raise SystemExit("expected gate decision fault reason code in report")
if payload.get("policy_outcome") != "FAIL":
    raise SystemExit("expected gate decision fault report policy_outcome=FAIL")
if payload.get("policy_evaluator_status") != "verified":
    raise SystemExit("expected gate decision fault report policy_evaluator_status=verified")
if payload.get("observed_reason_codes") != ["gate_decision_fault_injection_triggered"]:
    raise SystemExit("expected observed_reason_codes to include deterministic gate-decision marker")
PY

python3 - "$TMP_FALLBACK_MARKER_FAULT_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("status") != "fail":
    raise SystemExit("expected fallback marker fault report status=fail")
if payload.get("final_decision") != "NO-GO":
    raise SystemExit("expected fallback marker fault report final_decision=NO-GO")
if payload.get("fault_profile") != "libp2p_fallback_marker":
    raise SystemExit("expected fallback marker fault report fault_profile=libp2p_fallback_marker")
if payload.get("policy_outcome") != "FAIL":
    raise SystemExit("expected fallback marker fault report policy_outcome=FAIL")
reason_codes = payload.get("reason_codes", [])
if "gate_policy_libp2p_fallback_markers_detected" not in reason_codes:
    raise SystemExit("expected fallback marker drift reason code in report")
if payload.get("observed_reason_codes") != []:
    raise SystemExit("expected fallback marker fault observed_reason_codes to remain empty")
PY

python3 - "$TMP_READINESS_MARKER_FAULT_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("status") != "fail":
    raise SystemExit("expected readiness marker fault report status=fail")
if payload.get("policy_outcome") != "FAIL":
    raise SystemExit("expected readiness marker fault report policy_outcome=FAIL")
if payload.get("final_decision") != "NO-GO":
    raise SystemExit("expected readiness marker fault report final_decision=NO-GO")
if payload.get("fault_profile") != "readiness_marker_missing":
    raise SystemExit("expected readiness marker fault report fault_profile=readiness_marker_missing")
reason_codes = payload.get("reason_codes", [])
if "gate_required_artifact_status_mismatch:go_no_go_evidence" not in reason_codes:
    raise SystemExit("expected readiness marker mismatch reason code in readiness fault report")
if payload.get("observed_reason_codes") != []:
    raise SystemExit("expected readiness marker fault observed_reason_codes to remain empty")
PY

set +e
warn_output="$(
  bash "$LANE_SCRIPT" \
    --fault-profile runtime_budget_warn \
    --max-seconds 120 \
    --output-json "$TMP_WARN_REPORT" 2>&1
)"
warn_code=$?
set -e
if [ "$warn_code" -eq 0 ]; then
  echo "expected go/no-go gate runtime_budget_warn profile to fail closed" >&2
  exit 1
fi
if ! printf '%s\n' "$warn_output" | grep -q '^status=fail$'; then
  echo "expected go/no-go gate runtime_budget_warn profile to emit status=fail" >&2
  exit 1
fi
if ! printf '%s\n' "$warn_output" | grep -q '^policy_outcome=FAIL$'; then
  echo "expected go/no-go gate runtime_budget_warn profile to emit policy_outcome=FAIL" >&2
  exit 1
fi
if ! printf '%s\n' "$warn_output" | grep -q '^final_decision=NO-GO$'; then
  echo "expected go/no-go gate runtime_budget_warn profile to emit final_decision=NO-GO" >&2
  exit 1
fi
if ! printf '%s\n' "$warn_output" | grep -q '^reason_codes=runtime_budget_exceeded$'; then
  echo "expected go/no-go gate runtime_budget_warn profile to emit deterministic budget reason code" >&2
  exit 1
fi

python3 - "$TMP_WARN_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("status") != "fail":
    raise SystemExit("expected runtime_budget_warn report status=fail")
if payload.get("policy_outcome") != "FAIL":
    raise SystemExit("expected runtime_budget_warn report policy_outcome=FAIL")
if payload.get("final_decision") != "NO-GO":
    raise SystemExit("expected runtime_budget_warn report final_decision=NO-GO")
if payload.get("policy_evaluator_status") != "verified":
    raise SystemExit("expected runtime_budget_warn report policy_evaluator_status=verified")
if payload.get("reason_codes") != ["runtime_budget_exceeded"]:
    raise SystemExit("expected runtime_budget_warn report reason_codes=['runtime_budget_exceeded']")
if payload.get("observed_reason_codes") != ["runtime_budget_exceeded"]:
    raise SystemExit("expected runtime_budget_warn report observed_reason_codes=['runtime_budget_exceeded']")
PY

set +e
invalid_budget_output="$(
  bash "$LANE_SCRIPT" \
    --max-seconds nope 2>&1
)"
invalid_budget_code=$?
set -e
if [ "$invalid_budget_code" -eq 0 ]; then
  echo "expected go/no-go gate lane to reject invalid max-seconds" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_budget_output" | grep -q 'KAMN_GONOGO_GATE_MAX_SECONDS must be an integer'; then
  echo "expected deterministic invalid max-seconds marker for go/no-go gate lane" >&2
  exit 1
fi

echo "go/no-go gate lane script tests passed."
