#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SMOKE_SCRIPT="$ROOT_DIR/scripts/runtime/run_live_network_smoke_lane.sh"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$SMOKE_SCRIPT" ]; then
  echo "expected live-network smoke lane runner to be executable" >&2
  exit 1
fi

smoke_output="$(
  bash "$SMOKE_SCRIPT" \
    --output-json "$TMP_REPORT"
)"
if ! printf '%s\n' "$smoke_output" | grep -q '^status=pass$'; then
  echo "expected live-network smoke lane to report pass status" >&2
  exit 1
fi
if ! printf '%s\n' "$smoke_output" | grep -q '^final_decision=GO$'; then
  echo "expected live-network smoke lane to report GO decision" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text())
if payload.get("schema_version") != "kamn.runtime.live-network-smoke-report.v1":
    raise SystemExit("unexpected live-network smoke report schema")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected live-network smoke report final decision to be GO")
if payload.get("status") != "pass":
    raise SystemExit("expected live-network smoke report status to be pass")
if payload.get("command_count", 0) < 2:
    raise SystemExit("expected live-network smoke report to record at least two smoke commands")
PY

set +e
budget_failure_output="$(
  KAMN_LIVE_NETWORK_SMOKE_SKIP_COMMANDS=true \
  KAMN_LIVE_NETWORK_SMOKE_FAKE_DELAY_SECONDS=1 \
  KAMN_LIVE_NETWORK_SMOKE_MAX_SECONDS=0 \
  bash "$SMOKE_SCRIPT" \
    --output-json "$TMP_REPORT" 2>&1
)"
budget_failure_code=$?
set -e

if [ "$budget_failure_code" -eq 0 ]; then
  echo "expected live-network smoke lane budget guard to fail closed" >&2
  exit 1
fi
if ! printf '%s\n' "$budget_failure_output" | grep -q "exceeded runtime budget"; then
  echo "expected budget-failure run to emit runtime budget guard message" >&2
  exit 1
fi

echo "live-network smoke lane script tests passed."
