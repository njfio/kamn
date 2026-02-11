#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$ROOT_DIR/scripts/kolme/check_local_e2e_integration_policy.py"
TMP_DIR="$(mktemp -d)"
TMP_REPORT_OK="$TMP_DIR/ok-report.json"
TMP_REPORT_BAD="$TMP_DIR/bad-report.json"
TMP_POLICY_OUT="$TMP_DIR/policy-report.json"
TMP_ERR="$TMP_DIR/error.log"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$CHECKER" ]; then
  echo "expected local e2e integration policy checker to be executable" >&2
  exit 1
fi

cat >"$TMP_REPORT_OK" <<'JSON'
{
  "schema_version": "kamn.kolme.local-e2e-integration-summary.v1",
  "summary_type": "checkpoints",
  "mode": "dry-run",
  "status": "ok",
  "reason_code": "dry_run_no_commands_executed",
  "local_only_enforced": true,
  "elapsed_seconds": 0,
  "max_seconds": 300,
  "budget_status": "pass",
  "checkpoints": [
    {
      "id": "bootstrap_health_checks",
      "command": "bash scripts/kolme/run_local_bootstrap_health_checks.sh --mode run --output-json /tmp/kolme-local-bootstrap-summary.json",
      "status": "planned"
    },
    {
      "id": "fork_checkout_bootstrap_contract",
      "command": "bash scripts/kolme/run_local_kolme_fork_checkout_bootstrap_contract_lane.sh --output-json /tmp/kolme-local-fork-checkout-bootstrap-summary.json --policy-output-json /tmp/kolme-local-fork-checkout-bootstrap-policy.json",
      "status": "planned"
    },
    {
      "id": "runtime_commit_adapter",
      "command": "bash scripts/kolme/run_runtime_commit_adapter_contract_lane.sh",
      "status": "planned"
    },
    {
      "id": "sdk_live_transport_parity",
      "command": "bash scripts/sdk/run_live_transport_parity_contract_lane.sh --languages python,typescript",
      "status": "planned"
    },
    {
      "id": "fork_rust_test_matrix",
      "command": "bash scripts/kolme/run_local_kolme_fork_rust_test_matrix_contract_lane.sh --output-json /tmp/kolme-local-fork-rust-test-matrix-summary.json --policy-output-json /tmp/kolme-local-fork-rust-test-matrix-policy.json",
      "status": "planned"
    },
    {
      "id": "fork_live_api_conformance",
      "command": "bash scripts/kolme/run_local_kolme_live_api_conformance_contract_lane.sh --output-json /tmp/kolme-local-live-api-conformance-summary.json --policy-output-json /tmp/kolme-local-live-api-conformance-policy.json",
      "status": "planned"
    }
  ],
  "artifact_paths": [
    "/tmp/kolme-local-bootstrap-summary.json",
    "/tmp/kolme-local-fork-checkout-bootstrap-summary.json",
    "/tmp/kolme-local-fork-checkout-bootstrap-policy.json",
    "/tmp/kolme-local-fork-rust-test-matrix-summary.json",
    "/tmp/kolme-local-fork-rust-test-matrix-policy.json",
    "/tmp/kolme-local-live-api-conformance-summary.json",
    "/tmp/kolme-local-live-api-conformance-policy.json"
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
if report.get("schema_version") != "kamn.kolme.local-e2e-integration-policy-report.v1":
    raise SystemExit("unexpected local e2e integration policy schema")
if report.get("final_decision") != "GO":
    raise SystemExit("expected final_decision GO for valid local e2e integration report")
if report.get("reason_codes") != []:
    raise SystemExit("expected no reason codes for valid local e2e integration report")
PY

cat >"$TMP_REPORT_BAD" <<'JSON'
{
  "schema_version": "kamn.kolme.local-e2e-integration-summary.v1",
  "summary_type": "checkpoints",
  "mode": "dry-run",
  "status": "ok",
  "reason_code": "dry_run_no_commands_executed",
  "local_only_enforced": true,
  "elapsed_seconds": 0,
  "max_seconds": 300,
  "budget_status": "pass",
  "checkpoints": [
    {
      "id": "bootstrap_health_checks",
      "command": "bash scripts/kolme/run_local_bootstrap_health_checks.sh --mode run --output-json /tmp/kolme-local-bootstrap-summary.json",
      "status": "planned"
    },
    {
      "id": "runtime_commit_adapter",
      "command": "bash scripts/kolme/run_runtime_commit_adapter_contract_lane.sh",
      "status": "planned"
    }
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
  echo "expected policy checker to fail for missing local e2e checkpoints" >&2
  exit 1
fi

if ! grep -q "check_missing:fork_checkout_bootstrap_contract" "$TMP_ERR"; then
  echo "expected missing checkpoint reason marker for policy failure" >&2
  exit 1
fi

echo "local e2e integration policy checker tests passed."
