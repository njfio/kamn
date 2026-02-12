#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$ROOT_DIR/scripts/kolme/check_local_kamn_live_runtime_real_node_profile_policy.py"
RUNNER="$ROOT_DIR/scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
CI_DOC_FILE="$ROOT_DIR/docs/ci/strategy.md"
README_FILE="$ROOT_DIR/README.md"
TMP_DIR="$(mktemp -d)"
TMP_REPORT_OK="$TMP_DIR/ok-report.json"
TMP_REPORT_OK_SECONDARY="$TMP_DIR/ok-report-secondary.json"
TMP_REPORT_BAD="$TMP_DIR/bad-report.json"
TMP_REPORT_SYNTHETIC="$TMP_DIR/synthetic-report.json"
TMP_REPORT_INMEMORY="$TMP_DIR/inmemory-report.json"
TMP_POLICY_OUT="$TMP_DIR/policy-report.json"
TMP_POLICY_OUT_SECONDARY="$TMP_DIR/policy-report-secondary.json"
TMP_INTEGRATION_POLICY_OUT="$TMP_DIR/integration-policy-report.json"
TMP_INTEGRATION_POLICY_OUT_SECONDARY="$TMP_DIR/integration-policy-secondary-report.json"
TMP_SUMMARY="$TMP_DIR/integration-summary.json"
TMP_SUMMARY_SECONDARY="$TMP_DIR/integration-summary-secondary.json"
TMP_RUNTIME_SUMMARY="$TMP_DIR/runtime-summary.json"
TMP_RUNTIME_POLICY="$TMP_DIR/runtime-policy.json"
TMP_ERR="$TMP_DIR/error.log"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$CHECKER" ]; then
  echo "expected local KAMN live runtime real-node profile policy checker to be executable" >&2
  exit 1
fi

if ! grep -q "check_local_kamn_live_runtime_real_node_profile_policy.py" "$DOC_FILE"; then
  echo "expected Kolme devnet ops docs to reference real-node profile policy checker command" >&2
  exit 1
fi

if ! grep -q "check_local_kamn_live_runtime_real_node_profile_policy.py" "$CI_DOC_FILE"; then
  echo "expected CI strategy docs to reference real-node profile policy checker command" >&2
  exit 1
fi

if ! grep -q "check_local_kamn_live_runtime_real_node_profile_policy.py" "$README_FILE"; then
  echo "expected README to reference real-node profile policy checker command" >&2
  exit 1
fi

