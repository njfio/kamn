#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
exec bash "$ROOT_DIR/scripts/ci/run_kolme_manifest_migration_contract_dispatch.sh" --group parity_demo_real_process "$@"
