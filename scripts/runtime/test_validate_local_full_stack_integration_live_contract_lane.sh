#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/runtime/validate_local_full_stack_integration_live_contract_lane.sh"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_local_full_stack_integration_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_local_full_stack_integration_live_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected local full-stack integration contract lane script to be executable" >&2
  exit 1
fi
if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected local full-stack integration validation script to be executable" >&2
  exit 1
fi
if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected local full-stack integration policy checker script to be executable" >&2
  exit 1
fi

lane_report="$TMP_DIR/local-full-stack-integration-contract-lane-report.json"
policy_report="$TMP_DIR/local-full-stack-integration-policy-report.json"

lane_output="$(
  bash "$CONTRACT_LANE" \
    --mode dry-run \
    --max-seconds 120 \
    --output-json "$lane_report" \
    --policy-output-json "$policy_report"
)"
if ! printf '%s\n' "$lane_output" | grep -q '^status=pass$'; then
  echo "expected local full-stack integration contract lane status=pass marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^final_decision=GO$'; then
  echo "expected local full-stack integration contract lane final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^lane_mode=dry-run$'; then
  echo "expected local full-stack integration contract lane mode marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^local_full_stack_integration_policy_status=verified$'; then
  echo "expected local full-stack integration contract lane policy status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^local_full_stack_integration_contract_status=verified$'; then
  echo "expected local full-stack integration contract lane status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^transport_convergence_status=planned$'; then
  echo "expected local full-stack integration contract lane transport marker in dry-run" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^signer_provenance_status=planned$'; then
  echo "expected local full-stack integration contract lane signer marker in dry-run" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^runtime_commit_submission_status=planned$'; then
  echo "expected local full-stack integration contract lane runtime submit marker in dry-run" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^runtime_commit_finality_status=planned$'; then
  echo "expected local full-stack integration contract lane runtime finality marker in dry-run" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^runtime_provider_contract_status=planned$'; then
  echo "expected local full-stack integration contract lane provider marker in dry-run" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^kolme_local_prerequisite_status=planned$'; then
  echo "expected local full-stack integration contract lane Kolme local prerequisite marker in dry-run" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^kolme_local_only_enforced_status=planned$'; then
  echo "expected local full-stack integration contract lane Kolme local-only marker in dry-run" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^kolme_integration_mode_status=planned$'; then
  echo "expected local full-stack integration contract lane Kolme integration mode marker in dry-run" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^kolme_integration_policy_status=planned$'; then
  echo "expected local full-stack integration contract lane Kolme integration policy marker in dry-run" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^fail_closed_reason_code=local_full_stack_integration_policy_runtime_commit_finality_status_mismatch$'; then
  echo "expected local full-stack integration contract lane fail-closed reason marker" >&2
  exit 1
fi

python3 - "$lane_report" "$policy_report" <<'PY'
import json
import pathlib
import sys

lane_payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if lane_payload.get("schema_version") != "kamn.runtime.local-full-stack-integration-live-contract-lane-report.v1":
    raise SystemExit("unexpected local full-stack integration contract lane report schema")
if lane_payload.get("status") != "pass":
    raise SystemExit("expected contract lane status=pass")
if lane_payload.get("final_decision") != "GO":
    raise SystemExit("expected contract lane final_decision=GO")
if lane_payload.get("local_full_stack_integration_policy_status") != "verified":
    raise SystemExit("expected local_full_stack_integration_policy_status=verified")
if lane_payload.get("local_full_stack_integration_contract_status") != "verified":
    raise SystemExit("expected local_full_stack_integration_contract_status=verified")
if lane_payload.get("docs_contract_status") != "verified":
    raise SystemExit("expected docs_contract_status=verified")
if lane_payload.get("performance_budget_status") != "verified":
    raise SystemExit("expected performance_budget_status=verified")
if lane_payload.get("transport_convergence_status") != "planned":
    raise SystemExit("expected transport_convergence_status=planned in dry-run")
if lane_payload.get("signer_provenance_status") != "planned":
    raise SystemExit("expected signer_provenance_status=planned in dry-run")
