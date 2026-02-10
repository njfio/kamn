#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/sdk/run_live_transport_replay_tamper_contract_lane.sh"
SHARED_SCRIPT="$ROOT_DIR/scripts/sdk/live_transport_replay_tamper_contract_lane_contract.py"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$SCRIPT" ]; then
  echo "expected live transport replay/tamper contract lane script to be executable" >&2
  exit 1
fi

if [ ! -x "$SHARED_SCRIPT" ]; then
  echo "expected shared live transport replay/tamper contract lane implementation to be executable" >&2
  exit 1
fi

if ! grep -q 'live_transport_replay_tamper_contract_lane_contract.py' "$SCRIPT"; then
  echo "expected replay/tamper contract lane wrapper to delegate to shared implementation" >&2
  exit 1
fi

report_file="$TMP_DIR/live-transport-replay-tamper-contract-report.json"
output="$(bash "$SCRIPT" --output-report "$report_file")"

if ! printf '%s\n' "$output" | grep -q 'live transport replay/tamper contract lane tests passed.'; then
  echo "expected success output from live transport replay/tamper contract lane" >&2
  exit 1
fi

if [ ! -f "$report_file" ]; then
  echo "expected replay/tamper contract lane to emit report file" >&2
  exit 1
fi

if ! grep -q '"schema_version": "kamn.sdk.live-transport-replay-tamper-evidence.v1"' "$report_file"; then
  echo "expected replay/tamper contract lane report schema marker" >&2
  exit 1
fi

if ! grep -q '"final_decision": "GO"' "$report_file"; then
  echo "expected GO final decision in replay/tamper contract lane report" >&2
  exit 1
fi

if ! grep -q 'generate_live_transport_replay_tamper_evidence_bundle.sh' "$SHARED_SCRIPT"; then
  echo "expected replay/tamper contract lane implementation to execute evidence generator" >&2
  exit 1
fi

if ! grep -q 'check_live_transport_replay_tamper_policy.sh' "$SHARED_SCRIPT"; then
  echo "expected replay/tamper contract lane implementation to execute policy checker" >&2
  exit 1
fi

if ! grep -q 'docs/foundation/runtime-network.md' "$SHARED_SCRIPT"; then
  echo "expected replay/tamper contract lane implementation to verify runtime-network docs reference" >&2
  exit 1
fi

if ! grep -q 'docs/testing/invariant-and-fuzz-strategy.md' "$SHARED_SCRIPT"; then
  echo "expected replay/tamper contract lane implementation to verify invariant/fuzz strategy docs reference" >&2
  exit 1
fi

echo "live transport replay/tamper contract lane tests passed."
