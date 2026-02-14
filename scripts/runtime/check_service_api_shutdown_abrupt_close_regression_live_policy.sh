#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

exec python3 "$ROOT_DIR/scripts/runtime/service_api_shutdown_abrupt_close_regression_contract.py" check-policy "$@"
