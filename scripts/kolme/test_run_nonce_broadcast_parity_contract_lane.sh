#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/kolme/run_nonce_broadcast_parity_contract_lane.sh"
MATRIX_RUNNER="$ROOT_DIR/scripts/kolme/run_nonce_broadcast_parity_matrix.py"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected nonce/broadcast parity contract lane script to be executable" >&2
  exit 1
fi

if [ ! -x "$MATRIX_RUNNER" ]; then
  echo "expected nonce/broadcast parity matrix runner to be executable" >&2
  exit 1
fi

if ! grep -q "run_nonce_broadcast_parity_matrix.py" "$CONTRACT_LANE"; then
  echo "expected nonce/broadcast parity contract lane to execute parity matrix runner" >&2
  exit 1
fi

if ! grep -q "check_nonce_broadcast_parity_policy.py" "$CONTRACT_LANE"; then
  echo "expected nonce/broadcast parity contract lane to execute parity policy checker" >&2
  exit 1
fi

contract_output="$(bash "$CONTRACT_LANE")"
if ! printf '%s\n' "$contract_output" | grep -q "Kolme nonce/broadcast parity contract lane tests passed."; then
  echo "expected nonce/broadcast parity contract lane success marker" >&2
  exit 1
fi

python3 "$MATRIX_RUNNER" \
  --max-cases 3 \
  --output-json "$TMP_REPORT" \
  >/dev/null

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text())
if payload.get("schema_version") != "kamn.kolme.nonce-broadcast-parity-matrix-report.v1":
    raise SystemExit("unexpected nonce/broadcast parity matrix report schema")
if payload.get("status") != "pass":
    raise SystemExit("expected nonce/broadcast parity matrix report to pass")
PY

echo "nonce/broadcast parity contract lane script tests passed."
