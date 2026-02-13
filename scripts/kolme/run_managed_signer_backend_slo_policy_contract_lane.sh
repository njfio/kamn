#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

python3 "$ROOT_DIR/scripts/kolme/contracts/managed_signer_backend_slo_policy_contract_lane.py" "$@"