cat >"$TMP_REPORT_OK" <<'JSON'
{
  "schema_version": "kamn.kolme.local-kamn-live-runtime-integration-summary.v1",
  "mode": "dry-run",
  "status": "ok",
  "reason_code": "dry_run_no_commands_executed",
  "local_only_enforced": true,
  "ci_fast_gate_eligible": false,
  "elapsed_seconds": 0,
  "max_seconds": 210,
  "budget_status": "not_run",
  "checkout_path": "/tmp/kolme_fork",
  "expected_remote_url": "https://github.com/njfio/kolme_fork.git",
  "expected_ref": "refs/heads/main",
  "base_url": "http://127.0.0.1:3000",
  "fork_chain_version": "v0.15.2",
  "runtime_profile": "real-node",
  "runtime_provider_client_contract": "KolmeRuntimeCommitLiveProvider",
  "runtime_signer_profile_selector_env": "KAMN_KOLME_LIVE_SIGNER_PROFILE",
  "runtime_signer_profile": "ops-primary",
  "runtime_signer_previous_profile": "ops-primary",
  "runtime_signer_failover_active": false,
  "runtime_signer_rotation_epoch": 1,
  "runtime_signer_previous_rotation_epoch": 1,
  "runtime_signer_private_key_env": "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
  "runtime_commit_command_profile": "real-node-non-synthetic-v1",
  "runtime_commit_policy_command_profile": "real-node-non-synthetic-v1",
  "runtime_commit_command_profile_version": "v1",
  "runtime_commit_command": "KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_runtime_commit_live_finality_evidence_contract_lane.sh --expected-provider-client-contract KolmeRuntimeCommitLiveProvider --require-non-synthetic-run-evidence --require-native-payload-evidence --live-command \"KAMN_KOLME_LIVE_BASE_URL=http://127.0.0.1:3000 KAMN_KOLME_LIVE_PROVIDER_HINT=kolme-fork-local KAMN_KOLME_LIVE_SIGNER_PROFILE=ops-primary KAMN_KOLME_LIVE_SIGNING_PROFILE=kolme-fork-secp256k1-v1 cargo test -p kamn-core --test kolme_runtime_commit_http_transport -- --ignored --exact integration_kolme_fork_live_node_submit_reaches_endpoint && printf 'status=submitted\\\\n{\\\"pubkey\\\":\\\"proof\\\",\\\"nonce\\\":1,\\\"messages\\\":[]}\\\\n'\" --output-json /tmp/runtime-summary.json --policy-output-json /tmp/runtime-policy.json",
  "runtime_commit_live_policy_report": "/tmp/runtime-policy.json",
  "runtime_commit_finality_command": "",
  "runtime_commit_finality_output_file": "",
  "runtime_commit_finality_enabled": false,
  "runtime_commit_finality_max_seconds": 15,
  "bootstrap_reason_code": "not_run",
  "localhost_signed_reason_code": "not_run",
  "conformance_reason_code": "not_run",
  "runtime_commit_reason_code": "not_run",
  "runtime_commit_policy_reason_code": "not_run",
  "contracts": {
    "ci_fast_gate_scope": "local-only",
    "runtime_profile": "real-node",
    "runtime_provider_client_contract": "KolmeRuntimeCommitLiveProvider",
    "runtime_signer_profile_selector_env": "KAMN_KOLME_LIVE_SIGNER_PROFILE",
    "runtime_signer_profile": "ops-primary",
    "runtime_signer_failover_requires_profile_change": true,
    "runtime_signer_rotation_epoch_must_increase_on_failover": true,
    "runtime_signer_private_key_env": "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
    "runtime_commit_endpoint": "/broadcast/runtime-commit",
    "runtime_commit_method": "POST",
    "runtime_commit_finality_primary_endpoint": "/notifications",
    "runtime_commit_finality_fallback_endpoint": "/block/{height}"
  },
  "checks": [
    {
      "id": "bootstrap_readiness",
      "command": "bash scripts/kolme/run_local_kolme_fork_bootstrap_readiness_lane.sh --mode run",
      "status": "planned",
      "reason_code": "not_run"
    },
    {
      "id": "localhost_signed_integration",
      "command": "bash scripts/sdk/run_localhost_signed_integration_contract_lane.sh --output-json /tmp/localhost-signed.json",
      "status": "planned",
      "reason_code": "not_run"
    },
    {
      "id": "live_api_conformance",
      "command": "bash scripts/kolme/run_local_kolme_live_api_conformance_harness.sh --mode run",
      "status": "planned",
      "reason_code": "not_run"
    },
    {
      "id": "runtime_commit_endpoint",
      "command": "KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_runtime_commit_live_finality_evidence_contract_lane.sh --expected-provider-client-contract KolmeRuntimeCommitLiveProvider --require-non-synthetic-run-evidence --require-native-payload-evidence --output-json /tmp/runtime-summary.json --policy-output-json /tmp/runtime-policy.json",
      "status": "planned",
      "reason_code": "not_run"
    },
    {
      "id": "runtime_commit_policy",
      "command": "python3 scripts/kolme/check_local_runtime_commit_live_evidence_policy.py --report-file /tmp/runtime-summary.json --expected-final-decision GO --ci-fast-gate PASS --require-non-synthetic-run-evidence --require-native-payload-evidence --output-json /tmp/runtime-policy.json",
      "status": "planned",
      "reason_code": "not_run"
    }
  ],
  "artifact_paths": [
    "/tmp/bootstrap-summary.json",
    "/tmp/localhost-signed.json",
    "/tmp/conformance-summary.json",
    "/tmp/runtime-output.log",
    "/tmp/runtime-summary.json",
    "/tmp/runtime-policy.json"
  ]
}
JSON

start_epoch="$(date +%s)"
python3 "$CHECKER" \
  --report-file "$TMP_REPORT_OK" \
  --expected-final-decision GO \
  --ci-fast-gate PASS \
  --require-reason-code dry_run_no_commands_executed \
  --require-non-synthetic-run-evidence \
  --output-json "$TMP_POLICY_OUT" >/dev/null
elapsed_seconds="$(( $(date +%s) - start_epoch ))"

if [ "$elapsed_seconds" -gt 2 ]; then
  echo "expected real-node profile policy checker to complete in <=2 seconds for fixture input" >&2
  exit 1
fi

python3 - "$TMP_POLICY_OUT" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if report.get("schema_version") != "kamn.kolme.local-kamn-live-runtime-real-node-policy-report.v1":
    raise SystemExit("unexpected real-node profile policy report schema")
