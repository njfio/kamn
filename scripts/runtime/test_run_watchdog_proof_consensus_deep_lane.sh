#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DEEP_LANE="$ROOT_DIR/scripts/runtime/run_watchdog_proof_consensus_deep_lane.sh"
DEEP_LANE_IMPL="$ROOT_DIR/scripts/runtime/run_watchdog_proof_consensus_deep_lane_impl.sh"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/runtime_watchdog_proof_consensus_deep_lane.json"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [[ ! -x "$DEEP_LANE" ]]; then
  echo "expected watchdog proof consensus deep lane script to be executable" >&2
  exit 1
fi

if [[ ! -x "$DEEP_LANE_IMPL" ]]; then
  echo "expected watchdog proof consensus deep lane implementation script to be executable" >&2
  exit 1
fi

if [[ ! -L "$DEEP_LANE" ]]; then
  echo "expected watchdog proof consensus deep lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi

if [[ "$(readlink "$DEEP_LANE")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]]; then
  echo "expected watchdog proof consensus deep lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$DEEP_LANE")" --resolve-manifest-path)"
if [[ "$resolved_manifest" != "$MANIFEST_FILE" ]]; then
  echo "expected watchdog proof consensus deep lane wrapper to resolve runtime deep manifest via dispatcher" >&2
  exit 1
fi

if ! grep -Fq "run_watchdog_proof_consensus_deep_lane_impl.sh" "$MANIFEST_FILE"; then
  echo "expected watchdog proof consensus deep manifest to dispatch implementation module" >&2
  exit 1
fi

if ! grep -Fq "run_watchdog_proof_consensus_contract_lane.sh" "$DEEP_LANE_IMPL"; then
  echo "expected watchdog proof consensus deep implementation to execute contract lane baseline checks first" >&2
  exit 1
fi

report_json="$TMP_DIR/watchdog-proof-consensus-deep-summary.json"
lane_output="$(
  KAMN_WATCHDOG_PROOF_CONSENSUS_DEEP_CADENCE=scheduled \
  bash "$DEEP_LANE" \
    --event-name schedule \
    --skip-contract-tests \
    --max-seconds 120 \
    --output-json "$report_json"
)"

if ! printf '%s\n' "$lane_output" | grep -q "watchdog proof consensus deep lane tests passed."; then
  echo "expected watchdog proof consensus deep lane success output" >&2
  exit 1
fi

python3 - "$report_json" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.watchdog-proof-consensus-deep-summary.v1":
    raise SystemExit("unexpected watchdog proof consensus deep summary schema")
if payload.get("event_name") != "schedule":
    raise SystemExit("expected schedule event in watchdog proof consensus deep summary")
if payload.get("cadence") != "scheduled":
    raise SystemExit("expected scheduled cadence in watchdog proof consensus deep summary")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected GO final decision for watchdog proof consensus deep summary")
if payload.get("budget_status") != "within":
    raise SystemExit("expected runtime budget within threshold for watchdog proof consensus deep summary")
PY

set +e
invalid_event_output="$(
  bash "$DEEP_LANE" \
    --event-name pull_request \
    --output-json "$TMP_DIR/watchdog-proof-consensus-invalid-cadence.json" 2>&1
)"
invalid_event_code=$?
set -e

if [[ "$invalid_event_code" -eq 0 ]]; then
  echo "expected watchdog proof consensus deep lane to reject pull_request cadence" >&2
  exit 1
fi

if ! printf '%s\n' "$invalid_event_output" | grep -q "scheduled/manual-only cadence policy"; then
  echo "expected cadence policy rejection marker for watchdog proof consensus deep lane" >&2
  exit 1
fi

echo "watchdog proof consensus deep lane script tests passed."
