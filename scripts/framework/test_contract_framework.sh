#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

python3 "$ROOT_DIR/scripts/framework/test_contract_framework.py"
python3 "$ROOT_DIR/scripts/framework/test_contract_lane_helpers.py"
python3 "$ROOT_DIR/scripts/framework/test_lane_manifest.py"

echo "contract framework unit tests passed."
