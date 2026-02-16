#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$ROOT_DIR/scripts/kolme/check_runtime_commit_replay_policy.py"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT
EXPECTED_RECOVERY_REASON_TAXONOMY_VERSION="kamn.kolme.runtime-commit-recovery-reason-taxonomy.v1"
EXPECTED_RECOVERY_REASON_CODES_CSV="recovery_nonce_not_monotonic,recovery_payload_hash_mismatch,recovery_receipt_not_final,recovery_replay_detected"

extract_value() {
  local output="$1"
  local key="$2"
  printf '%s\n' "$output" | awk -F= -v key="$key" '$1 == key { sub($1 "=",""); print; exit }'
}

assert_eq() {
  local actual="$1"
  local expected="$2"
  local message="$3"
  if [ "$actual" != "$expected" ]; then
    echo "$message: expected '$expected', got '$actual'" >&2
    exit 1
  fi
}

if [ ! -x "$CHECKER" ]; then
  echo "expected runtime commit replay policy checker to be executable" >&2
  exit 1
fi

go_output="$(
  python3 "$CHECKER" \
    --operation-id "op-go-001" \
    --idempotency-key "kolme-runtime-commit:op-go-001:state:agent:1:12" \
    --receipt-provider "kolme-local" \
    --expected-receipt-provider "kolme-local" \
    --receipt-commit-id "kolme-commit:op-go-001:agent:1:12" \
    --expected-receipt-commit-id "kolme-commit:op-go-001:agent:1:12" \
    --nonce-monotonic true \
    --replay-detected false \
    --payload-hash-match true \
    --receipt-finality FINAL \
    --ci-fast-gate PASS \
    --output-json "$TMP_REPORT"
)"
assert_eq "$(extract_value "$go_output" "status")" "ok" "expected GO case to report ok status"
assert_eq "$(extract_value "$go_output" "final_decision")" "GO" "expected GO case to produce GO"
assert_eq "$(extract_value "$go_output" "failed_checks")" "none" "expected GO case to have no failed checks"
assert_eq "$(extract_value "$go_output" "recovery_reason_taxonomy_version")" "$EXPECTED_RECOVERY_REASON_TAXONOMY_VERSION" "expected deterministic recovery taxonomy version for GO case"
assert_eq "$(extract_value "$go_output" "recovery_reason_codes_csv")" "$EXPECTED_RECOVERY_REASON_CODES_CSV" "expected deterministic recovery taxonomy code ordering"
assert_eq "$(extract_value "$go_output" "recovery_reason_codes_value")" "none" "expected deterministic recovery taxonomy value for GO case"
assert_eq "$(extract_value "$go_output" "retransmission_evidence_contract_version")" "v1" "expected deterministic retransmission evidence contract version for GO case"
assert_eq "$(extract_value "$go_output" "nonce_idempotency_contract_version")" "v1" "expected deterministic nonce/idempotency contract version for GO case"

set +e
no_go_output="$(
  python3 "$CHECKER" \
    --operation-id "op-no-go-001" \
    --idempotency-key "kolme-runtime-commit:op-no-go-001:state:agent:2:12" \
    --receipt-provider "kolme-rogue" \
    --expected-receipt-provider "kolme-local" \
    --receipt-commit-id "kolme-commit:op-no-go-001:agent:2:12-tampered" \
    --expected-receipt-commit-id "kolme-commit:op-no-go-001:agent:2:12" \
    --nonce-monotonic false \
    --replay-detected true \
    --payload-hash-match false \
    --receipt-finality PENDING \
    --ci-fast-gate PASS \
    --output-json "$TMP_REPORT" 2>&1
)"
no_go_code=$?
set -e

if [ "$no_go_code" -eq 0 ]; then
  echo "expected replay/tamper case to fail closed" >&2
  exit 1
fi

assert_eq "$(extract_value "$no_go_output" "status")" "fail" "expected NO-GO case to report fail status"
assert_eq "$(extract_value "$no_go_output" "final_decision")" "NO-GO" "expected NO-GO case to produce NO-GO"
if ! printf '%s\n' "$no_go_output" | grep -q "receipt_commit_id_mismatch"; then
  echo "expected NO-GO case to include receipt_commit_id_mismatch reason code" >&2
  exit 1
fi
if ! printf '%s\n' "$no_go_output" | grep -q "replay_detected"; then
  echo "expected NO-GO case to include replay_detected reason code" >&2
  exit 1
fi
assert_eq "$(extract_value "$no_go_output" "recovery_reason_taxonomy_version")" "$EXPECTED_RECOVERY_REASON_TAXONOMY_VERSION" "expected deterministic recovery taxonomy version for NO-GO case"
assert_eq "$(extract_value "$no_go_output" "recovery_reason_codes_csv")" "$EXPECTED_RECOVERY_REASON_CODES_CSV" "expected deterministic recovery taxonomy ordering for NO-GO case"
assert_eq "$(extract_value "$no_go_output" "recovery_reason_codes_value")" "recovery_nonce_not_monotonic,recovery_payload_hash_mismatch,recovery_receipt_not_final,recovery_replay_detected" "expected deterministic recovery taxonomy value for NO-GO case"

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text())
if payload.get("schema_version") != "kamn.kolme.runtime-commit-replay-policy-report.v1":
    raise SystemExit("unexpected runtime commit replay policy report schema")
if payload.get("final_decision") != "NO-GO":
    raise SystemExit("expected persisted NO-GO decision in policy report")
required = {"receipt_provider_mismatch", "receipt_commit_id_mismatch", "replay_detected"}
reasons = set(payload.get("reason_codes", []))
if not required.issubset(reasons):
    raise SystemExit("missing expected runtime commit replay fail reasons in report")
if payload.get("recovery_reason_taxonomy_version") != "kamn.kolme.runtime-commit-recovery-reason-taxonomy.v1":
    raise SystemExit("expected recovery_reason_taxonomy_version in policy report")
if payload.get("recovery_reason_codes_csv") != "recovery_nonce_not_monotonic,recovery_payload_hash_mismatch,recovery_receipt_not_final,recovery_replay_detected":
    raise SystemExit("expected deterministic recovery_reason_codes_csv in policy report")
if payload.get("recovery_reason_codes_value") != "recovery_nonce_not_monotonic,recovery_payload_hash_mismatch,recovery_receipt_not_final,recovery_replay_detected":
    raise SystemExit("expected deterministic recovery_reason_codes_value in policy report")
if payload.get("retransmission_evidence_contract_version") != "v1":
    raise SystemExit("expected retransmission_evidence_contract_version=v1 in policy report")
if payload.get("nonce_idempotency_contract_version") != "v1":
    raise SystemExit("expected nonce_idempotency_contract_version=v1 in policy report")
PY

echo "runtime commit replay policy checker tests passed."