if report.get("final_decision") != "GO":
    raise SystemExit("expected final_decision GO for valid real-node profile report")
if report.get("reason_codes") != []:
    raise SystemExit("expected no reason codes for valid real-node profile report")
PY

python3 - "$TMP_REPORT_OK" "$TMP_REPORT_OK_SECONDARY" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
report["runtime_signer_profile"] = "ops-secondary"
report["runtime_signer_previous_profile"] = "ops-secondary"
report["runtime_signer_private_key_env"] = "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY"
report["runtime_commit_command"] = str(report["runtime_commit_command"]).replace(
    "KAMN_KOLME_LIVE_SIGNER_PROFILE=ops-primary",
    "KAMN_KOLME_LIVE_SIGNER_PROFILE=ops-secondary",
)
contracts = report.get("contracts", {})
if isinstance(contracts, dict):
    contracts["runtime_signer_profile"] = "ops-secondary"
    contracts["runtime_signer_private_key_env"] = "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY"
pathlib.Path(sys.argv[2]).write_text(
    json.dumps(report, sort_keys=True, indent=2) + "\n",
    encoding="utf-8",
)
PY

python3 "$CHECKER" \
  --report-file "$TMP_REPORT_OK_SECONDARY" \
  --expected-final-decision GO \
  --ci-fast-gate PASS \
  --require-reason-code dry_run_no_commands_executed \
  --require-non-synthetic-run-evidence \
  --output-json "$TMP_POLICY_OUT_SECONDARY" >/dev/null

python3 - "$TMP_POLICY_OUT_SECONDARY" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if report.get("schema_version") != "kamn.kolme.local-kamn-live-runtime-real-node-policy-report.v1":
    raise SystemExit("unexpected secondary real-node profile policy report schema")
if report.get("final_decision") != "GO":
    raise SystemExit("expected final_decision GO for valid secondary real-node profile report")
if report.get("reason_codes") != []:
    raise SystemExit("expected no reason codes for valid secondary real-node profile report")
PY

cat >"$TMP_REPORT_BAD" <<'JSON'
{
  "schema_version": "kamn.kolme.local-kamn-live-runtime-integration-summary.v1",
  "mode": "dry-run",
  "status": "ok",
  "reason_code": "dry_run_no_commands_executed",
  "local_only_enforced": true,
  "ci_fast_gate_eligible": false,
  "elapsed_seconds": 0,
  "max_seconds": 210,
  "budget_status": "not_run",
  "runtime_profile": "standard",
  "runtime_provider_client_contract": "InMemoryKolmeRuntimeCommitClient",
  "runtime_signer_profile_selector_env": "KAMN_KOLME_LIVE_SIGNER_PROFILE",
  "runtime_signer_profile": "ops-primary",
  "runtime_signer_previous_profile": "ops-primary",
  "runtime_signer_failover_active": true,
  "runtime_signer_rotation_epoch": 4,
  "runtime_signer_previous_rotation_epoch": 4,
  "runtime_signer_private_key_env": "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY",
  "runtime_commit_command_profile": "standard-default-v1",
  "runtime_commit_policy_command_profile": "standard-default-v1",
  "runtime_commit_command_profile_version": "v0",
  "runtime_commit_command": "echo runtime",
  "runtime_commit_live_policy_report": "/tmp/runtime-policy.json",
  "contracts": {
    "ci_fast_gate_scope": "ci-fast-gate",
    "runtime_profile": "standard",
    "runtime_provider_client_contract": "InMemoryKolmeRuntimeCommitClient",
    "runtime_signer_profile_selector_env": "KAMN_KOLME_LIVE_SIGNER_PROFILE",
    "runtime_signer_profile": "ops-primary",
    "runtime_signer_failover_requires_profile_change": false,
    "runtime_signer_rotation_epoch_must_increase_on_failover": false,
    "runtime_signer_private_key_env": "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY"
  },
  "checks": [
    {
      "id": "bootstrap_readiness",
      "command": "echo bootstrap",
      "status": "planned",
      "reason_code": "not_run"
    }
  ],
  "artifact_paths": [
    "/tmp/runtime-policy.json"
  ]
}
JSON

set +e
python3 "$CHECKER" \
  --report-file "$TMP_REPORT_BAD" \
  --expected-final-decision GO \
  --ci-fast-gate PASS \
  --require-reason-code dry_run_no_commands_executed \
  --require-non-synthetic-run-evidence \
  --output-json "$TMP_POLICY_OUT" >"$TMP_ERR" 2>&1
