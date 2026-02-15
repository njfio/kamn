#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_local_full_stack_integration_live.sh"
TMP_REPORT="$(mktemp)"
TMP_DIR="$(mktemp -d)"
trap 'rm -f "$TMP_REPORT"; rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected local full-stack integration validation script to be executable" >&2
  exit 1
fi

validation_output="$(
  bash "$VALIDATION_SCRIPT" \
    --mode dry-run \
    --max-seconds 120 \
    --ci-fast-gate PASS \
    --output-json "$TMP_REPORT"
)"
if ! printf '%s\n' "$validation_output" | grep -q '^status=pass$'; then
  echo "expected local full-stack integration validation pass status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^final_decision=GO$'; then
  echo "expected local full-stack integration validation GO decision marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^lane_mode=dry-run$'; then
  echo "expected local full-stack integration validation dry-run mode marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^scenario_matrix_status=verified$'; then
  echo "expected local full-stack integration scenario matrix marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^full_runtime_status=verified$'; then
  echo "expected local full-stack integration full runtime marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^native_libp2p_convergence_status=planned$'; then
  echo "expected local full-stack integration native libp2p convergence marker in dry-run" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^libp2p_runtime_transport_mode=libp2p_process_isolated_convergence$'; then
  echo "expected local full-stack integration libp2p runtime transport mode marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^libp2p_native_provider_marker=p2p-live-libp2p-provider:native$'; then
  echo "expected local full-stack integration native libp2p provider marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^libp2p_fallback_marker_blocklist=p2p-in-memory-transport-fallback,p2p-live-libp2p-provider:contract-only$'; then
  echo "expected local full-stack integration libp2p fallback marker blocklist" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^libp2p_fallback_markers_detected=none$'; then
  echo "expected local full-stack integration empty fallback marker detection output" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^libp2p_provider_marker_contract_status=verified$'; then
  echo "expected local full-stack integration provider marker contract status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^libp2p_convergence_report_schema_version=kamn.runtime.libp2p-convergence-process-isolated-live-report.v1$'; then
  echo "expected local full-stack integration libp2p convergence report schema marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^libp2p_convergence_policy_schema_version=kamn.runtime.libp2p-convergence-process-isolated-live-policy-report.v1$'; then
  echo "expected local full-stack integration libp2p convergence policy schema marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^evidence_bundle_status=verified$'; then
  echo "expected local full-stack integration evidence bundle marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^transport_convergence_status=planned$'; then
  echo "expected local full-stack integration transport convergence marker in dry-run" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^signer_provenance_status=planned$'; then
  echo "expected local full-stack integration signer provenance marker in dry-run" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^runtime_commit_submission_status=planned$'; then
  echo "expected local full-stack integration runtime submit marker in dry-run" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^runtime_commit_finality_status=planned$'; then
  echo "expected local full-stack integration runtime finality marker in dry-run" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^runtime_provider_contract_status=planned$'; then
  echo "expected local full-stack integration provider contract marker in dry-run" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^runtime_provider_client_contract=KolmeRuntimeCommitLiveProvider$'; then
  echo "expected local full-stack integration provider client contract marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^runtime_signing_profile=kolme-fork-secp256k1-v1$'; then
  echo "expected local full-stack integration runtime signing profile marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^runtime_signer_attestation_schema_version=kamn.kolme.runtime-signer-attestation.v1$'; then
  echo "expected local full-stack integration runtime signer attestation schema marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^kolme_local_prerequisite_status=planned$'; then
  echo "expected local full-stack integration Kolme local prerequisite marker in dry-run" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^kolme_local_only_enforced_status=planned$'; then
  echo "expected local full-stack integration Kolme local-only marker in dry-run" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^kolme_integration_mode_status=planned$'; then
  echo "expected local full-stack integration Kolme integration mode marker in dry-run" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^kolme_integration_policy_status=planned$'; then
  echo "expected local full-stack integration Kolme integration policy marker in dry-run" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^combined_reason_taxonomy_version=kamn.runtime.local-full-stack-integration-reason-taxonomy.v1$'; then
  echo "expected local full-stack integration combined reason taxonomy version marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^combined_transport_reason_codes=fork_choice_stale_block_height$'; then
  echo "expected local full-stack integration combined transport reason marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^combined_kolme_runtime_reason_code=not_run$'; then
  echo "expected local full-stack integration combined Kolme reason marker in dry-run" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^kolme_runtime_commit_failure_taxonomy_version=v1$'; then
  echo "expected local full-stack integration Kolme runtime commit failure taxonomy version marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^kolme_runtime_commit_failure_taxonomy=not_run$'; then
  echo "expected local full-stack integration Kolme runtime commit failure taxonomy marker in dry-run" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^kolme_fixture_profile=real-node-non-synthetic-v1$'; then
  echo "expected local full-stack integration Kolme fixture profile marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^kolme_fixture_profile_version=v1$'; then
  echo "expected local full-stack integration Kolme fixture profile version marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^kolme_fixture_profile_status=planned$'; then
  echo "expected local full-stack integration Kolme fixture profile status marker in dry-run" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^run_mode_command_status=dry_run_no_commands_executed$'; then
  echo "expected local full-stack integration dry-run command marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.local-full-stack-integration-live-report.v1":
    raise SystemExit("unexpected local full-stack integration validation schema")
