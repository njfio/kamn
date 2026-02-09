#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DEEP_LANE="$ROOT_DIR/scripts/did/run_federated_did_handshake_deep_lane.sh"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$DEEP_LANE" ]; then
  echo "expected federated DID handshake deep lane script to be executable" >&2
  exit 1
fi

output="$(bash "$DEEP_LANE" --output-json "$TMP_REPORT")"
if ! printf '%s\n' "$output" | grep -q "federated DID handshake deep lane tests passed."; then
  echo "expected federated DID handshake deep lane success marker" >&2
  exit 1
fi

if [ ! -s "$TMP_REPORT" ]; then
  echo "expected federated DID handshake deep lane to produce non-empty report" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text())
if report.get("schema_version") != "kamn.did.federated-handshake.partition-replay-matrix.v1":
    raise SystemExit("unexpected federated DID handshake deep report schema version")
if report.get("status") != "pass":
    raise SystemExit("expected federated DID handshake deep report to pass")
PY

echo "federated DID handshake deep lane script tests passed."
