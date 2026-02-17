#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/runtime/validate_unified_api_observability_local_heavy_live_contract_lane.sh"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_unified_api_observability_local_heavy_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_unified_api_observability_local_heavy_live_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected unified API-observability local-heavy contract lane script to be executable" >&2
  exit 1
fi
if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected unified API-observability local-heavy validation script to be executable" >&2
  exit 1
fi
if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected unified API-observability local-heavy policy checker script to be executable" >&2
  exit 1
fi

lane_report="$TMP_DIR/unified-api-observability-local-heavy-contract-lane-report.json"
policy_report="$TMP_DIR/unified-api-observability-local-heavy-policy-report.json"

lane_output="$(
  bash "$CONTRACT_LANE" \
    --mode dry-run \
    --max-seconds 240 \
    --command-max-seconds 120 \
    --soak-iterations 2 \
    --output-json "$lane_report" \
    --policy-output-json "$policy_report"
)"
for marker in \
  '^status=pass$' \
  '^final_decision=GO$' \
  '^lane_mode=dry-run$' \
  '^unified_api_observability_local_heavy_contract_status=verified$' \
  '^unified_api_observability_local_heavy_policy_status=verified$' \
  '^docs_contract_status=verified$' \
  '^performance_budget_status=verified$' \
  '^policy_reason_taxonomy_version=kamn.runtime.unified-api-observability-local-heavy-policy-reason-taxonomy.v1$' \
  '^policy_reason_codes_csv=ci_fast_gate_failed,unified_api_observability_local_heavy_policy_artifact_paths_invalid,unified_api_observability_local_heavy_policy_evidence_artifact_missing,unified_api_observability_local_heavy_policy_evidence_convergence_mismatch,unified_api_observability_local_heavy_policy_evidence_links_incomplete,unified_api_observability_local_heavy_policy_ci_fast_gate_mismatch,unified_api_observability_local_heavy_policy_command_budget_exceeded,unified_api_observability_local_heavy_policy_command_count_invalid,unified_api_observability_local_heavy_policy_command_max_seconds_invalid,unified_api_observability_local_heavy_policy_compatibility_matrix_status_mismatch,unified_api_observability_local_heavy_policy_compatibility_policy_schema_mismatch,unified_api_observability_local_heavy_policy_compatibility_policy_status_mismatch,unified_api_observability_local_heavy_policy_compatibility_report_schema_mismatch,unified_api_observability_local_heavy_policy_dry_run_command_count_mismatch,unified_api_observability_local_heavy_policy_dry_run_command_status_mismatch,unified_api_observability_local_heavy_policy_dry_run_eligibility_mismatch,unified_api_observability_local_heavy_policy_dry_run_reason_code_mismatch,unified_api_observability_local_heavy_policy_dry_run_soak_iterations_executed_mismatch,unified_api_observability_local_heavy_policy_dry_run_soak_status_mismatch,unified_api_observability_local_heavy_policy_elapsed_seconds_invalid,unified_api_observability_local_heavy_policy_fast_gate_exclusion_reason_mismatch,unified_api_observability_local_heavy_policy_fast_gate_exclusion_status_mismatch,unified_api_observability_local_heavy_policy_final_decision_invalid,unified_api_observability_local_heavy_policy_final_decision_mismatch,unified_api_observability_local_heavy_policy_lane_mode_invalid,unified_api_observability_local_heavy_policy_max_seconds_invalid,unified_api_observability_local_heavy_policy_observability_policy_schema_mismatch,unified_api_observability_local_heavy_policy_observability_policy_status_mismatch,unified_api_observability_local_heavy_policy_observability_report_schema_mismatch,unified_api_observability_local_heavy_policy_observability_soak_status_mismatch,unified_api_observability_local_heavy_policy_run_mode_command_count_mismatch,unified_api_observability_local_heavy_policy_run_mode_command_status_mismatch,unified_api_observability_local_heavy_policy_run_mode_exclusion_mismatch,unified_api_observability_local_heavy_policy_run_mode_reason_code_mismatch,unified_api_observability_local_heavy_policy_run_mode_soak_iterations_executed_invalid,unified_api_observability_local_heavy_policy_run_mode_soak_iterations_mismatch,unified_api_observability_local_heavy_policy_run_mode_soak_iterations_requested_invalid,unified_api_observability_local_heavy_policy_run_mode_soak_status_mismatch,unified_api_observability_local_heavy_policy_runtime_budget_exceeded,unified_api_observability_local_heavy_policy_runtime_budget_status_mismatch,unified_api_observability_local_heavy_policy_schema_mismatch,unified_api_observability_local_heavy_policy_soak_iterations_executed_invalid,unified_api_observability_local_heavy_policy_soak_iterations_requested_invalid,unified_api_observability_local_heavy_policy_status_mismatch$' \
  '^policy_reason_codes_value=none$' \
  '^fail_closed_reason_code=unified_api_observability_local_heavy_policy_compatibility_matrix_status_mismatch$'; do
  if ! printf '%s\n' "$lane_output" | grep -q "$marker"; then
    echo "expected unified API-observability local-heavy contract lane marker: $marker" >&2
    exit 1
  fi
