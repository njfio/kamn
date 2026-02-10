#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/kolme/run_version_compatibility_contract_lane.sh"
DEEP_LANE="$ROOT_DIR/scripts/kolme/run_version_compatibility_replay_deep_lane.sh"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected Kolme version compatibility contract lane script to be executable" >&2
  exit 1
fi

if [ ! -x "$DEEP_LANE" ]; then
  echo "expected Kolme version compatibility deep lane script to be executable" >&2
  exit 1
fi

if ! grep -q "run_runtime_commit_contract_lane.sh" "$CONTRACT_LANE"; then
  echo "expected Kolme version compatibility contract lane to include runtime commit contract lane coverage" >&2
  exit 1
fi

if ! grep -q "run_runtime_commit_replay_contract_lane.sh" "$CONTRACT_LANE"; then
  echo "expected Kolme version compatibility contract lane to include runtime commit replay contract lane coverage" >&2
  exit 1
fi

if ! grep -q "run_nonce_broadcast_parity_contract_lane.sh" "$CONTRACT_LANE"; then
  echo "expected Kolme version compatibility contract lane to include nonce/broadcast parity contract lane coverage" >&2
  exit 1
fi

if ! grep -q "generate_fork_compatibility_evidence.py" "$CONTRACT_LANE"; then
  echo "expected Kolme version compatibility contract lane to include fork compatibility evidence coverage" >&2
  exit 1
fi

if ! grep -q "check_fork_compatibility_policy.py" "$CONTRACT_LANE"; then
  echo "expected Kolme version compatibility contract lane to include fork compatibility policy coverage" >&2
  exit 1
fi

contract_output="$(bash "$CONTRACT_LANE")"
if ! printf '%s\n' "$contract_output" | grep -q "Kolme version compatibility contract lane tests passed."; then
  echo "expected Kolme version compatibility contract lane success marker" >&2
  exit 1
fi

deep_output="$(bash "$DEEP_LANE" --output-json "$TMP_REPORT")"
if ! printf '%s\n' "$deep_output" | grep -q "Kolme version compatibility replay deep lane tests passed."; then
  echo "expected Kolme version compatibility deep lane success marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text())
if payload.get("schema_version") != "kamn.kolme.version-compatibility-replay-report.v1":
    raise SystemExit("unexpected deep replay report schema")
if payload.get("status") != "pass":
    raise SystemExit("expected Kolme replay deep report to pass")
PY

echo "Kolme version compatibility contract lane script tests passed."
