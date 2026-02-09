#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/kolme/run_triadic_devnet_smoke_contract_lane.sh"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected triadic devnet smoke contract lane script to be executable" >&2
  exit 1
fi

lane_output="$(bash "$CONTRACT_LANE" --output-json "$TMP_REPORT")"
if ! printf '%s\n' "$lane_output" | grep -q "triadic devnet smoke contract lane tests passed."; then
  echo "expected triadic devnet smoke contract lane success marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text())
if payload.get("schema_version") != "kamn.kolme.triadic-devnet-smoke-validation-report.v1":
    raise SystemExit("unexpected triadic devnet smoke report schema")
if payload.get("final_decision") != "PASS":
    raise SystemExit("expected triadic devnet smoke report to pass")
PY

echo "triadic devnet smoke contract lane script tests passed."
