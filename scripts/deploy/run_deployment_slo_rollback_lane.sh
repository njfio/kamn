#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

exec python3 "$ROOT_DIR/scripts/deploy/deployment_slo_rollback_lane_contract.py" "$@"

echo "deployment slo/rollback lane tests passed."
