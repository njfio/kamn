#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/kolme/run_local_signed_to_kolme_demo_contract_lane.sh"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/kolme_local_signed_to_kolme_demo_contract_lane.json"
CONTRACT_IMPL="$ROOT_DIR/scripts/kolme/contracts/local_signed_to_kolme_demo_contract_lane.py"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
README_FILE="$ROOT_DIR/README.md"

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected local signed-to-Kolme demo contract lane script to be executable" >&2
  exit 1
fi

if ! grep -q "scripts/framework/run_manifest_lane.sh" "$CONTRACT_LANE"; then
  echo "expected local signed-to-Kolme demo contract lane to dispatch through manifest wrapper" >&2
  exit 1
fi

if [ ! -f "$MANIFEST" ]; then
  echo "expected local signed-to-Kolme demo contract lane manifest to exist" >&2
  exit 1
fi

python3 - "$MANIFEST" <<'PY'
import json
import pathlib
import sys

manifest_path = pathlib.Path(sys.argv[1])
payload = json.loads(manifest_path.read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.contract-lane.manifest.v1":
    raise SystemExit("unexpected local signed-to-Kolme demo manifest schema")
if payload.get("lane_id") != "kolme.local_signed_to_kolme_demo.contract":
    raise SystemExit("unexpected local signed-to-Kolme demo manifest lane_id")
contract_command = payload.get("phases", {}).get("contract")
if contract_command != [
    "python3",
    "scripts/kolme/contracts/local_signed_to_kolme_demo_contract_lane.py",
]:
    raise SystemExit("unexpected local signed-to-Kolme demo manifest contract command")
PY

if [ ! -f "$CONTRACT_IMPL" ]; then
  echo "expected local signed-to-Kolme demo contract implementation to exist" >&2
  exit 1
fi

if ! grep -q "run_local_signed_to_kolme_demo_contract_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local signed-to-Kolme demo contract lane" >&2
  exit 1
fi

if ! grep -q "check_local_signed_to_kolme_demo_policy.py" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local signed-to-Kolme demo policy checker" >&2
  exit 1
fi

if ! grep -q "Regression: #1640" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to include local signed-to-Kolme demo regression marker" >&2
  exit 1
fi

if ! grep -q "run_local_signed_to_kolme_demo_contract_lane.sh" "$README_FILE"; then
  echo "expected README to reference local signed-to-Kolme demo contract lane" >&2
  exit 1
fi

lane_output="$(
  bash "$CONTRACT_LANE" \
    --mode dry-run \
    --max-seconds 120
)"
if ! printf '%s\n' "$lane_output" | grep -q "unified local signed-to-Kolme demo contract lane tests passed."; then
  echo "expected local signed-to-Kolme demo contract lane success marker" >&2
  exit 1
fi

echo "local signed-to-Kolme demo contract lane tests passed."