bad_exit_code=$?
set -e

if [ "$bad_exit_code" -eq 0 ]; then
  echo "expected real-node profile policy checker to fail for marker drift" >&2
  exit 1
fi

if ! grep -q "runtime_profile_mismatch" "$TMP_ERR"; then
  echo "expected runtime profile mismatch reason for policy failure" >&2
  exit 1
fi

if ! grep -q "runtime_provider_client_contract_mismatch" "$TMP_ERR"; then
  echo "expected provider contract mismatch reason for policy failure" >&2
  exit 1
fi

if ! grep -q "runtime_commit_command_profile_mismatch" "$TMP_ERR"; then
  echo "expected runtime commit command profile mismatch reason for policy failure" >&2
  exit 1
fi

if ! grep -q "runtime_commit_policy_command_profile_mismatch" "$TMP_ERR"; then
  echo "expected runtime commit policy command profile mismatch reason for policy failure" >&2
  exit 1
fi

if ! grep -q "runtime_signer_failover_profile_unchanged" "$TMP_ERR"; then
  echo "expected failover unchanged reason for policy failure" >&2
  exit 1
fi

if ! grep -q "runtime_signer_rotation_epoch_stale" "$TMP_ERR"; then
  echo "expected stale rotation epoch reason for policy failure" >&2
  exit 1
fi

if ! grep -q "runtime_signer_private_key_env_mismatch" "$TMP_ERR"; then
  echo "expected signer private key env mismatch reason for policy failure" >&2
  exit 1
fi

if ! grep -q "runtime_commit_command_profile_version_mismatch" "$TMP_ERR"; then
  echo "expected runtime commit profile marker version mismatch reason for policy failure" >&2
  exit 1
fi

if ! grep -q "runtime_commit_non_synthetic_policy_marker_missing" "$TMP_ERR"; then
  echo "expected strict non-synthetic marker requirement reason for policy failure" >&2
  exit 1
fi

if ! grep -q "check_missing:runtime_commit_endpoint" "$TMP_ERR"; then
  echo "expected missing runtime_commit_endpoint check marker for policy failure" >&2
  exit 1
fi

