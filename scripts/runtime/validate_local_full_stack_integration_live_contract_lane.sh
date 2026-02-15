#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_local_full_stack_integration_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_local_full_stack_integration_live_policy.sh"
STRATEGY_DOC="$ROOT_DIR/docs/ci/strategy.md"

output_json=""
policy_output_json=""
max_seconds="${KAMN_LOCAL_FULL_STACK_INTEGRATION_CONTRACT_MAX_SECONDS:-180}"
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

summary_report="$TMP_DIR/local-full-stack-integration-summary.json"
policy_report="$TMP_DIR/local-full-stack-integration-policy.json"
tampered_report="$TMP_DIR/local-full-stack-integration-summary.tampered.json"

validation_output="$(
  bash "$VALIDATION_SCRIPT" \
    --mode "$mode" \
    --max-seconds "$max_seconds" \
    --output-json "$summary_report"
)"
domain_expected_status="planned"
if [ "$mode" = "run" ]; then
  domain_expected_status="verified"
fi
if ! printf '%s\n' "$validation_output" | grep -q '^status=pass$'; then
  echo "expected local full-stack integration validation status=pass marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^final_decision=GO$'; then
  echo "expected local full-stack integration validation final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^scenario_matrix_status=verified$'; then
  echo "expected local full-stack integration scenario matrix marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^full_runtime_status=verified$'; then
  echo "expected local full-stack integration runtime marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^evidence_bundle_status=verified$'; then
  echo "expected local full-stack integration evidence bundle marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q "^transport_convergence_status=${domain_expected_status}$"; then
  echo "expected local full-stack integration transport convergence marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q "^signer_provenance_status=${domain_expected_status}$"; then
  echo "expected local full-stack integration signer provenance marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q "^runtime_commit_submission_status=${domain_expected_status}$"; then
  echo "expected local full-stack integration runtime commit submission marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q "^runtime_commit_finality_status=${domain_expected_status}$"; then
  echo "expected local full-stack integration runtime commit finality marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q "^runtime_provider_contract_status=${domain_expected_status}$"; then
  echo "expected local full-stack integration runtime provider contract marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^runtime_provider_client_contract=KolmeRuntimeCommitLiveProvider$'; then
  echo "expected local full-stack integration runtime provider contract client marker" >&2
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

policy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$summary_report" \
    --expected-final-decision GO \
    --ci-fast-gate "$ci_fast_gate" \
    --output-json "$policy_report"
)"
if ! printf '%s\n' "$policy_output" | grep -q '^status=ok$'; then
  echo "expected local full-stack integration policy checker status=ok marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^final_decision=GO$'; then
  echo "expected local full-stack integration policy checker final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^local_full_stack_integration_policy_status=verified$'; then
  echo "expected local full-stack integration policy checker status marker" >&2
  exit 1
fi

cp "$summary_report" "$tampered_report"
python3 - "$tampered_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["runtime_commit_finality_status"] = "missing"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_policy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_report" \
    --expected-final-decision GO \
    --ci-fast-gate "$ci_fast_gate" \
    --output-json "$TMP_DIR/local-full-stack-integration-policy.tampered.json" 2>&1
)"
tampered_policy_code=$?
set -e
if [ "$tampered_policy_code" -eq 0 ]; then
  echo "expected tampered local full-stack integration report to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_policy_output" | grep -q 'local_full_stack_integration_policy_runtime_commit_finality_status_mismatch'; then
  echo "expected deterministic fail-closed reason for tampered local full-stack integration report" >&2
  exit 1
fi

for required_ref in \
  "validate_local_full_stack_integration_live.sh" \
  "check_local_full_stack_integration_live_policy.sh" \
  "validate_local_full_stack_integration_live_contract_lane.sh" \
  "test_validate_local_full_stack_integration_live.sh" \
  "test_check_local_full_stack_integration_live_policy.sh" \
  "test_validate_local_full_stack_integration_live_contract_lane.sh"; do
  if ! grep -q "$required_ref" "$STRATEGY_DOC"; then
    echo "expected CI strategy docs to reference $required_ref" >&2
    exit 1
  fi
done
if ! grep -q "local full-stack integration run-mode commands remain excluded from ci-fast-gate and ci-tools fast mode." "$STRATEGY_DOC"; then
  echo "expected CI strategy docs to include local full-stack integration run-mode exclusion marker" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "local full-stack integration contract lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

lane_report="$TMP_DIR/local-full-stack-integration-contract-lane-report.json"
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

if summary_report.get("schema_version") != "kamn.runtime.local-full-stack-integration-live-report.v1":
    raise SystemExit("unexpected local full-stack integration summary schema")
if policy_report.get("schema_version") != "kamn.runtime.local-full-stack-integration-live-policy-report.v1":
    raise SystemExit("unexpected local full-stack integration policy schema")
