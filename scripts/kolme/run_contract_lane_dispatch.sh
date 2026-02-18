#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DISPATCH_SCRIPT_NAME="${KAMN_DISPATCH_SCRIPT_NAME:-$(basename "$0")}"
# Legacy wrappers call scripts/kolme/run_contract_lane_dispatch.sh.
# Manifest execution remains centralized in scripts/framework/run_manifest_lane.sh.
exec env KAMN_DISPATCH_SCRIPT_NAME="$DISPATCH_SCRIPT_NAME" bash "$ROOT_DIR/scripts/kolme/contract_lane_dispatch_impl.sh" "$@"
