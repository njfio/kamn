#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

exec python3 "$ROOT_DIR/scripts/runtime/service_api_serde_payload_parity_live_contract.py" check-policy "$@"
