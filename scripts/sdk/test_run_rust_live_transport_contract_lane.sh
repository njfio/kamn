#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_SCRIPT="$ROOT_DIR/scripts/sdk/run_rust_live_transport_contract_lane.sh"
DEEP_SCRIPT="$ROOT_DIR/scripts/sdk/run_rust_live_transport_deep_lane.sh"
SHARED_CONTRACT="$ROOT_DIR/scripts/sdk/rust_live_transport_contract_lane_contract.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/sdk_rust_live_transport_contract_lane.json"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"

if [ ! -x "$FAST_SCRIPT" ]; then
  echo "expected rust sdk live transport fast-lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$DEEP_SCRIPT" ]; then
  echo "expected rust sdk live transport deep-lane runner to be executable" >&2
  exit 1
fi
if [ ! -x "$SHARED_CONTRACT" ]; then
  echo "expected rust sdk live transport shared contract-lane module to be executable" >&2
  exit 1
fi

TMP_OUT="$(mktemp)"
trap 'rm -f "$TMP_OUT"' EXIT

bash "$FAST_SCRIPT" >"$TMP_OUT"
if ! grep -q "rust sdk live transport contract lane tests passed." "$TMP_OUT"; then
  echo "expected rust sdk live transport contract lane success marker" >&2
  exit 1
fi

if [ ! -L "$FAST_SCRIPT" ]; then
  echo "expected rust sdk live transport contract lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi

if [ "$(readlink "$FAST_SCRIPT")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected rust sdk live transport contract lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$FAST_SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected rust sdk live transport wrapper to resolve sdk manifest via dispatcher" >&2
  exit 1
fi

if ! grep -Fq "rust_live_transport_contract_lane_contract.sh" "$MANIFEST_FILE"; then
  echo "expected rust sdk live transport manifest to dispatch shared contract module" >&2
  exit 1
fi

if ! grep -Fq "performance_live_transport_multi_client_deep_lane -- --ignored" "$DEEP_SCRIPT"; then
  echo "expected deep-lane script to execute ignored live transport stress test" >&2
  exit 1
fi

if ! grep -Fq "tcp_transport_adapter" "$SHARED_CONTRACT"; then
  echo "expected rust sdk live transport shared contract module to include tcp transport adapter tests" >&2
  exit 1
fi

if ! grep -Fq "test_run_tcp_signed_relay_demo.sh" "$SHARED_CONTRACT"; then
  echo "expected rust sdk live transport shared contract module to include tcp signed relay demo coverage" >&2
  exit 1
fi

if ! grep -Fq "test_run_tcp_failover_reconnect_matrix.sh" "$SHARED_CONTRACT"; then
  echo "expected rust sdk live transport shared contract module to include tcp failover reconnect matrix coverage" >&2
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