cat >"$TMP_REPORT_SYNTHETIC" <<'JSON'
{
  "schema_version": "kamn.kolme.local-kamn-live-runtime-integration-summary.v1",
  "mode": "dry-run",
  "status": "ok",
  "reason_code": "dry_run_no_commands_executed",
  "local_only_enforced": true,
  "ci_fast_gate_eligible": false,
  "elapsed_seconds": 0,
  "max_seconds": 210,
  "budget_status": "not_run",
  "runtime_profile": "real-node",
  "runtime_provider_client_contract": "KolmeRuntimeCommitLiveProvider",
  "runtime_signer_profile_selector_env": "KAMN_KOLME_LIVE_SIGNER_PROFILE",
  "runtime_signer_profile": "ops-primary",
  "runtime_signer_private_key_env": "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
  "runtime_commit_command_profile": "real-node-non-synthetic-v1",
  "runtime_commit_policy_command_profile": "real-node-non-synthetic-v1",
  "runtime_commit_command_profile_version": "v1",
  "runtime_commit_command": "KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_runtime_commit_live_finality_evidence_contract_lane.sh --expected-provider-client-contract KolmeRuntimeCommitLiveProvider --require-non-synthetic-run-evidence --live-command \"printf 'runtime=synthetic\\\\n'\" --output-json /tmp/runtime-summary.json --policy-output-json /tmp/runtime-policy.json",
  "runtime_commit_live_policy_report": "/tmp/runtime-policy.json",
  "runtime_commit_finality_command": "",
  "runtime_commit_finality_output_file": "",
  "runtime_commit_finality_enabled": false,
  "runtime_commit_finality_max_seconds": 15,
  "bootstrap_reason_code": "not_run",
  "localhost_signed_reason_code": "not_run",
  "conformance_reason_code": "not_run",
  "runtime_commit_reason_code": "not_run",
  "runtime_commit_policy_reason_code": "not_run",
  "contracts": {
    "ci_fast_gate_scope": "local-only",
    "runtime_profile": "real-node",
    "runtime_provider_client_contract": "KolmeRuntimeCommitLiveProvider",
    "runtime_signer_profile_selector_env": "KAMN_KOLME_LIVE_SIGNER_PROFILE",
    "runtime_signer_profile": "ops-primary",
    "runtime_signer_private_key_env": "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
    "runtime_commit_endpoint": "/broadcast/runtime-commit",
    "runtime_commit_method": "POST",
    "runtime_commit_finality_primary_endpoint": "/notifications",
    "runtime_commit_finality_fallback_endpoint": "/block/{height}"
  },
  "checks": [
    {
      "id": "bootstrap_readiness",
      "command": "bash scripts/kolme/run_local_kolme_fork_bootstrap_readiness_lane.sh --mode run",
      "status": "planned",
      "reason_code": "not_run"
    },
    {
      "id": "localhost_signed_integration",
      "command": "bash scripts/sdk/run_localhost_signed_integration_contract_lane.sh --output-json /tmp/localhost-signed.json",
      "status": "planned",
      "reason_code": "not_run"
    },
    {
      "id": "live_api_conformance",
      "command": "bash scripts/kolme/run_local_kolme_live_api_conformance_harness.sh --mode run",
      "status": "planned",
      "reason_code": "not_run"
    },
    {
      "id": "runtime_commit_endpoint",
      "command": "KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_runtime_commit_live_finality_evidence_contract_lane.sh --expected-provider-client-contract KolmeRuntimeCommitLiveProvider --require-non-synthetic-run-evidence --live-command \"printf 'runtime=synthetic\\\\n'\" --output-json /tmp/runtime-summary.json --policy-output-json /tmp/runtime-policy.json",
      "status": "planned",
      "reason_code": "not_run"
    },
    {
      "id": "runtime_commit_policy",
      "command": "python3 scripts/kolme/check_local_runtime_commit_live_evidence_policy.py --report-file /tmp/runtime-summary.json --expected-final-decision GO --ci-fast-gate PASS --require-non-synthetic-run-evidence --require-native-payload-evidence --output-json /tmp/runtime-policy.json",
      "status": "planned",
      "reason_code": "not_run"
    }
  ],
  "artifact_paths": [
    "/tmp/bootstrap-summary.json",
    "/tmp/localhost-signed.json",
    "/tmp/conformance-summary.json",
    "/tmp/runtime-output.log",
    "/tmp/runtime-summary.json",
    "/tmp/runtime-policy.json"
  ]
}
JSON

set +e
python3 "$CHECKER" \
  --report-file "$TMP_REPORT_SYNTHETIC" \
  --expected-final-decision NO-GO \
  --ci-fast-gate PASS \
  --require-non-synthetic-run-evidence \
  --output-json "$TMP_POLICY_OUT" >"$TMP_ERR" 2>&1
synthetic_exit_code=$?
set -e

if [ "$synthetic_exit_code" -eq 0 ]; then
  echo "expected real-node profile policy checker to fail for synthetic command regression" >&2
  exit 1
fi

if ! grep -q "runtime_commit_non_synthetic_submit_probe_missing" "$TMP_ERR"; then
  echo "expected non-synthetic submit probe marker requirement reason for synthetic policy failure" >&2
  exit 1
fi

if ! grep -q "runtime_commit_real_signing_profile_marker_missing" "$TMP_ERR"; then
  echo "expected real signing profile marker requirement reason for synthetic policy failure" >&2
  exit 1
fi

if ! grep -q "runtime_commit_signer_profile_marker_missing" "$TMP_ERR"; then
  echo "expected signer profile marker requirement reason for synthetic policy failure" >&2
  exit 1
fi