done

python3 - "$lane_report" "$policy_report" <<'PY'
import json
import pathlib
import sys

lane_payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if lane_payload.get("schema_version") != "kamn.runtime.unified-api-observability-local-heavy-live-contract-lane-report.v1":
    raise SystemExit("unexpected unified API-observability local-heavy contract lane schema")
if lane_payload.get("status") != "pass":
    raise SystemExit("expected contract lane status=pass")
if lane_payload.get("final_decision") != "GO":
    raise SystemExit("expected contract lane final_decision=GO")
if lane_payload.get("unified_api_observability_local_heavy_contract_status") != "verified":
    raise SystemExit("expected unified_api_observability_local_heavy_contract_status=verified")
if lane_payload.get("unified_api_observability_local_heavy_policy_status") != "verified":
    raise SystemExit("expected unified_api_observability_local_heavy_policy_status=verified")
if lane_payload.get("docs_contract_status") != "verified":
    raise SystemExit("expected docs_contract_status=verified")
if lane_payload.get("policy_reason_taxonomy_version") != "kamn.runtime.unified-api-observability-local-heavy-policy-reason-taxonomy.v1":
    raise SystemExit("expected deterministic policy_reason_taxonomy_version marker in contract lane report")
if lane_payload.get("policy_reason_codes_csv") != "ci_fast_gate_failed,unified_api_observability_local_heavy_policy_artifact_paths_invalid,unified_api_observability_local_heavy_policy_evidence_artifact_missing,unified_api_observability_local_heavy_policy_evidence_convergence_mismatch,unified_api_observability_local_heavy_policy_evidence_links_incomplete,unified_api_observability_local_heavy_policy_ci_fast_gate_mismatch,unified_api_observability_local_heavy_policy_command_budget_exceeded,unified_api_observability_local_heavy_policy_command_count_invalid,unified_api_observability_local_heavy_policy_command_max_seconds_invalid,unified_api_observability_local_heavy_policy_compatibility_matrix_status_mismatch,unified_api_observability_local_heavy_policy_compatibility_policy_schema_mismatch,unified_api_observability_local_heavy_policy_compatibility_policy_status_mismatch,unified_api_observability_local_heavy_policy_compatibility_report_schema_mismatch,unified_api_observability_local_heavy_policy_dry_run_command_count_mismatch,unified_api_observability_local_heavy_policy_dry_run_command_status_mismatch,unified_api_observability_local_heavy_policy_dry_run_eligibility_mismatch,unified_api_observability_local_heavy_policy_dry_run_reason_code_mismatch,unified_api_observability_local_heavy_policy_dry_run_soak_iterations_executed_mismatch,unified_api_observability_local_heavy_policy_dry_run_soak_status_mismatch,unified_api_observability_local_heavy_policy_elapsed_seconds_invalid,unified_api_observability_local_heavy_policy_fast_gate_exclusion_reason_mismatch,unified_api_observability_local_heavy_policy_fast_gate_exclusion_status_mismatch,unified_api_observability_local_heavy_policy_final_decision_invalid,unified_api_observability_local_heavy_policy_final_decision_mismatch,unified_api_observability_local_heavy_policy_lane_mode_invalid,unified_api_observability_local_heavy_policy_max_seconds_invalid,unified_api_observability_local_heavy_policy_observability_policy_schema_mismatch,unified_api_observability_local_heavy_policy_observability_policy_status_mismatch,unified_api_observability_local_heavy_policy_observability_report_schema_mismatch,unified_api_observability_local_heavy_policy_observability_soak_status_mismatch,unified_api_observability_local_heavy_policy_run_mode_command_count_mismatch,unified_api_observability_local_heavy_policy_run_mode_command_status_mismatch,unified_api_observability_local_heavy_policy_run_mode_exclusion_mismatch,unified_api_observability_local_heavy_policy_run_mode_reason_code_mismatch,unified_api_observability_local_heavy_policy_run_mode_soak_iterations_executed_invalid,unified_api_observability_local_heavy_policy_run_mode_soak_iterations_mismatch,unified_api_observability_local_heavy_policy_run_mode_soak_iterations_requested_invalid,unified_api_observability_local_heavy_policy_run_mode_soak_status_mismatch,unified_api_observability_local_heavy_policy_runtime_budget_exceeded,unified_api_observability_local_heavy_policy_runtime_budget_status_mismatch,unified_api_observability_local_heavy_policy_schema_mismatch,unified_api_observability_local_heavy_policy_soak_iterations_executed_invalid,unified_api_observability_local_heavy_policy_soak_iterations_requested_invalid,unified_api_observability_local_heavy_policy_status_mismatch":
    raise SystemExit("expected deterministic policy_reason_codes_csv marker in contract lane report")
