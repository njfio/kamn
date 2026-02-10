#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SMOKE_SCRIPT="$ROOT_DIR/scripts/sdk/run_live_transport_smoke_parity_lane.sh"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$SMOKE_SCRIPT" ]; then
  echo "expected sdk live transport smoke parity lane runner to be executable" >&2
  exit 1
fi

if ! grep -q 'live_transport_smoke_parity_lane_contract.py' "$SMOKE_SCRIPT"; then
  echo "expected sdk smoke parity lane runner to delegate to shared lane contract implementation" >&2
  exit 1
fi

smoke_output="$(
  KAMN_SDK_SMOKE_PARITY_SKIP_COMMANDS=true \
  bash "$SMOKE_SCRIPT" --output-json "$TMP_REPORT"
)"

if ! printf '%s\n' "$smoke_output" | grep -q '^status=pass$'; then
  echo "expected sdk smoke parity lane to report pass status" >&2
  exit 1
fi

if ! printf '%s\n' "$smoke_output" | grep -q '^final_decision=GO$'; then
  echo "expected sdk smoke parity lane to report GO decision" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text())
if payload.get("schema_version") != "kamn.sdk.live-transport-smoke-parity-report.v1":
    raise SystemExit("unexpected sdk smoke parity report schema")
if payload.get("status") != "pass":
    raise SystemExit("expected sdk smoke parity report status to be pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected sdk smoke parity report decision to be GO")
if payload.get("retry_attempts") != 1:
    raise SystemExit("expected sdk smoke parity report retry_attempts=1 for no-retry pass path")
if payload.get("retry_used") is not False:
    raise SystemExit("expected sdk smoke parity report retry_used=false for no-retry pass path")
if payload.get("reason_codes") != []:
    raise SystemExit("expected sdk smoke parity report reason_codes to be empty on GO path")
PY

set +e
runtime_budget_output="$(
  KAMN_SDK_SMOKE_PARITY_SKIP_COMMANDS=true \
  KAMN_SDK_SMOKE_PARITY_FAKE_DELAY_SECONDS=1 \
  KAMN_SDK_SMOKE_PARITY_MAX_SECONDS=0 \
  bash "$SMOKE_SCRIPT" --output-json "$TMP_REPORT" 2>&1
)"
runtime_budget_code=$?
set -e

if [ "$runtime_budget_code" -eq 0 ]; then
  echo "expected sdk smoke parity runtime budget run to fail closed" >&2
  exit 1
fi

if ! printf '%s\n' "$runtime_budget_output" | grep -q 'runtime_budget_exceeded'; then
  echo "expected sdk smoke parity runtime budget run to emit runtime_budget_exceeded" >&2
  exit 1
fi

set +e
retry_budget_output="$(
  KAMN_SDK_SMOKE_PARITY_SKIP_COMMANDS=true \
  KAMN_SDK_SMOKE_PARITY_FORCE_FAILURE=true \
  KAMN_SDK_SMOKE_PARITY_MAX_RETRIES=1 \
  bash "$SMOKE_SCRIPT" --output-json "$TMP_REPORT" 2>&1
)"
retry_budget_code=$?
set -e

if [ "$retry_budget_code" -eq 0 ]; then
  echo "expected sdk smoke parity retry budget run to fail closed" >&2
  exit 1
fi

if ! printf '%s\n' "$retry_budget_output" | grep -q 'retry_budget_exceeded'; then
  echo "expected sdk smoke parity retry budget run to emit retry_budget_exceeded" >&2
  exit 1
fi

echo "sdk live transport smoke parity lane script tests passed."
