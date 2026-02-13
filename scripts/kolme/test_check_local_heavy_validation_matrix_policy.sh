#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$ROOT_DIR/scripts/kolme/check_local_heavy_validation_matrix_policy.py"
TMP_DIR="$(mktemp -d)"
TMP_REPORT_OK="$TMP_DIR/ok-report.json"
TMP_REPORT_BAD="$TMP_DIR/bad-report.json"
TMP_POLICY_OUT="$TMP_DIR/policy-report.json"
TMP_ERR="$TMP_DIR/error.log"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$CHECKER" ]; then
  echo "expected local heavy validation matrix policy checker to be executable" >&2
  exit 1
fi

cat >"$TMP_REPORT_OK" <<'JSON'
{
  "schema_version": "kamn.kolme.local-heavy-validation-summary.v1",
  "summary_type": "commands",
  "mode": "dry-run",
  "status": "ok",
  "reason_code": "dry_run_no_commands_executed",
  "local_only_enforced": true,
  "commands": [
    "bash scripts/kolme/run_local_bootstrap_health_checks.sh --mode run --output-json /tmp/kolme-local-bootstrap-summary.json",
    "bash scripts/kolme/run_version_compatibility_replay_deep_lane.sh --output-json /tmp/kolme-version-compatibility-report.json",
    "bash scripts/kolme/run_local_kolme_fork_rust_test_matrix_contract_lane.sh --output-json /tmp/kolme-local-fork-rust-test-matrix-summary.json --policy-output-json /tmp/kolme-local-fork-rust-test-matrix-policy.json",
    "bash scripts/kolme/run_local_kolme_live_api_conformance_contract_lane.sh --output-json /tmp/kolme-local-live-api-conformance-summary.json --policy-output-json /tmp/kolme-local-live-api-conformance-policy.json",
    "KAMN_KOLME_SIGNATURE_PARITY_MAX_SECONDS=120 bash scripts/kolme/run_signature_parity_contract_lane.sh --output-json /tmp/kolme-signature-parity-matrix-report.json --policy-output-json /tmp/kolme-signature-parity-policy-report.json",
    "bash scripts/kolme/run_local_runtime_commit_live_finality_evidence_contract_lane.sh --output-json /tmp/kolme-local-runtime-commit-live-summary.json --policy-output-json /tmp/kolme-local-runtime-commit-live-policy.json --max-seconds 120 --finality-max-seconds 15 --require-non-synthetic-run-evidence --require-native-payload-evidence",
    "bash scripts/kolme/run_local_native_api_parity_live_proof_contract_lane.sh --output-json /tmp/kolme-local-native-api-parity-live-proof-summary.json --policy-output-json /tmp/kolme-local-native-api-parity-live-proof-policy.json --max-seconds 180",
    "bash scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh --mode dry-run --runtime-profile real-node --runtime-provider-client-contract KolmeRuntimeCommitLiveProvider --max-seconds 210 --runtime-commit-max-seconds 30 --runtime-commit-finality-max-seconds 15 --runtime-commit-live-summary /tmp/kolme-local-runtime-commit-live-summary.json --runtime-commit-live-policy-report /tmp/kolme-local-runtime-commit-live-policy.json --output-json /tmp/kolme-local-kamn-live-runtime-integration-summary.json",
    "python3 scripts/kolme/check_local_kamn_live_runtime_real_node_profile_policy.py --report-file /tmp/kolme-local-kamn-live-runtime-integration-summary.json --expected-final-decision GO --ci-fast-gate PASS --require-reason-code dry_run_no_commands_executed --require-non-synthetic-run-evidence --output-json /tmp/kolme-local-kamn-live-runtime-real-node-policy.json"
  ],
  "artifact_paths": [
    "/tmp/kolme-local-bootstrap-summary.json",
    "/tmp/kolme-version-compatibility-report.json",
    "/tmp/kolme-local-fork-rust-test-matrix-summary.json",
    "/tmp/kolme-local-fork-rust-test-matrix-policy.json",
    "/tmp/kolme-local-live-api-conformance-summary.json",
    "/tmp/kolme-local-live-api-conformance-policy.json",
    "/tmp/kolme-signature-parity-matrix-report.json",
    "/tmp/kolme-signature-parity-policy-report.json",
    "/tmp/kolme-local-runtime-commit-live-summary.json",
    "/tmp/kolme-local-runtime-commit-live-policy.json",
    "/tmp/kolme-local-native-api-parity-live-proof-summary.json",
    "/tmp/kolme-local-native-api-parity-live-proof-policy.json",
    "/tmp/kolme-local-kamn-live-runtime-integration-summary.json",
    "/tmp/kolme-local-kamn-live-runtime-real-node-policy.json"
  ]
}
JSON

python3 "$CHECKER" \
  --report-file "$TMP_REPORT_OK" \
  --expected-final-decision GO \
  --ci-fast-gate PASS \
  --require-reason-code dry_run_no_commands_executed \
  --output-json "$TMP_POLICY_OUT" >/dev/null

python3 - "$TMP_POLICY_OUT" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if report.get("schema_version") != "kamn.kolme.local-heavy-validation-policy-report.v1":
    raise SystemExit("unexpected local heavy validation policy report schema")
if report.get("final_decision") != "GO":
    raise SystemExit("expected final_decision GO for valid local heavy validation report")
if report.get("reason_codes") != []:
    raise SystemExit("expected no failed checks for valid local heavy validation report")
PY

cat >"$TMP_REPORT_BAD" <<'JSON'
{
  "schema_version": "kamn.kolme.local-heavy-validation-summary.v1",
  "summary_type": "commands",
  "mode": "dry-run",
  "status": "ok",
  "reason_code": "dry_run_no_commands_executed",
  "local_only_enforced": true,
  "commands": [
    "bash scripts/kolme/run_local_bootstrap_health_checks.sh --mode run --output-json /tmp/kolme-local-bootstrap-summary.json"
  ],
  "artifact_paths": [
    "/tmp/kolme-local-bootstrap-summary.json"
  ]
}
JSON

set +e
python3 "$CHECKER" \
  --report-file "$TMP_REPORT_BAD" \
  --expected-final-decision GO \
  --ci-fast-gate PASS \
  --require-reason-code dry_run_no_commands_executed \
  --output-json "$TMP_POLICY_OUT" >"$TMP_ERR" 2>&1
bad_exit_code=$?
set -e

if [ "$bad_exit_code" -eq 0 ]; then
  echo "expected policy checker to fail for missing local heavy commands" >&2
  exit 1
fi

if ! grep -q "command_missing:run_version_compatibility_replay_deep_lane.sh" "$TMP_ERR"; then
  echo "expected missing deep replay command marker for policy failure" >&2
  exit 1
fi

if ! grep -q "native_runtime_commit_budget_marker_missing" "$TMP_ERR"; then
  echo "expected missing native runtime commit budget marker reason for policy failure" >&2
  exit 1
fi

echo "local heavy validation matrix policy checker tests passed."
