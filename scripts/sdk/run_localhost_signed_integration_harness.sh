#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

usage() {
  cat <<'EOF'
Usage: run_localhost_signed_integration_harness.sh [options]

Options:
  --scenario <success|signature-mismatch|timeout>  Scenario to execute.
  --addr <host:port>                               Localhost listener address.
  --timeout-seconds <seconds>                      Positive integer wait budget.
  --output-json <path>                             Optional report output path.
  --help                                           Show this help output.
EOF
}

require_arg_value() {
  local option="$1"
  local value="${2:-}"
  if [ -z "$value" ]; then
    echo "missing value for $option" >&2
    exit 1
  fi
}

is_positive_integer() {
  [[ "$1" =~ ^[1-9][0-9]*$ ]]
}

SCENARIO="${KAMN_LOCALHOST_SIGNED_INTEGRATION_SCENARIO:-success}"
ADDR="${KAMN_LOCALHOST_SIGNED_INTEGRATION_ADDR:-127.0.0.1:17883}"
TIMEOUT_SECONDS="${KAMN_LOCALHOST_SIGNED_INTEGRATION_TIMEOUT_SECONDS:-5}"
OUTPUT_JSON=""

while [ "$#" -gt 0 ]; do
  case "$1" in
    --scenario)
      require_arg_value "$1" "${2:-}"
      SCENARIO="$2"
      shift 2
      ;;
    --addr)
      require_arg_value "$1" "${2:-}"
      ADDR="$2"
      shift 2
      ;;
    --timeout-seconds)
      require_arg_value "$1" "${2:-}"
      TIMEOUT_SECONDS="$2"
      shift 2
      ;;
    --output-json)
      require_arg_value "$1" "${2:-}"
      OUTPUT_JSON="$2"
      shift 2
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

case "$SCENARIO" in
  success|signature-mismatch|timeout) ;;
  *)
    echo "scenario must be one of: success, signature-mismatch, timeout" >&2
    exit 1
    ;;
esac

if [ -z "$ADDR" ] || [[ "$ADDR" != *:* ]]; then
  echo "addr must be in host:port form" >&2
  exit 1
fi

if ! is_positive_integer "$TIMEOUT_SECONDS"; then
  echo "timeout-seconds must be a positive integer" >&2
  exit 1
fi

FROM_DID="kamn:did:agent:sender-1"
TO_DID="kamn:did:agent:listener-1"
STATE_HASH="state:localhost-demo"
BODY="hello-from-localhost-demo"

START_EPOCH="$(date +%s)"
TMP_DIR="$(mktemp -d)"
LISTENER_OUT="$TMP_DIR/listener.out"
SENDER_OUT="$TMP_DIR/sender.out"
LISTENER_PID=""

