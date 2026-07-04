#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

cargo test -p kamn-sdk --test live_transport_agent -- --test-threads=1
cargo test -p kamn-sdk --test tcp_transport_adapter -- --test-threads=1
bash scripts/sdk/test_run_local_e2e_demo.sh
bash scripts/sdk/test_run_localhost_signed_demo.sh
bash scripts/sdk/test_run_tcp_signed_relay_demo.sh
bash scripts/sdk/test_run_tcp_failover_reconnect_matrix.sh

echo "rust sdk live transport contract lane tests passed."
