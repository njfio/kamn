#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

usage() {
  cat <<'EOF'
Usage: run_localhost_signed_demo.sh [options]

Options:
  --addr <host:port>            Listener bind/connect address.
  --from <did>                  Sender DID.
  --to <did>                    Listener DID.
  --state-hash <hash>           State hash attached to message envelope.
  --body <text>                 Message body payload.
  --nonce <n>                   Positive integer nonce.
  --timeout-seconds <seconds>   Positive integer listener completion timeout.
  --help                        Show this help output.

Environment defaults:
  KAMN_LOCALHOST_SIGNED_DEMO_ADDR
  KAMN_LOCALHOST_SIGNED_DEMO_FROM
  KAMN_LOCALHOST_SIGNED_DEMO_TO
  KAMN_LOCALHOST_SIGNED_DEMO_STATE_HASH
  KAMN_LOCALHOST_SIGNED_DEMO_BODY
  KAMN_LOCALHOST_SIGNED_DEMO_NONCE
  KAMN_LOCALHOST_SIGNED_DEMO_TIMEOUT_SECONDS
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

validate_agent_did() {
  local label="$1"
  local value="$2"
  if [[ ! "$value" =~ ^kamn:did:agent:[[:alnum:]_.:-]+$ ]]; then
    echo "$label must be a kamn agent DID (kamn:did:agent:<id>)" >&2
    exit 1
  fi
}

ADDR="${KAMN_LOCALHOST_SIGNED_DEMO_ADDR:-127.0.0.1:17879}"
FROM_DID="${KAMN_LOCALHOST_SIGNED_DEMO_FROM:-kamn:did:agent:sender-1}"
TO_DID="${KAMN_LOCALHOST_SIGNED_DEMO_TO:-kamn:did:agent:listener-1}"
STATE_HASH="${KAMN_LOCALHOST_SIGNED_DEMO_STATE_HASH:-state:localhost-demo}"
BODY="${KAMN_LOCALHOST_SIGNED_DEMO_BODY:-hello-from-localhost-demo}"
NONCE="${KAMN_LOCALHOST_SIGNED_DEMO_NONCE:-1}"
TIMEOUT_SECONDS="${KAMN_LOCALHOST_SIGNED_DEMO_TIMEOUT_SECONDS:-15}"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --addr)
      require_arg_value "$1" "${2:-}"
      ADDR="$2"
      shift 2
      ;;
    --from)
      require_arg_value "$1" "${2:-}"
      FROM_DID="$2"
      shift 2
      ;;
    --to)
      require_arg_value "$1" "${2:-}"
      TO_DID="$2"
      shift 2
      ;;
    --state-hash)
      require_arg_value "$1" "${2:-}"
      STATE_HASH="$2"
      shift 2
      ;;
    --body)
      require_arg_value "$1" "${2:-}"
      BODY="$2"
      shift 2
      ;;
    --nonce)
      require_arg_value "$1" "${2:-}"
      NONCE="$2"
      shift 2
      ;;
    --timeout-seconds)
      require_arg_value "$1" "${2:-}"
      TIMEOUT_SECONDS="$2"
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

if [ -z "$ADDR" ] || [[ "$ADDR" != *:* ]]; then
  echo "addr must be in host:port form" >&2
  exit 1
fi

validate_agent_did "from DID" "$FROM_DID"
validate_agent_did "to DID" "$TO_DID"

if ! is_positive_integer "$NONCE"; then
  echo "nonce must be a positive integer" >&2
  exit 1
fi

if ! is_positive_integer "$TIMEOUT_SECONDS"; then
  echo "timeout-seconds must be a positive integer" >&2
  exit 1
fi

TMP_DIR="$(mktemp -d)"
LISTENER_OUT="$TMP_DIR/listener.out"
SENDER_OUT="$TMP_DIR/sender.out"

LISTENER_PID=""

cleanup() {
  if [ -n "$LISTENER_PID" ] && kill -0 "$LISTENER_PID" >/dev/null 2>&1; then
    kill "$LISTENER_PID" >/dev/null 2>&1 || true
  fi
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

wait_for_listener_completion() {
  local pid="$1"
  local timeout_seconds="$2"
  local elapsed=0

  while kill -0 "$pid" >/dev/null 2>&1; do
    if [ "$elapsed" -ge "$timeout_seconds" ]; then
      echo "listener did not complete within ${timeout_seconds}s" >&2
      kill "$pid" >/dev/null 2>&1 || true
      wait "$pid" >/dev/null 2>&1 || true
      return 1
    fi
    sleep 1
    elapsed=$((elapsed + 1))
  done

  wait "$pid"
}

cargo run --quiet -p kamn-sdk --example localhost_signed_listener -- \
  --addr "$ADDR" \
  --expected-from "$FROM_DID" \
  --expected-to "$TO_DID" \
  --state-hash "$STATE_HASH" >"$LISTENER_OUT" 2>&1 &
LISTENER_PID=$!

cargo run --quiet -p kamn-sdk --example localhost_signed_sender -- \
  --addr "$ADDR" \
  --from "$FROM_DID" \
  --to "$TO_DID" \
  --nonce "$NONCE" \
  --state-hash "$STATE_HASH" \
  --body "$BODY" >"$SENDER_OUT"

wait_for_listener_completion "$LISTENER_PID" "$TIMEOUT_SECONDS"

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

echo "localhost signed message demo completed."
