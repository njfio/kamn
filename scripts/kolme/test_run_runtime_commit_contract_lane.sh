#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/kolme/run_runtime_commit_contract_lane.sh"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/kolme_runtime_commit_contract_lane.json"
CONTRACT_IMPL="$ROOT_DIR/scripts/kolme/contracts/runtime_commit_contract_lane.py"

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected Kolme runtime commit contract lane script to be executable" >&2
  exit 1
fi

if ! grep -q "scripts/framework/run_manifest_lane.sh" "$CONTRACT_LANE"; then
  echo "expected Kolme runtime commit contract lane to dispatch through manifest wrapper" >&2
  exit 1
fi

if [ ! -f "$MANIFEST" ]; then
  echo "expected Kolme runtime commit contract lane manifest to exist" >&2
  exit 1
fi

python3 - "$MANIFEST" <<'PY'
import json
import pathlib
import sys

manifest_path = pathlib.Path(sys.argv[1])
payload = json.loads(manifest_path.read_text(encoding="utf-8"))
contract_command = payload.get("phases", {}).get("contract")
if contract_command != [
    "python3",
    "scripts/kolme/contracts/runtime_commit_contract_lane.py",
]:
    raise SystemExit("expected runtime commit manifest contract command")
PY

if [ ! -f "$CONTRACT_IMPL" ]; then
  echo "expected Kolme runtime commit contract implementation to exist" >&2
  exit 1
fi

required_coverage_markers=(
  "kolme_runtime_commit_client"
  "kolme_runtime_commit_finality"
  "run_runtime_commit_contract_lane.sh"
)
for marker in "${required_coverage_markers[@]}"; do
  if ! grep -q "$marker" "$CONTRACT_IMPL"; then
    echo "expected Kolme runtime commit contract implementation to include coverage marker: $marker" >&2
    exit 1
  fi
done

if ! grep -q "fixtures/kolme_commit/runtime_commit_request_cases.txt" "$CONTRACT_IMPL"; then
  echo "expected Kolme runtime commit contract implementation to include fixture coverage" >&2
  exit 1
fi

lane_output="$(bash "$CONTRACT_LANE")"
if ! printf '%s\n' "$lane_output" | grep -q "Kolme runtime commit contract lane tests passed."; then
  echo "expected Kolme runtime commit contract lane success marker" >&2
  exit 1
fi

echo "Kolme runtime commit contract lane script tests passed."
