#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
POLICY_SCRIPT="$ROOT_DIR/scripts/runtime/check_local_full_stack_integration_live_policy.sh"
TMP_REPORT="$(mktemp)"
TMP_POLICY="$(mktemp)"
TMP_TAMPERED="$(mktemp)"
trap 'rm -f "$TMP_REPORT" "$TMP_POLICY" "$TMP_TAMPERED"' EXIT

if [ ! -x "$POLICY_SCRIPT" ]; then
  echo "expected local full-stack integration policy script to be executable" >&2
  exit 1
fi

cat >"$TMP_REPORT" <<'JSON'
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

echo "local full-stack integration policy checker tests passed."