if lane_payload.get("policy_reason_codes_value") != "none":
    raise SystemExit("expected policy_reason_codes_value=none in contract lane report")

policy_payload = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
if policy_payload.get("schema_version") != "kamn.runtime.unified-api-observability-local-heavy-live-policy-report.v1":
    raise SystemExit("unexpected policy report schema in contract lane")
if policy_payload.get("final_decision") != "GO":
    raise SystemExit("expected contract-lane policy final_decision=GO")
if policy_payload.get("unified_api_observability_local_heavy_policy_status") != "verified":
    raise SystemExit("expected policy status marker in contract-lane policy report")
if policy_payload.get("reason_taxonomy_version") != "kamn.runtime.unified-api-observability-local-heavy-policy-reason-taxonomy.v1":
    raise SystemExit("expected deterministic policy reason taxonomy marker in contract-lane policy report")
if policy_payload.get("reason_codes_value") != "none":
    raise SystemExit("expected reason_codes_value=none in contract-lane policy report")
PY

set +e
blocked_fast_gate_output="$(
  bash "$CONTRACT_LANE" \
    --mode dry-run \
    --ci-fast-gate FAIL 2>&1
)"
blocked_fast_gate_code=$?
set -e
if [ "$blocked_fast_gate_code" -eq 0 ]; then
  echo "expected unified API-observability local-heavy contract lane to fail closed when ci-fast-gate=FAIL" >&2
  exit 1
fi
if ! printf '%s\n' "$blocked_fast_gate_output" | grep -q 'ci_fast_gate_failed'; then
  echo "expected deterministic ci_fast_gate_failed marker for unified API-observability local-heavy contract lane" >&2
  exit 1
fi

echo "unified API-observability local-heavy contract lane tests passed."
