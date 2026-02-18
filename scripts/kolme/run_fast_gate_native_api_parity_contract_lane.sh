#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
exec bash "$ROOT_DIR/scripts/framework/run_manifest_lane.sh" --manifest "$ROOT_DIR/scripts/framework/manifests/kolme_fast_gate_native_api_parity_contract_lane.json" --phase contract "$@"
