#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

python3 "$ROOT_DIR/scripts/framework/test_contract_framework.py"
python3 "$ROOT_DIR/scripts/framework/test_contract_lane_helpers.py"
python3 "$ROOT_DIR/scripts/framework/test_declarative_policy_checker.py"
python3 "$ROOT_DIR/scripts/framework/test_lane_manifest.py"
python3 "$ROOT_DIR/scripts/framework/test_manifest_wrapper_dispatch.py"
python3 "$ROOT_DIR/scripts/framework/test_pilot_lane_manifests.py"
bash "$ROOT_DIR/scripts/framework/test_declarative_policy_checker_contract.sh"
bash "$ROOT_DIR/scripts/framework/test_lane_registry_generation.sh"
bash "$ROOT_DIR/scripts/framework/test_check_lane_registry_drift.sh"

echo "contract framework tests passed."
