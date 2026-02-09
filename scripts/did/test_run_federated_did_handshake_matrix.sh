#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
MATRIX_SCRIPT="$ROOT_DIR/scripts/did/run_federated_did_handshake_matrix.py"
FIXTURE_FILE="$ROOT_DIR/fixtures/federated_did_handshake/partition_replay_cases.json"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$MATRIX_SCRIPT" ]; then
  echo "expected federated DID handshake matrix runner to be executable" >&2
  exit 1
fi

if [ ! -f "$FIXTURE_FILE" ]; then
  echo "expected federated DID handshake fixture file to exist" >&2
  exit 1
fi

python3 "$MATRIX_SCRIPT" \
  --fixture "$FIXTURE_FILE" \
  --output-json "$TMP_REPORT" \
  >/dev/null

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

report_path = pathlib.Path(sys.argv[1])
payload = json.loads(report_path.read_text())

if payload.get("schema_version") != "kamn.did.federated-handshake.partition-replay-matrix.v1":
    raise SystemExit("unexpected federated DID handshake matrix schema version")

cases = payload.get("cases", [])
if not cases:
    raise SystemExit("expected at least one federated DID handshake matrix case")

if not any(case.get("case_id") == "no_go_partition_replay" for case in cases):
    raise SystemExit("expected no_go_partition_replay case in federated DID handshake matrix report")
PY

echo "federated DID handshake matrix tests passed."