cat >"$TMP_REPORT_INMEMORY" <<'JSON'
{
  "schema_version": "kamn.kolme.local-kamn-live-runtime-integration-summary.v1",
  "mode": "dry-run",
  "status": "ok",
  "reason_code": "dry_run_no_commands_executed",
  "local_only_enforced": true,
  "ci_fast_gate_eligible": false,
  "elapsed_seconds": 0,
  "max_seconds": 210,
  "budget_status": "not_run",
  "runtime_profile": "real-node",
  "runtime_provider_client_contract": "KolmeRuntimeCommitLiveProvider",
  "runtime_signer_profile_selector_env": "KAMN_KOLME_LIVE_SIGNER_PROFILE",
  "runtime_signer_profile": "ops-primary",
  "runtime_signer_private_key_env": "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
  "runtime_commit_command_profile": "real-node-non-synthetic-v1",
  "runtime_commit_policy_command_profile": "real-node-non-synthetic-v1",
  "runtime_commit_command_profile_version": "v1",
  "runtime_commit_command": "KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_runtime_commit_live_finality_evidence_contract_lane.sh --expected-provider-client-contract KolmeRuntimeCommitLiveProvider --require-non-synthetic-run-evidence --require-native-payload-evidence --live-command \"KAMN_KOLME_LIVE_BASE_URL=http://127.0.0.1:3000 KAMN_KOLME_LIVE_PROVIDER_HINT=kolme-fork-local KAMN_KOLME_LIVE_SIGNER_PROFILE=ops-primary KAMN_KOLME_LIVE_SIGNING_PROFILE=kolme-fork-secp256k1-v1 cargo test -p kamn-core --test kolme_runtime_commit_http_transport -- --ignored --exact integration_kolme_fork_live_node_submit_reaches_endpoint && printf 'status=submitted\\\\n{\\\"pubkey\\\":\\\"proof\\\",\\\"nonce\\\":1,\\\"messages\\\":[]}\\\\n'\" --provider-hint InMemoryKolmeRuntimeCommitClient --output-json /tmp/runtime-summary.json --policy-output-json /tmp/runtime-policy.json",
  "runtime_commit_live_policy_report": "/tmp/runtime-policy.json",
  "runtime_commit_finality_command": "",
  "runtime_commit_finality_output_file": "",
  "runtime_commit_finality_enabled": false,
  "runtime_commit_finality_max_seconds": 15,
  "bootstrap_reason_code": "not_run",
  "localhost_signed_reason_code": "not_run",
  "conformance_reason_code": "not_run",
  "runtime_commit_reason_code": "not_run",
  "runtime_commit_policy_reason_code": "not_run",
  "contracts": {
    "ci_fast_gate_scope": "local-only",
    "runtime_profile": "real-node",
    "runtime_provider_client_contract": "KolmeRuntimeCommitLiveProvider",
    "runtime_signer_profile_selector_env": "KAMN_KOLME_LIVE_SIGNER_PROFILE",
    "runtime_signer_profile": "ops-primary",
    "runtime_signer_private_key_env": "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
    "runtime_commit_endpoint": "/broadcast/runtime-commit",
    "runtime_commit_method": "POST",
    "runtime_commit_finality_primary_endpoint": "/notifications",
    "runtime_commit_finality_fallback_endpoint": "/block/{height}"
  },
  "checks": [
    {
      "id": "bootstrap_readiness",
      "command": "bash scripts/kolme/run_local_kolme_fork_bootstrap_readiness_lane.sh --mode run",
      "status": "planned",
      "reason_code": "not_run"
    },
    {
      "id": "localhost_signed_integration",
      "command": "bash scripts/sdk/run_localhost_signed_integration_contract_lane.sh --output-json /tmp/localhost-signed.json",
      "status": "planned",
      "reason_code": "not_run"
    },
    {
      "id": "live_api_conformance",
      "command": "bash scripts/kolme/run_local_kolme_live_api_conformance_harness.sh --mode run",
      "status": "planned",
      "reason_code": "not_run"
    },
    {
      "id": "runtime_commit_endpoint",
      "command": "KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_runtime_commit_live_finality_evidence_contract_lane.sh --expected-provider-client-contract KolmeRuntimeCommitLiveProvider --require-non-synthetic-run-evidence --require-native-payload-evidence --live-command \"KAMN_KOLME_LIVE_BASE_URL=http://127.0.0.1:3000 KAMN_KOLME_LIVE_PROVIDER_HINT=kolme-fork-local KAMN_KOLME_LIVE_SIGNING_PROFILE=kolme-fork-secp256k1-v1 cargo test -p kamn-core --test kolme_runtime_commit_http_transport -- --ignored --exact integration_kolme_fork_live_node_submit_reaches_endpoint && printf 'status=submitted\\\\n{\\\"pubkey\\\":\\\"proof\\\",\\\"nonce\\\":1,\\\"messages\\\":[]}\\\\n'\" --provider-hint InMemoryKolmeRuntimeCommitClient --output-json /tmp/runtime-summary.json --policy-output-json /tmp/runtime-policy.json",
      "status": "planned",
      "reason_code": "not_run"
    },
    {
      "id": "runtime_commit_policy",
      "command": "python3 scripts/kolme/check_local_runtime_commit_live_evidence_policy.py --report-file /tmp/runtime-summary.json --expected-final-decision GO --ci-fast-gate PASS --require-non-synthetic-run-evidence --require-native-payload-evidence --output-json /tmp/runtime-policy.json",
      "status": "planned",
      "reason_code": "not_run"
    }
  ],
  "artifact_paths": [
    "/tmp/bootstrap-summary.json",
    "/tmp/localhost-signed.json",
    "/tmp/conformance-summary.json",
    "/tmp/runtime-output.log",
    "/tmp/runtime-summary.json",
    "/tmp/runtime-policy.json"
  ]
}
JSON