if lane_payload.get("runtime_commit_submission_status") != "planned":
    raise SystemExit("expected runtime_commit_submission_status=planned in dry-run")
if lane_payload.get("runtime_commit_finality_status") != "planned":
    raise SystemExit("expected runtime_commit_finality_status=planned in dry-run")
if lane_payload.get("runtime_provider_contract_status") != "planned":
    raise SystemExit("expected runtime_provider_contract_status=planned in dry-run")
if lane_payload.get("runtime_provider_client_contract") != "KolmeRuntimeCommitLiveProvider":
    raise SystemExit("expected runtime_provider_client_contract marker")
if lane_payload.get("runtime_signing_profile") != "kolme-fork-secp256k1-v1":
    raise SystemExit("expected runtime_signing_profile marker")
if lane_payload.get("runtime_signer_attestation_schema_version") != "kamn.kolme.runtime-signer-attestation.v1":
    raise SystemExit("expected runtime_signer_attestation_schema_version marker")
if lane_payload.get("kolme_local_prerequisite_status") != "planned":
    raise SystemExit("expected kolme_local_prerequisite_status=planned in dry-run")
if lane_payload.get("kolme_local_only_enforced_status") != "planned":
    raise SystemExit("expected kolme_local_only_enforced_status=planned in dry-run")
if lane_payload.get("kolme_integration_mode_status") != "planned":
    raise SystemExit("expected kolme_integration_mode_status=planned in dry-run")
if lane_payload.get("kolme_integration_policy_status") != "planned":
    raise SystemExit("expected kolme_integration_policy_status=planned in dry-run")
if lane_payload.get("kolme_checkout_path") != "/tmp/kolme_fork":
    raise SystemExit("expected default kolme_checkout_path marker")
if lane_payload.get("kolme_expected_remote_url") != "https://github.com/njfio/kolme_fork.git":
    raise SystemExit("expected default kolme_expected_remote_url marker")
if lane_payload.get("kolme_expected_ref") != "refs/heads/main":
    raise SystemExit("expected default kolme_expected_ref marker")
if lane_payload.get("kolme_base_url") != "http://127.0.0.1:3000":
    raise SystemExit("expected default kolme_base_url marker")
if lane_payload.get("kolme_fork_chain_version") != "v0.15.2":
    raise SystemExit("expected default kolme_fork_chain_version marker")
if lane_payload.get("kolme_integration_report_schema_version") != "kamn.kolme.local-kamn-live-runtime-integration-summary.v1":
    raise SystemExit("expected kolme integration report schema marker")
if lane_payload.get("kolme_integration_policy_schema_version") != "kamn.kolme.local-kamn-live-runtime-integration-policy-report.v1":
    raise SystemExit("expected kolme integration policy schema marker")

policy_payload = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
if policy_payload.get("schema_version") != "kamn.runtime.local-full-stack-integration-live-policy-report.v1":
    raise SystemExit("unexpected local full-stack integration policy report schema")
if policy_payload.get("final_decision") != "GO":
    raise SystemExit("expected policy final_decision=GO")
if policy_payload.get("local_full_stack_integration_policy_status") != "verified":
    raise SystemExit("expected local_full_stack_integration_policy_status=verified in policy report")
PY

if ! grep -q "check_local_full_stack_integration_live_policy.sh" "$CONTRACT_LANE"; then
  echo "expected local full-stack integration contract lane to compose policy checker" >&2
  exit 1
fi
if ! grep -q "validate_local_full_stack_integration_live.sh" "$CONTRACT_LANE"; then
  echo "expected local full-stack integration contract lane to compose validation lane" >&2
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
  echo "expected local full-stack integration contract lane to reject invalid ci-fast-gate value" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_ci_fast_gate_output" | grep -q 'ci-fast-gate must be PASS or FAIL'; then
  echo "expected deterministic invalid ci-fast-gate marker for local full-stack integration contract lane" >&2
  exit 1
fi

echo "local full-stack integration contract lane tests passed."
