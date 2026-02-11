#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/kolme/run_local_kolme_fork_checkout_bootstrap_contract_lane.sh"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/kolme_local_kolme_fork_checkout_bootstrap_contract_lane.json"
CONTRACT_IMPL="$ROOT_DIR/scripts/kolme/contracts/local_kolme_fork_checkout_bootstrap_contract_lane.py"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
README_FILE="$ROOT_DIR/README.md"

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected local fork checkout bootstrap contract lane script to be executable" >&2
  exit 1
fi

if ! grep -q "scripts/framework/run_manifest_lane.sh" "$CONTRACT_LANE"; then
  echo "expected local fork checkout bootstrap contract lane to dispatch through manifest wrapper" >&2
  exit 1
fi

if [ ! -f "$MANIFEST" ]; then
  echo "expected local fork checkout bootstrap contract lane manifest to exist" >&2
  exit 1
fi

python3 - "$MANIFEST" <<'PY'
import json
import pathlib
import sys

manifest_path = pathlib.Path(sys.argv[1])
payload = json.loads(manifest_path.read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.contract-lane.manifest.v1":
    raise SystemExit("unexpected local fork checkout bootstrap manifest schema")
if payload.get("lane_id") != "kolme.local_kolme_fork_checkout_bootstrap.contract":
    raise SystemExit("unexpected local fork checkout bootstrap manifest lane_id")
contract_command = payload.get("phases", {}).get("contract")
if contract_command != [
    "python3",
    "scripts/kolme/contracts/local_kolme_fork_checkout_bootstrap_contract_lane.py",
]:
    raise SystemExit("unexpected local fork checkout bootstrap manifest contract command")
PY

if [ ! -f "$CONTRACT_IMPL" ]; then
  echo "expected local fork checkout bootstrap contract implementation to exist" >&2
  exit 1
fi

if ! grep -q "run_local_kolme_fork_checkout_bootstrap_contract_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local fork checkout bootstrap contract lane" >&2
  exit 1
fi

if ! grep -q "check_local_kolme_fork_checkout_bootstrap_policy.py" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local fork checkout bootstrap policy checker" >&2
  exit 1
fi

if ! grep -q "Regression: #1663" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to include checkout bootstrap regression marker" >&2
  exit 1
fi

if ! grep -q "run_local_kolme_fork_checkout_bootstrap_contract_lane.sh" "$README_FILE"; then
  echo "expected README to reference local fork checkout bootstrap contract lane" >&2
  exit 1
fi

lane_output="$(
  bash "$CONTRACT_LANE" \
    --mode dry-run \
    --max-seconds 90
)"
if ! printf '%s\n' "$lane_output" | grep -q "local fork checkout bootstrap contract lane tests passed."; then
  echo "expected local fork checkout bootstrap contract lane success marker" >&2
  exit 1
fi

echo "local fork checkout bootstrap contract lane tests passed."