set +e
python3 "$CHECKER" \
  --report-file "$TMP_REPORT_INMEMORY" \
  --expected-final-decision NO-GO \
  --ci-fast-gate PASS \
  --require-non-synthetic-run-evidence \
  --output-json "$TMP_POLICY_OUT" >"$TMP_ERR" 2>&1
inmemory_exit_code=$?
set -e

if [ "$inmemory_exit_code" -eq 0 ]; then
  echo "expected real-node profile policy checker to fail for in-memory provider reference drift" >&2
  exit 1
fi

if ! grep -q "runtime_commit_in_memory_provider_reference_detected" "$TMP_ERR"; then
  echo "expected in-memory provider reference reason for policy failure" >&2
  exit 1
fi

bash "$RUNNER" \
  --mode dry-run \
  --runtime-profile real-node \
  --runtime-provider-client-contract KolmeRuntimeCommitLiveProvider \
  --runtime-commit-live-summary "$TMP_RUNTIME_SUMMARY" \
  --runtime-commit-live-policy-report "$TMP_RUNTIME_POLICY" \
  --output-json "$TMP_SUMMARY" >/dev/null

python3 "$CHECKER" \
  --report-file "$TMP_SUMMARY" \
  --expected-final-decision GO \
  --ci-fast-gate PASS \
  --require-reason-code dry_run_no_commands_executed \
  --require-non-synthetic-run-evidence \
  --output-json "$TMP_INTEGRATION_POLICY_OUT" >/dev/null

bash "$RUNNER" \
  --mode dry-run \
  --runtime-profile real-node \
  --runtime-signer-profile ops-secondary \
  --runtime-provider-client-contract KolmeRuntimeCommitLiveProvider \
  --runtime-commit-live-summary "$TMP_RUNTIME_SUMMARY" \
  --runtime-commit-live-policy-report "$TMP_RUNTIME_POLICY" \
  --output-json "$TMP_SUMMARY_SECONDARY" >/dev/null

python3 "$CHECKER" \
  --report-file "$TMP_SUMMARY_SECONDARY" \
  --expected-final-decision GO \
  --ci-fast-gate PASS \
  --require-reason-code dry_run_no_commands_executed \
  --require-non-synthetic-run-evidence \
  --output-json "$TMP_INTEGRATION_POLICY_OUT_SECONDARY" >/dev/null

python3 - "$TMP_SUMMARY" <<'PY'
import json
import pathlib
import sys

summary = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
runtime_commit_command = summary.get("runtime_commit_command")
if not isinstance(runtime_commit_command, str):
    raise SystemExit("expected runtime_commit_command string in runner-generated summary")
if "--require-non-synthetic-run-evidence" not in runtime_commit_command:
    raise SystemExit("expected strict non-synthetic runtime marker in real-node profile runtime commit command")
if "--require-native-payload-evidence" not in runtime_commit_command:
    raise SystemExit("expected native payload evidence marker in real-node profile runtime commit command")
if "integration_kolme_fork_live_node_submit_reaches_endpoint" not in runtime_commit_command:
    raise SystemExit("expected non-synthetic runtime submit probe marker in real-node profile runtime commit command")
if "KAMN_KOLME_LIVE_SIGNING_PROFILE=kolme-fork-secp256k1-v1" not in runtime_commit_command:
    raise SystemExit("expected real signing profile marker in real-node profile runtime commit command")
if "KAMN_KOLME_LIVE_SIGNER_PROFILE=ops-primary" not in runtime_commit_command:
    raise SystemExit("expected signer profile marker in real-node profile runtime commit command")
if "pubkey" not in runtime_commit_command:
    raise SystemExit("expected native payload pubkey marker in real-node profile runtime commit command")
if "nonce" not in runtime_commit_command:
    raise SystemExit("expected native payload nonce marker in real-node profile runtime commit command")
if "messages" not in runtime_commit_command:
    raise SystemExit("expected native payload messages marker in real-node profile runtime commit command")
if summary.get("runtime_commit_command_profile") != "real-node-non-synthetic-v1":
    raise SystemExit("expected deterministic runtime commit command profile marker in runner-generated summary")
