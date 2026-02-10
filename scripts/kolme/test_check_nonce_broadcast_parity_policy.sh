#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$ROOT_DIR/scripts/kolme/check_nonce_broadcast_parity_policy.py"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

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
  echo "expected nonce/broadcast parity policy checker to be executable" >&2
  exit 1
fi

go_output="$(
  python3 "$CHECKER" \
    --case-id "nonce-go-001" \
    --operation "nonce" \
    --http-status 200 \
    --nonce-value 42 \
    --broadcast-accepted false \
    --duplicate-detected false \
    --payload-valid true \
    --authorization-present true \
    --ci-fast-gate PASS \
    --output-json "$TMP_REPORT"
)"
assert_eq "$(extract_value "$go_output" "status")" "ok" "expected GO case to report ok status"
assert_eq "$(extract_value "$go_output" "final_decision")" "GO" "expected GO case to produce GO"
assert_eq "$(extract_value "$go_output" "failed_checks")" "none" "expected GO case to have no failed checks"

set +e
no_go_output="$(
  python3 "$CHECKER" \
    --case-id "broadcast-no-go-unauthorized-001" \
    --operation "broadcast" \
    --http-status 401 \
    --nonce-value 0 \
    --broadcast-accepted false \
    --duplicate-detected false \
    --payload-valid true \
    --authorization-present false \
    --ci-fast-gate PASS \
    --output-json "$TMP_REPORT" 2>&1
)"
no_go_code=$?
set -e

if [ "$no_go_code" -eq 0 ]; then
  echo "expected unauthorized parity case to fail closed" >&2
  exit 1
fi

assert_eq "$(extract_value "$no_go_output" "status")" "fail" "expected NO-GO case to report fail status"
assert_eq "$(extract_value "$no_go_output" "final_decision")" "NO-GO" "expected NO-GO case to produce NO-GO"
if ! printf '%s\n' "$no_go_output" | grep -q "unauthorized_status"; then
  echo "expected NO-GO case to include unauthorized_status reason code" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text())
if payload.get("schema_version") != "kamn.kolme.nonce-broadcast-parity-policy-report.v1":
    raise SystemExit("unexpected nonce/broadcast parity policy report schema")
if payload.get("final_decision") != "NO-GO":
    raise SystemExit("expected persisted NO-GO decision in parity policy report")
reasons = set(payload.get("reason_codes", []))
required = {"unauthorized_status", "authorization_missing"}
if not required.issubset(reasons):
    raise SystemExit("missing expected unauthorized parity fail reasons in report")
PY

echo "nonce/broadcast parity policy checker tests passed."
