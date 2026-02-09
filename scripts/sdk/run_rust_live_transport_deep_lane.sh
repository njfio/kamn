#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

cargo test -p kamn-sdk --test live_transport_agent performance_live_transport_multi_client_deep_lane -- --ignored
cargo test -p kamn-sdk --test live_transport_agent regression_transport_mode_mismatch_is_rejected
cargo test -p kamn-sdk --test live_transport_agent integration_live_transport_clients_share_endpoint_state

if [ "${KAMN_TCP_FAILOVER_DEEP_CADENCE:-}" = "scheduled" ]; then
  bash scripts/sdk/run_tcp_failover_reconnect_matrix.sh \
    --lane deep \
    --output-json "${KAMN_TCP_FAILOVER_DEEP_REPORT:-/tmp/kamn-tcp-failover-reconnect-deep-report.json}"
fi

echo "rust sdk live transport deep lane tests passed."
