#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DEEP_LANE="$ROOT_DIR/scripts/runtime/run_live_network_partition_reconnect_deep_lane.sh"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/runtime_live_network_partition_reconnect_deep_lane.json"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$DEEP_LANE" ]; then
  echo "expected partition/reconnect deep lane script to be executable" >&2
  exit 1
fi

if [ ! -L "$DEEP_LANE" ]; then
  echo "expected partition/reconnect deep lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi

if [ "$(readlink "$DEEP_LANE")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected partition/reconnect deep lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$DEEP_LANE")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected partition/reconnect deep lane wrapper to resolve runtime deep manifest via dispatcher" >&2
  exit 1
fi

if ! grep -q '"run-deep"' "$MANIFEST_FILE"; then
  echo "expected partition/reconnect deep manifest to dispatch python deep runner entrypoint" >&2
  exit 1
fi

lane_output="$(bash "$DEEP_LANE" --event-name schedule --output-json "$TMP_REPORT")"
if ! printf '%s\n' "$lane_output" | grep -q "live-network partition/reconnect deep lane tests passed."; then
  echo "expected partition/reconnect deep lane success marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.live-network-partition-reconnect-matrix-report.v1":
    raise SystemExit("unexpected partition/reconnect deep report schema")
if payload.get("lane") != "deep":
    raise SystemExit("expected deep lane report")
if payload.get("cadence") != "scheduled":
    raise SystemExit("expected scheduled cadence for deep lane")
if payload.get("status") != "pass":
    raise SystemExit("expected deep lane status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected deep lane final_decision=GO")
if payload.get("scenario_count") != 6:
    raise SystemExit("expected deep lane scenario count to include stress scenarios")
PY

set +e
invalid_event_output="$(
  bash "$DEEP_LANE" --event-name pull_request --output-json "$TMP_REPORT" 2>&1
)"
invalid_event_code=$?
set -e

if [ "$invalid_event_code" -eq 0 ]; then
  echo "expected partition/reconnect deep lane to reject non-scheduled cadence events" >&2
  exit 1
fi

if ! printf '%s\n' "$invalid_event_output" | grep -q "scheduled/manual-only cadence policy"; then
  echo "expected partition/reconnect deep lane cadence policy rejection marker" >&2
  exit 1
fi

echo "partition/reconnect deep lane script tests passed."
