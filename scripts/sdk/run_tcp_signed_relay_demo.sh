#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

ADDR="${KAMN_TCP_RELAY_DEMO_ADDR:-127.0.0.1:17881}"
FROM_DID="${KAMN_TCP_RELAY_DEMO_FROM:-kamn:did:agent:sender-1}"
TO_DID="${KAMN_TCP_RELAY_DEMO_TO:-kamn:did:agent:listener-1}"
STATE_HASH="${KAMN_TCP_RELAY_DEMO_STATE_HASH:-state:tcp-relay-demo}"
BODY="${KAMN_TCP_RELAY_DEMO_BODY:-hello-from-tcp-relay-demo}"

TMP_DIR="$(mktemp -d)"
LISTENER_OUT="$TMP_DIR/listener.out"
SENDER_OUT="$TMP_DIR/sender.out"
trap 'rm -rf "$TMP_DIR"' EXIT

cargo run --quiet -p kamn-sdk --example tcp_signed_relay_listener -- \
  --addr "$ADDR" \
  --expected-from "$FROM_DID" \
  --expected-to "$TO_DID" \
  --state-hash "$STATE_HASH" >"$LISTENER_OUT" 2>&1 &
LISTENER_PID=$!

cleanup_listener() {
  if kill -0 "$LISTENER_PID" >/dev/null 2>&1; then
    kill "$LISTENER_PID" >/dev/null 2>&1 || true
  fi
}
trap 'cleanup_listener; rm -rf "$TMP_DIR"' EXIT

cargo run --quiet -p kamn-sdk --example tcp_signed_relay_sender -- \
  --addr "$ADDR" \
  --from "$FROM_DID" \
  --to "$TO_DID" \
  --nonce 1 \
  --state-hash "$STATE_HASH" \
  --body "$BODY" >"$SENDER_OUT"

wait "$LISTENER_PID"

echo "--- sender ---"
cat "$SENDER_OUT"
echo "--- listener ---"
cat "$LISTENER_OUT"

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
