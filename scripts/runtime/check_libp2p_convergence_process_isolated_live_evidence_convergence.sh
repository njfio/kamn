#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

exec python3 "$ROOT_DIR/scripts/runtime/libp2p_convergence_process_isolated_live_contract.py" check-evidence-convergence "$@"
