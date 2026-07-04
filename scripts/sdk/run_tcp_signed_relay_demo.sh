#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

choose_loopback_addr() {
  python3 - <<'PY'
import socket

with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
    sock.bind(("127.0.0.1", 0))
    print(f"127.0.0.1:{sock.getsockname()[1]}")
PY
}

ADDR="${KAMN_TCP_RELAY_DEMO_ADDR:-$(choose_loopback_addr)}"
FROM_DID="${KAMN_TCP_RELAY_DEMO_FROM:-}"
TO_DID="${KAMN_TCP_RELAY_DEMO_TO:-kamn:did:agent:listener-1}"
STATE_HASH="${KAMN_TCP_RELAY_DEMO_STATE_HASH:-state:tcp-relay-demo}"
BODY="${KAMN_TCP_RELAY_DEMO_BODY:-hello-from-tcp-relay-demo}"
TIMEOUT_SECONDS="${KAMN_TCP_RELAY_DEMO_TIMEOUT_SECONDS:-60}"

is_positive_integer() {
  [[ "$1" =~ ^[1-9][0-9]*$ ]]
}

if ! is_positive_integer "$TIMEOUT_SECONDS"; then
  echo "KAMN_TCP_RELAY_DEMO_TIMEOUT_SECONDS must be a positive integer" >&2
  exit 1
fi

TMP_DIR="$(mktemp -d)"
LISTENER_OUT="$TMP_DIR/listener.out"
SENDER_OUT="$TMP_DIR/sender.out"
trap 'rm -rf "$TMP_DIR"' EXIT

listener_args=(
  --addr "$ADDR"
  --expected-to "$TO_DID"
  --state-hash "$STATE_HASH"
)
if [[ -n "$FROM_DID" ]]; then
  listener_args+=(--expected-from "$FROM_DID")
fi

cargo run --quiet -p kamn-sdk --example tcp_signed_relay_listener -- \
  "${listener_args[@]}" >"$LISTENER_OUT" 2>&1 &
LISTENER_PID=$!

cleanup_listener() {
  if kill -0 "$LISTENER_PID" >/dev/null 2>&1; then
    kill "$LISTENER_PID" >/dev/null 2>&1 || true
  fi
}
trap 'cleanup_listener; rm -rf "$TMP_DIR"' EXIT

wait_for_listener_ready() {
  elapsed=0
  while true; do
    if grep -Fq "status=listening" "$LISTENER_OUT"; then
      return 0
    fi
    if ! kill -0 "$LISTENER_PID" >/dev/null 2>&1; then
      wait "$LISTENER_PID" >/dev/null 2>&1 || true
      echo "listener exited before accepting connections" >&2
      cat "$LISTENER_OUT" >&2 || true
      return 1
    fi
    if [ "$elapsed" -ge "$TIMEOUT_SECONDS" ]; then
      echo "listener did not become ready within ${TIMEOUT_SECONDS}s" >&2
      cat "$LISTENER_OUT" >&2 || true
      return 1
    fi
    sleep 1
    elapsed=$((elapsed + 1))
  done
}

wait_for_listener_ready

sender_args=(
  --addr "$ADDR"
  --to "$TO_DID"
  --nonce 1
  --state-hash "$STATE_HASH"
  --body "$BODY"
)
if [[ -n "$FROM_DID" ]]; then
  sender_args+=(--from "$FROM_DID")
fi

set +e
cargo run --quiet -p kamn-sdk --example tcp_signed_relay_sender -- \
  "${sender_args[@]}" >"$SENDER_OUT" 2>&1
sender_status=$?
set -e

if [[ "$sender_status" -ne 0 ]]; then
  cleanup_listener
  wait "$LISTENER_PID" >/dev/null 2>&1 || true
  echo "--- sender ---"
  cat "$SENDER_OUT"
  echo "--- listener ---"
  cat "$LISTENER_OUT"
  exit "$sender_status"
fi

set +e
wait "$LISTENER_PID"
listener_status=$?
set -e

echo "--- sender ---"
cat "$SENDER_OUT"
echo "--- listener ---"
cat "$LISTENER_OUT"

if [[ "$listener_status" -ne 0 ]]; then
  exit "$listener_status"
fi

if ! grep -Fq "status=ok" "$SENDER_OUT"; then
  echo "expected sender status=ok" >&2
  exit 1
fi
if ! grep -Fq "status=ok" "$LISTENER_OUT"; then
  echo "expected listener status=ok" >&2
  exit 1
fi
if ! grep -Fq "verified=true" "$LISTENER_OUT"; then
  echo "expected listener verified=true marker" >&2
  exit 1
fi
if ! grep -Fq "adapter=tcp" "$SENDER_OUT"; then
  echo "expected sender adapter=tcp marker" >&2
  exit 1
fi
if ! grep -Fq "adapter=tcp" "$LISTENER_OUT"; then
  echo "expected listener adapter=tcp marker" >&2
  exit 1
fi

echo "tcp signed relay demo completed."
