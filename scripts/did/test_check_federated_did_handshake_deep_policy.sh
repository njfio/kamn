#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$ROOT_DIR/scripts/did/check_federated_did_handshake_deep_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$CHECKER" ]; then
  echo "expected federated DID handshake deep policy checker to be executable" >&2
  exit 1
fi

go_report="$TMP_DIR/go-summary.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$go_report" <<'JSON'
{
  "schema_version": "kamn.did.federated-handshake.deep-summary.v1",
  "event_name": "schedule",
  "cadence": "scheduled",
  "contract_lane_status": "pass",
  "matrix_status": "pass",
  "matrix_case_count": 5,
  "matrix_failed_count": 0,
  "matrix_report_file": "/tmp/federated-did-handshake-report.json",
  "elapsed_seconds": 9,
  "max_seconds": 180,
  "budget_status": "within",
  "reason_codes": [],
  "final_decision": "GO",
  "policy_status": "pending"
}
JSON

go_output="$(bash "$CHECKER" --report-file "$go_report")"
if ! printf '%s\n' "$go_output" | grep -q "^status=ok$"; then
  echo "expected GO summary to pass deep policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$go_output" | grep -q "^final_decision=GO$"; then
  echo "expected GO summary final decision in deep policy checker output" >&2
  exit 1
fi

tampered_report="$TMP_DIR/tampered-summary.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$tampered_report" <<'JSON'
{
  "schema_version": "kamn.did.federated-handshake.deep-summary.v1",
  "event_name": "schedule",
  "cadence": "scheduled",
  "contract_lane_status": "pass",
  "matrix_status": "fail",
  "matrix_case_count": 5,
  "matrix_failed_count": 2,
  "matrix_report_file": "/tmp/federated-did-handshake-report.json",
  "elapsed_seconds": 14,
  "max_seconds": 180,
  "budget_status": "within",
  "reason_codes": [],
  "final_decision": "GO",
  "policy_status": "pending"
}
JSON

set +e
tampered_output="$(bash "$CHECKER" --report-file "$tampered_report" 2>&1)"
tampered_code=$?
set -e

if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered deep summary to fail deep policy checker" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_output" | grep -q "reason_codes mismatch"; then
  echo "expected reason-code mismatch marker for tampered deep summary" >&2
  exit 1
fi

echo "federated DID handshake deep policy checker tests passed."
