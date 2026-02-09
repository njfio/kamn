#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_SCRIPT="$ROOT_DIR/scripts/sdk/run_rust_live_transport_contract_lane.sh"
DEEP_SCRIPT="$ROOT_DIR/scripts/sdk/run_rust_live_transport_deep_lane.sh"

if [ ! -x "$FAST_SCRIPT" ]; then
  echo "expected rust sdk live transport fast-lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$DEEP_SCRIPT" ]; then
  echo "expected rust sdk live transport deep-lane runner to be executable" >&2
  exit 1
fi

TMP_OUT="$(mktemp)"
trap 'rm -f "$TMP_OUT"' EXIT

bash "$FAST_SCRIPT" >"$TMP_OUT"
if ! grep -q "rust sdk live transport contract lane tests passed." "$TMP_OUT"; then
  echo "expected rust sdk live transport contract lane success marker" >&2
  exit 1
fi

if ! grep -Fq "performance_live_transport_multi_client_deep_lane -- --ignored" "$DEEP_SCRIPT"; then
  echo "expected deep-lane script to execute ignored live transport stress test" >&2
  exit 1
fi

if ! grep -Fq "tcp_transport_adapter" "$FAST_SCRIPT"; then
  echo "expected rust sdk live transport fast-lane runner to include tcp transport adapter tests" >&2
  exit 1
fi

if ! grep -Fq "test_run_tcp_signed_relay_demo.sh" "$FAST_SCRIPT"; then
  echo "expected rust sdk live transport fast-lane runner to include tcp signed relay demo coverage" >&2
  exit 1
fi

if ! grep -Fq "test_run_tcp_failover_reconnect_matrix.sh" "$FAST_SCRIPT"; then
  echo "expected rust sdk live transport fast-lane runner to include tcp failover reconnect matrix coverage" >&2
  exit 1
fi

if ! grep -Fq "KAMN_TCP_FAILOVER_DEEP_CADENCE" "$DEEP_SCRIPT"; then
  echo "expected deep-lane script to gate tcp failover reconnect matrix by scheduled cadence" >&2
  exit 1
fi

if ! grep -Fq "run_tcp_failover_reconnect_matrix.sh" "$DEEP_SCRIPT"; then
  echo "expected deep-lane script to include tcp failover reconnect matrix execution path" >&2
  exit 1
fi

echo "rust sdk live transport contract lane script tests passed."
