#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SMOKE_RUNNER="$ROOT_DIR/scripts/runtime/run_live_network_smoke_lane.sh"
LIVE_NETWORK_DOC="$ROOT_DIR/docs/planning/live-network-wave.md"
README_FILE="$ROOT_DIR/README.md"
MAKEFILE="$ROOT_DIR/Makefile"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$SMOKE_RUNNER" ]; then
  echo "expected live-network smoke runner script to be executable" >&2
  exit 1
fi

if [ ! -f "$LIVE_NETWORK_DOC" ]; then
  echo "expected live-network wave planning doc to exist" >&2
  exit 1
fi

if [ ! -f "$README_FILE" ]; then
  echo "expected README.md to exist" >&2
  exit 1
fi

if [ ! -f "$MAKEFILE" ]; then
  echo "expected Makefile to exist" >&2
  exit 1
fi

start_epoch="$(date +%s)"

smoke_output="$(
  bash "$SMOKE_RUNNER" \
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
if payload.get("status") != "pass":
    raise SystemExit("expected live-network smoke report status to be pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected live-network smoke report decision to be GO")
if payload.get("command_count", 0) < 2:
    raise SystemExit("expected live-network smoke report to include at least two commands")
PY

set +e
budget_failure_output="$(
  KAMN_LIVE_NETWORK_SMOKE_SKIP_COMMANDS=true \
  KAMN_LIVE_NETWORK_SMOKE_FAKE_DELAY_SECONDS=1 \
  KAMN_LIVE_NETWORK_SMOKE_MAX_SECONDS=0 \
  bash "$SMOKE_RUNNER" \
    --output-json "$TMP_REPORT" 2>&1
)"
budget_failure_code=$?
set -e

if [ "$budget_failure_code" -eq 0 ]; then
  echo "expected live-network smoke budget regression run to fail closed" >&2
  exit 1
fi
if ! printf '%s\n' "$budget_failure_output" | grep -q "exceeded runtime budget"; then
  echo "expected budget regression run to emit runtime budget guard message" >&2
  exit 1
fi

if ! grep -q "run_live_network_smoke_lane.sh" "$LIVE_NETWORK_DOC"; then
  echo "expected live-network planning doc to reference smoke runner command" >&2
  exit 1
fi

if ! grep -q "kamn.runtime.live-network-smoke-report.v1" "$LIVE_NETWORK_DOC"; then
  echo "expected live-network planning doc to reference smoke report schema" >&2
  exit 1
fi

if ! grep -q "make smoke-live-network" "$LIVE_NETWORK_DOC"; then
  echo "expected live-network planning doc to reference Makefile smoke target" >&2
  exit 1
fi

if ! grep -q "make smoke-live-network" "$README_FILE"; then
  echo "expected README to reference make smoke-live-network developer command" >&2
  exit 1
fi

if ! grep -q "smoke-live-network:" "$MAKEFILE"; then
  echo "expected Makefile to define smoke-live-network target" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt 180 ]; then
  echo "live-network smoke contract lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

echo "live-network smoke contract lane tests passed."