cleanup() {
  if [ -n "$LISTENER_PID" ] && kill -0 "$LISTENER_PID" >/dev/null 2>&1; then
    kill "$LISTENER_PID" >/dev/null 2>&1 || true
    wait "$LISTENER_PID" >/dev/null 2>&1 || true
  fi
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

elapsed_seconds() {
  local now
  now="$(date +%s)"
  echo $((now - START_EPOCH))
}

emit_report() {
  local status="$1"
  local reason_code="$2"
  local elapsed
  elapsed="$(elapsed_seconds)"

  echo "status=$status; scenario=$SCENARIO; reason_code=$reason_code; elapsed_seconds=$elapsed;"

  if [ -n "$OUTPUT_JSON" ]; then
    mkdir -p "$(dirname "$OUTPUT_JSON")"
    cat >"$OUTPUT_JSON" <<EOF
{"schema_version":"kamn.sdk.localhost-signed.integration-harness.v1","status":"$status","scenario":"$SCENARIO","reason_code":"$reason_code","addr":"$ADDR","timeout_seconds":$TIMEOUT_SECONDS,"elapsed_seconds":$elapsed}
EOF
  fi
}

fail_with_reason() {
  local reason="$1"
  emit_report "fail" "$reason"
  exit 1
}

start_listener() {
  cargo run --quiet -p kamn-sdk --example localhost_signed_listener -- \
    --addr "$ADDR" \
    --expected-from "$FROM_DID" \
    --expected-to "$TO_DID" \
    --state-hash "$STATE_HASH" >"$LISTENER_OUT" 2>&1 &
  LISTENER_PID=$!
}

wait_for_pid() {
  local pid="$1"
  local timeout="$2"
  local elapsed=0

  while kill -0 "$pid" >/dev/null 2>&1; do
    if [ "$elapsed" -ge "$timeout" ]; then
      return 124
    fi
    sleep 1
    elapsed=$((elapsed + 1))
  done

  if wait "$pid"; then
    return 0
  fi
  return "$?"
}

send_invalid_signature_payload() {
  local host="${ADDR%:*}"
  local port="${ADDR##*:}"
  python3 - "$host" "$port" "$FROM_DID" "$TO_DID" "$STATE_HASH" "$BODY" <<'PY'
import socket
import sys
import time

host = sys.argv[1]
port = int(sys.argv[2])
from_did = sys.argv[3]
to_did = sys.argv[4]
state_hash = sys.argv[5]
body = sys.argv[6]

payload = (
    f"from={from_did}\n"
    f"to={to_did}\n"
    "nonce=1\n"
    f"state_hash={state_hash}\n"
    f"body={body}\n"
    "signature=sig:ed25519:baseline-v1:invalid\n"
)

for _ in range(20):
    try:
        with socket.create_connection((host, port), timeout=1.0) as conn:
            conn.sendall(payload.encode("utf-8"))
            sys.exit(0)
    except OSError:
        time.sleep(0.1)

print("failed to send invalid signature payload after retries", file=sys.stderr)
sys.exit(1)
PY
}

run_success_scenario() {
  start_listener
  cargo run --quiet -p kamn-sdk --example localhost_signed_sender -- \
    --addr "$ADDR" \
    --from "$FROM_DID" \
    --to "$TO_DID" \
    --nonce 1 \
    --state-hash "$STATE_HASH" \
    --body "$BODY" >"$SENDER_OUT" 2>&1 || fail_with_reason "sender_failed"

  set +e
  wait_for_pid "$LISTENER_PID" "$TIMEOUT_SECONDS"
  local listener_status=$?
  set -e

  if [ "$listener_status" -eq 124 ]; then
    fail_with_reason "listener_timeout"
  fi
  if [ "$listener_status" -ne 0 ]; then
    fail_with_reason "listener_failed"
  fi
  if ! grep -Fq "status=ok" "$SENDER_OUT"; then
    fail_with_reason "sender_status_missing"
  fi
  if ! grep -Fq "status=ok" "$LISTENER_OUT"; then
    fail_with_reason "listener_status_missing"
  fi
  if ! grep -Fq "verified=true" "$LISTENER_OUT"; then
    fail_with_reason "listener_verification_missing"
  fi

  emit_report "pass" "none"
}

run_signature_mismatch_scenario() {
  start_listener
  send_invalid_signature_payload || fail_with_reason "payload_send_failed"

  set +e
  wait_for_pid "$LISTENER_PID" "$TIMEOUT_SECONDS"
  local listener_status=$?
  set -e

  if [ "$listener_status" -eq 124 ]; then
    fail_with_reason "listener_timeout"
  fi
  if ! grep -Fq "status=error" "$LISTENER_OUT"; then
    fail_with_reason "listener_error_status_missing"
  fi
  if ! grep -Fq "signature verification failed" "$LISTENER_OUT"; then
    if grep -Fq "status=ok" "$LISTENER_OUT"; then
      fail_with_reason "mismatch_not_detected"
    fi
    fail_with_reason "signature_mismatch_not_reported"
  fi

  emit_report "pass" "signature_mismatch_detected"
}

run_timeout_scenario() {
  start_listener

  set +e
  wait_for_pid "$LISTENER_PID" "$TIMEOUT_SECONDS"
  local listener_status=$?
  set -e

  if [ "$listener_status" -ne 124 ]; then
    fail_with_reason "unexpected_listener_completion"
  fi

  kill "$LISTENER_PID" >/dev/null 2>&1 || true
  wait "$LISTENER_PID" >/dev/null 2>&1 || true

  emit_report "pass" "listener_timeout_detected"
}

case "$SCENARIO" in
  success)
    run_success_scenario
    ;;
  signature-mismatch)
    run_signature_mismatch_scenario
    ;;
  timeout)
    run_timeout_scenario
    ;;
esac
