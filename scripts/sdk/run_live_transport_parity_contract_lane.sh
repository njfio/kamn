#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

bash "$ROOT_DIR/scripts/sdk/run_rust_live_transport_contract_lane.sh"
python3 -m unittest tests/python/test_sdk.py
npm --prefix packages/kamn-sdk test

echo "live transport parity contract lane tests passed."