if summary.get("runtime_commit_policy_command_profile") != "real-node-non-synthetic-v1":
    raise SystemExit("expected deterministic runtime commit policy command profile marker in runner-generated summary")
if summary.get("runtime_commit_command_profile_version") != "v1":
    raise SystemExit("expected runtime commit command profile marker version in runner-generated summary")
if summary.get("runtime_signer_profile_selector_env") != "KAMN_KOLME_LIVE_SIGNER_PROFILE":
    raise SystemExit("expected signer profile selector env marker in runner-generated summary")
if summary.get("runtime_signer_profile") != "ops-primary":
    raise SystemExit("expected signer profile marker in runner-generated summary")
if summary.get("runtime_signer_previous_profile") != "ops-primary":
    raise SystemExit("expected signer previous-profile marker in runner-generated summary")
if summary.get("runtime_signer_failover_active") is not False:
    raise SystemExit("expected signer failover-active marker false in runner-generated summary")
if summary.get("runtime_signer_rotation_epoch") != 1:
    raise SystemExit("expected signer rotation epoch marker in runner-generated summary")
if summary.get("runtime_signer_previous_rotation_epoch") != 1:
    raise SystemExit("expected signer previous rotation epoch marker in runner-generated summary")
if summary.get("runtime_signer_private_key_env") != "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX":
    raise SystemExit("expected signer private key env marker in runner-generated summary")
checks = summary.get("checks")
if not isinstance(checks, list) or not checks:
    raise SystemExit("expected checks list in runner-generated summary")
runtime_policy_checks = [
    check for check in checks if isinstance(check, dict) and check.get("id") == "runtime_commit_policy"
]
if len(runtime_policy_checks) != 1:
    raise SystemExit("expected exactly one runtime_commit_policy check in runner-generated summary")
runtime_policy_command = runtime_policy_checks[0].get("command")
if not isinstance(runtime_policy_command, str):
    raise SystemExit("expected runtime_commit_policy command string in runner-generated summary")
if "--require-native-payload-evidence" not in runtime_policy_command:
    raise SystemExit("expected native payload evidence marker in runner-generated runtime policy command")
PY

python3 - "$TMP_SUMMARY_SECONDARY" <<'PY'
import json
import pathlib
import sys

summary = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
runtime_commit_command = summary.get("runtime_commit_command")
if not isinstance(runtime_commit_command, str):
    raise SystemExit("expected runtime_commit_command string in secondary runner-generated summary")
if "KAMN_KOLME_LIVE_SIGNER_PROFILE=ops-secondary" not in runtime_commit_command:
    raise SystemExit("expected secondary signer profile marker in secondary runner-generated runtime commit command")
if summary.get("runtime_signer_profile") != "ops-secondary":
    raise SystemExit("expected secondary signer profile marker in secondary runner-generated summary")
if summary.get("runtime_signer_previous_profile") != "ops-secondary":
    raise SystemExit("expected secondary signer previous-profile marker in secondary runner-generated summary")
if summary.get("runtime_signer_private_key_env") != "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY":
    raise SystemExit("expected secondary signer private key env marker in secondary runner-generated summary")
contracts = summary.get("contracts")
if not isinstance(contracts, dict):
    raise SystemExit("expected contracts object in secondary runner-generated summary")
if contracts.get("runtime_signer_profile") != "ops-secondary":
    raise SystemExit("expected contracts secondary signer profile marker in secondary runner-generated summary")
if contracts.get("runtime_signer_private_key_env") != "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY":
    raise SystemExit("expected contracts secondary signer private key env marker in secondary runner-generated summary")
PY

python3 - "$TMP_INTEGRATION_POLICY_OUT" <<'PY'
import json
import pathlib
import sys

policy = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if policy.get("final_decision") != "GO":
    raise SystemExit("expected GO from real-node profile checker for runner-generated dry-run summary")
if policy.get("reason_codes") != []:
    raise SystemExit("expected no reason codes for runner-generated real-node profile summary")
PY

python3 - "$TMP_INTEGRATION_POLICY_OUT_SECONDARY" <<'PY'
import json
import pathlib
import sys

policy = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if policy.get("final_decision") != "GO":
    raise SystemExit("expected GO from real-node profile checker for secondary runner-generated dry-run summary")
if policy.get("reason_codes") != []:
    raise SystemExit("expected no reason codes for secondary runner-generated real-node profile summary")
PY

echo "local KAMN live runtime real-node profile policy checker tests passed."
