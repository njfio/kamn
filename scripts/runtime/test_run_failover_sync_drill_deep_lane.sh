#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DEEP_LANE="$ROOT_DIR/scripts/runtime/run_failover_sync_drill_deep_lane.sh"
DEEP_LANE_IMPL="$ROOT_DIR/scripts/runtime/run_failover_sync_drill_deep_lane_impl.sh"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/runtime_failover_sync_drill_deep_lane.json"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$DEEP_LANE" ]; then
  echo "expected failover/sync deep lane script to be executable" >&2
  exit 1
fi

if [ ! -x "$DEEP_LANE_IMPL" ]; then
  echo "expected failover/sync deep lane implementation script to be executable" >&2
  exit 1
fi

if [ ! -L "$DEEP_LANE" ]; then
  echo "expected failover/sync deep lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi

if [ "$(readlink "$DEEP_LANE")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected failover/sync deep lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$DEEP_LANE")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected failover/sync deep lane wrapper to resolve runtime deep manifest via dispatcher" >&2
  exit 1
fi

if ! grep -Fq "run_failover_sync_drill_deep_lane_impl.sh" "$MANIFEST_FILE"; then
  echo "expected failover/sync deep manifest to dispatch implementation module" >&2
  exit 1
fi

if ! grep -q "KAMN_FAILOVER_SYNC_DEEP_CADENCE" "$DEEP_LANE_IMPL"; then
  echo "expected failover/sync deep lane implementation to enforce scheduled cadence policy" >&2
  exit 1
fi

set +e
unscheduled_output="$(
  bash "$DEEP_LANE" --skip-suite --output-json "$TMP_REPORT" 2>&1
)"
unscheduled_code=$?
set -e

if [ "$unscheduled_code" -eq 0 ]; then
  echo "expected deep lane cadence guard to reject unscheduled execution" >&2
  exit 1
fi

if ! printf '%s\n' "$unscheduled_output" | grep -q "scheduled-only"; then
  echo "expected deep lane cadence rejection marker" >&2
  exit 1
fi

scheduled_output="$(
  KAMN_FAILOVER_SYNC_DEEP_CADENCE=scheduled \
    bash "$DEEP_LANE" --skip-suite --output-json "$TMP_REPORT"
)"
if ! printf '%s\n' "$scheduled_output" | grep -q "failover/sync deep lane tests passed."; then
  echo "expected failover/sync deep lane success marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text())
if payload.get("schema_version") != "kamn.runtime.failover-sync-drill-report.v1":
    raise SystemExit("unexpected failover/sync deep report schema")
if payload.get("lane") != "deep":
    raise SystemExit("expected deep lane report")
if payload.get("status") != "pass":
    raise SystemExit("expected deep lane to pass under scheduled cadence")
PY

echo "failover/sync deep lane script tests passed."