if summary_report.get("final_decision") != "GO":
    raise SystemExit("expected local full-stack integration summary final_decision=GO")
if policy_report.get("final_decision") != "GO":
    raise SystemExit("expected local full-stack integration policy final_decision=GO")
if summary_report.get("runtime_provider_client_contract") != "KolmeRuntimeCommitLiveProvider":
    raise SystemExit("expected runtime provider client contract marker in summary report")
if summary_report.get("runtime_signing_profile") != "kolme-fork-secp256k1-v1":
    raise SystemExit("expected runtime signing profile marker in summary report")
if summary_report.get("runtime_signer_attestation_schema_version") != "kamn.kolme.runtime-signer-attestation.v1":
    raise SystemExit("expected runtime signer attestation schema marker in summary report")
if summary_report.get("kolme_integration_report_schema_version") != "kamn.kolme.local-kamn-live-runtime-integration-summary.v1":
    raise SystemExit("expected kolme integration report schema marker in summary report")
if summary_report.get("kolme_integration_policy_schema_version") != "kamn.kolme.local-kamn-live-runtime-integration-policy-report.v1":
    raise SystemExit("expected kolme integration policy schema marker in summary report")

expected_domain_status = "planned" if mode == "dry-run" else "verified"
for marker in (
    "transport_convergence_status",
    "signer_provenance_status",
    "runtime_commit_submission_status",
    "runtime_commit_finality_status",
    "runtime_provider_contract_status",
):
    if summary_report.get(marker) != expected_domain_status:
        raise SystemExit(f"expected {marker}={expected_domain_status} in summary report")

lane_report = {
    "schema_version": "kamn.runtime.local-full-stack-integration-live-contract-lane-report.v1",
    "status": "pass",
    "final_decision": "GO",
    "lane_mode": mode,
    "local_full_stack_integration_contract_status": "verified",
    "local_full_stack_integration_policy_status": policy_report.get(
        "local_full_stack_integration_policy_status", "unknown"
    ),
    "transport_convergence_status": summary_report.get("transport_convergence_status", "unknown"),
    "signer_provenance_status": summary_report.get("signer_provenance_status", "unknown"),
    "runtime_commit_submission_status": summary_report.get("runtime_commit_submission_status", "unknown"),
    "runtime_commit_finality_status": summary_report.get("runtime_commit_finality_status", "unknown"),
    "runtime_provider_contract_status": summary_report.get("runtime_provider_contract_status", "unknown"),
    "runtime_provider_client_contract": summary_report.get("runtime_provider_client_contract", ""),
    "runtime_signing_profile": summary_report.get("runtime_signing_profile", ""),
    "runtime_signer_attestation_schema_version": summary_report.get(
        "runtime_signer_attestation_schema_version",
        "",
    ),
    "kolme_integration_report_schema_version": summary_report.get(
        "kolme_integration_report_schema_version",
        "",
    ),
    "kolme_integration_policy_schema_version": summary_report.get(
        "kolme_integration_policy_schema_version",
        "",
    ),
    "docs_contract_status": "verified",
    "performance_budget_status": "verified",
    "elapsed_seconds": elapsed_seconds,
    "max_seconds": max_seconds,
    "summary_report_file": str(pathlib.Path(sys.argv[1]).resolve()),
    "policy_report_file": str(pathlib.Path(sys.argv[2]).resolve()),
    "fail_closed_reason_code": "local_full_stack_integration_policy_runtime_commit_finality_status_mismatch",
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
echo "lane_mode=${mode}"
echo "local_full_stack_integration_contract_status=verified"
echo "local_full_stack_integration_policy_status=verified"
echo "transport_convergence_status=${domain_expected_status}"
echo "signer_provenance_status=${domain_expected_status}"
echo "runtime_commit_submission_status=${domain_expected_status}"
echo "runtime_commit_finality_status=${domain_expected_status}"
echo "runtime_provider_contract_status=${domain_expected_status}"
echo "runtime_provider_client_contract=KolmeRuntimeCommitLiveProvider"
echo "runtime_signing_profile=kolme-fork-secp256k1-v1"
echo "runtime_signer_attestation_schema_version=kamn.kolme.runtime-signer-attestation.v1"
echo "kolme_integration_report_schema_version=kamn.kolme.local-kamn-live-runtime-integration-summary.v1"
echo "kolme_integration_policy_schema_version=kamn.kolme.local-kamn-live-runtime-integration-policy-report.v1"
echo "docs_contract_status=verified"
echo "performance_budget_status=verified"
echo "fail_closed_reason_code=local_full_stack_integration_policy_runtime_commit_finality_status_mismatch"
if [[ -n "$output_json" ]]; then
  echo "report_file=$(realpath "$output_json")"
fi