if payload.get("status") != "pass":
    raise SystemExit("expected local full-stack integration validation status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected local full-stack integration validation final_decision=GO")
if payload.get("lane_mode") != "dry-run":
    raise SystemExit("expected lane_mode=dry-run")
if payload.get("ci_fast_gate_eligibility") != "eligible":
    raise SystemExit("expected ci_fast_gate_eligibility=eligible")
if payload.get("run_mode_command_count") != 0:
    raise SystemExit("expected run_mode_command_count=0 for dry-run")
if payload.get("reason_code") != "dry_run_no_commands_executed":
    raise SystemExit("expected deterministic dry-run reason code")
if payload.get("transport_convergence_status") != "planned":
    raise SystemExit("expected transport_convergence_status=planned in dry-run")
if payload.get("signer_provenance_status") != "planned":
    raise SystemExit("expected signer_provenance_status=planned in dry-run")
if payload.get("runtime_commit_submission_status") != "planned":
    raise SystemExit("expected runtime_commit_submission_status=planned in dry-run")
if payload.get("runtime_commit_finality_status") != "planned":
    raise SystemExit("expected runtime_commit_finality_status=planned in dry-run")
if payload.get("runtime_provider_contract_status") != "planned":
    raise SystemExit("expected runtime_provider_contract_status=planned in dry-run")
if payload.get("runtime_provider_client_contract") != "KolmeRuntimeCommitLiveProvider":
    raise SystemExit("expected runtime_provider_client_contract marker")
if payload.get("runtime_signing_profile") != "kolme-fork-secp256k1-v1":
    raise SystemExit("expected runtime_signing_profile marker")
if payload.get("runtime_signer_attestation_schema_version") != "kamn.kolme.runtime-signer-attestation.v1":
    raise SystemExit("expected runtime_signer_attestation_schema_version marker")
if payload.get("kolme_integration_report_schema_version") != "kamn.kolme.local-kamn-live-runtime-integration-summary.v1":
    raise SystemExit("expected kolme_integration_report_schema_version marker")
if payload.get("kolme_integration_policy_schema_version") != "kamn.kolme.local-kamn-live-runtime-integration-policy-report.v1":
    raise SystemExit("expected kolme_integration_policy_schema_version marker")
if payload.get("native_libp2p_convergence_status") != "planned":
    raise SystemExit("expected native_libp2p_convergence_status=planned in dry-run")
if payload.get("libp2p_runtime_transport_mode") != "libp2p_process_isolated_convergence":
    raise SystemExit("expected libp2p_runtime_transport_mode marker")
if payload.get("libp2p_native_provider_marker") != "p2p-live-libp2p-provider:native":
    raise SystemExit("expected libp2p_native_provider_marker")
if payload.get("libp2p_fallback_marker_blocklist") != [
    "p2p-in-memory-transport-fallback",
    "p2p-live-libp2p-provider:contract-only",
]:
    raise SystemExit("expected libp2p_fallback_marker_blocklist")
if payload.get("libp2p_fallback_markers_detected") != []:
    raise SystemExit("expected no libp2p fallback markers in report")
if payload.get("libp2p_provider_marker_contract_status") != "verified":
    raise SystemExit("expected libp2p_provider_marker_contract_status")
if payload.get("libp2p_convergence_report_schema_version") != "kamn.runtime.libp2p-convergence-process-isolated-live-report.v1":
    raise SystemExit("expected libp2p_convergence_report_schema_version marker")
if payload.get("libp2p_convergence_policy_schema_version") != "kamn.runtime.libp2p-convergence-process-isolated-live-policy-report.v1":
    raise SystemExit("expected libp2p_convergence_policy_schema_version marker")
if payload.get("combined_reason_taxonomy_version") != "kamn.runtime.local-full-stack-integration-reason-taxonomy.v1":
    raise SystemExit("expected combined_reason_taxonomy_version marker")
if payload.get("combined_transport_reason_codes") != ["fork_choice_stale_block_height"]:
    raise SystemExit("expected combined_transport_reason_codes marker")
