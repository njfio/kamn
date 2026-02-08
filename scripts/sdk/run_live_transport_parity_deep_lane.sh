#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

bash "$ROOT_DIR/scripts/sdk/run_rust_live_transport_deep_lane.sh"
python3 -m unittest tests/python/test_sdk_live_transport_deep.py
node --experimental-strip-types --test ./packages/kamn-sdk/tests/live_transport_client.deep.ts

echo "live transport parity deep lane tests passed."
