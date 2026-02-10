#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

exec bash "$ROOT_DIR/scripts/sdk/run_live_transport_replay_tamper_contract_lane.sh" --mode fast "$@"
