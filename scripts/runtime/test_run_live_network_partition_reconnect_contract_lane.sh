#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/runtime/run_live_network_partition_reconnect_contract_lane.sh"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected partition/reconnect contract lane script to be executable" >&2
  exit 1
fi

if ! grep -q "select_live_network_partition_reconnect_lane.sh" "$CONTRACT_LANE"; then
  echo "expected partition/reconnect contract lane to use lane selector" >&2
  exit 1
fi

if ! grep -q "check_live_network_partition_reconnect_policy.sh" "$CONTRACT_LANE"; then
  echo "expected partition/reconnect contract lane to enforce policy checker" >&2
  exit 1
fi

smoke_output="$(
  bash "$CONTRACT_LANE" \
    --event-name pull_request \
    --output-json "$TMP_REPORT"
)"
if ! printf '%s\n' "$smoke_output" | grep -q "live-network partition/reconnect contract lane tests passed."; then
  echo "expected partition/reconnect contract lane smoke success marker" >&2
  exit 1
fi
if ! printf '%s\n' "$smoke_output" | grep -q '^lane=smoke$'; then
  echo "expected partition/reconnect contract lane to select smoke lane for pull_request" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("lane") != "smoke":
    raise SystemExit("expected smoke lane report for pull_request")
if payload.get("status") != "pass":
    raise SystemExit("expected smoke lane report status=pass")
PY

deep_output="$(
  bash "$CONTRACT_LANE" \
    --event-name schedule \
    --output-json "$TMP_REPORT"
)"
if ! printf '%s\n' "$deep_output" | grep -q "live-network partition/reconnect contract lane tests passed."; then
  echo "expected partition/reconnect contract lane deep success marker" >&2
  exit 1
fi
if ! printf '%s\n' "$deep_output" | grep -q '^lane=deep$'; then
  echo "expected partition/reconnect contract lane to select deep lane for schedule" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("lane") != "deep":
    raise SystemExit("expected deep lane report for schedule")
if payload.get("cadence") != "scheduled":
    raise SystemExit("expected scheduled cadence in deep lane report")
if payload.get("status") != "pass":
    raise SystemExit("expected deep lane report status=pass")
PY

echo "partition/reconnect contract lane script tests passed."