if payload.get("combined_kolme_runtime_reason_code") != "not_run":
    raise SystemExit("expected combined_kolme_runtime_reason_code=not_run in dry-run")
if payload.get("kolme_runtime_commit_failure_taxonomy_version") != "v1":
    raise SystemExit("expected kolme_runtime_commit_failure_taxonomy_version marker")
if payload.get("kolme_runtime_commit_failure_taxonomy") != "not_run":
    raise SystemExit("expected kolme_runtime_commit_failure_taxonomy=not_run in dry-run")
if payload.get("kolme_fixture_profile") != "real-node-non-synthetic-v1":
    raise SystemExit("expected kolme_fixture_profile marker")
if payload.get("kolme_fixture_profile_version") != "v1":
    raise SystemExit("expected kolme_fixture_profile_version marker")
if payload.get("kolme_fixture_profile_status") != "planned":
    raise SystemExit("expected kolme_fixture_profile_status=planned")
if payload.get("kolme_local_prerequisite_status") != "planned":
    raise SystemExit("expected kolme_local_prerequisite_status=planned")
if payload.get("kolme_local_only_enforced_status") != "planned":
    raise SystemExit("expected kolme_local_only_enforced_status=planned")
if payload.get("kolme_integration_mode_status") != "planned":
    raise SystemExit("expected kolme_integration_mode_status=planned")
if payload.get("kolme_integration_policy_status") != "planned":
    raise SystemExit("expected kolme_integration_policy_status=planned")
if payload.get("kolme_checkout_path") != "/tmp/kolme_fork":
    raise SystemExit("expected default kolme_checkout_path marker")
if payload.get("kolme_expected_remote_url") != "https://github.com/njfio/kolme_fork.git":
    raise SystemExit("expected default kolme_expected_remote_url marker")
if payload.get("kolme_expected_ref") != "refs/heads/main":
    raise SystemExit("expected default kolme_expected_ref marker")
if payload.get("kolme_base_url") != "http://127.0.0.1:3000":
    raise SystemExit("expected default kolme_base_url marker")
if payload.get("kolme_fork_chain_version") != "v0.15.2":
    raise SystemExit("expected default kolme_fork_chain_version marker")
if not isinstance(payload.get("artifact_paths"), dict):
    raise SystemExit("expected artifact_paths dictionary")
PY

set +e
run_without_opt_in_output="$(
  bash "$VALIDATION_SCRIPT" \
    --mode run \
    --max-seconds 120 \
    --ci-fast-gate PASS 2>&1
)"
run_without_opt_in_code=$?
set -e
if [ "$run_without_opt_in_code" -eq 0 ]; then
  echo "expected run mode without opt-in to fail closed" >&2
  exit 1
fi
if ! printf '%s\n' "$run_without_opt_in_output" | grep -q 'run mode requires explicit local-only opt-in via KAMN_LOCAL_FULL_STACK_INTEGRATION_OPT_IN=1'; then
  echo "expected deterministic opt-in marker for local full-stack integration run mode" >&2
  exit 1
fi

set +e
run_missing_checkout_output="$(
  KAMN_LOCAL_FULL_STACK_INTEGRATION_OPT_IN=1 bash "$VALIDATION_SCRIPT" \
    --mode run \
    --max-seconds 120 \
    --ci-fast-gate PASS \
    --kolme-checkout-path "$TMP_DIR/missing-checkout" \
    --kolme-expected-remote-url "https://github.com/njfio/kolme_fork.git" \
    --kolme-expected-ref "refs/heads/main" \
    --kolme-base-url "http://127.0.0.1:3000" \
    --kolme-fork-chain-version "v0.15.2" 2>&1
)"
run_missing_checkout_code=$?
set -e
if [ "$run_missing_checkout_code" -eq 0 ]; then
  echo "expected run mode without local checkout prerequisite to fail closed" >&2
  exit 1
fi
if ! printf '%s\n' "$run_missing_checkout_output" | grep -q 'local_kolme_checkout_missing'; then
  echo "expected deterministic local checkout missing reason for run mode prerequisites" >&2
  exit 1
fi

set +e
invalid_budget_output="$(
  bash "$VALIDATION_SCRIPT" \
    --max-seconds nope 2>&1
)"
invalid_budget_code=$?
set -e
if [ "$invalid_budget_code" -eq 0 ]; then
  echo "expected local full-stack integration validation script to reject invalid max-seconds" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_budget_output" | grep -q 'KAMN_LOCAL_FULL_STACK_INTEGRATION_MAX_SECONDS must be an integer'; then
  echo "expected deterministic invalid max-seconds marker for local full-stack integration validation script" >&2
  exit 1
fi

echo "local full-stack integration live validation tests passed."
