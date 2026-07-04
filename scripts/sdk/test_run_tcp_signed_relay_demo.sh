#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DEMO_SCRIPT="$ROOT_DIR/scripts/sdk/run_tcp_signed_relay_demo.sh"

if [ ! -x "$DEMO_SCRIPT" ]; then
  echo "expected tcp signed relay demo runner to be executable" >&2
  exit 1
fi

TMP_OUT="$(mktemp)"
trap 'rm -f "$TMP_OUT"' EXIT

if grep -Fq 'KAMN_TCP_RELAY_DEMO_ADDR:-127.0.0.1:17881' "$DEMO_SCRIPT"; then
  echo "expected tcp signed relay demo to allocate a free default loopback port" >&2
  exit 1
fi

if ! grep -Fq 'sock.bind(("127.0.0.1", 0))' "$DEMO_SCRIPT"; then
  echo "expected tcp signed relay demo to ask the OS for a free loopback port" >&2
  exit 1
fi

if ! grep -Fq 'wait_for_listener_ready' "$DEMO_SCRIPT"; then
  echo "expected tcp signed relay demo to wait for listener readiness" >&2
  exit 1
fi

if ! grep -Fq 'status=listening' "$DEMO_SCRIPT"; then
  echo "expected tcp signed relay demo readiness wait to use listener status marker" >&2
  exit 1
fi

if ! bash "$DEMO_SCRIPT" >"$TMP_OUT" 2>&1; then
  cat "$TMP_OUT" >&2 || true
  exit 1
fi

required_markers=(
  "--- sender ---"
  "--- listener ---"
  "status=ok"
  "verified=true"
  "adapter=tcp"
  "from=kamn:did:agent:sender-1--keyh-"
  "signature=sig:secp256k1:baseline-v2:"
  "tcp signed relay demo completed."
)

for marker in "${required_markers[@]}"; do
  if ! grep -Fq -- "$marker" "$TMP_OUT"; then
    echo "expected tcp signed relay demo output marker '$marker'" >&2
    cat "$TMP_OUT" >&2 || true
    exit 1
  fi
done

echo "tcp signed relay demo script tests passed."
