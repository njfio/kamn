#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
exec bash "$ROOT_DIR/scripts/framework/run_manifest_lane.sh" --manifest "$ROOT_DIR/scripts/framework/manifests/kolme_local_kolme_fork_checkout_bootstrap_contract_lane.json" --phase contract "$@"
