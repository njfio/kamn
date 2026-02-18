#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DISPATCH_SCRIPT_NAME="${KAMN_DISPATCH_SCRIPT_NAME:-$(basename "$0")}"

# Wrapper-compatible shim over the full dispatcher implementation.
# Legacy wrappers invoke scripts/kolme/run_contract_lane_dispatch.sh and route through scripts/framework/run_manifest_lane.sh via the impl.
exec env KAMN_DISPATCH_SCRIPT_NAME="$DISPATCH_SCRIPT_NAME" \
  bash "$ROOT_DIR/scripts/kolme/contract_lane_dispatch_impl.sh" "$@"
