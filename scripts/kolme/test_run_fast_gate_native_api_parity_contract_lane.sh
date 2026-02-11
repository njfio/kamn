#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/kolme/run_fast_gate_native_api_parity_contract_lane.sh"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/kolme_fast_gate_native_api_parity_contract_lane.json"
CONTRACT_IMPL="$ROOT_DIR/scripts/kolme/contracts/fast_gate_native_api_parity_contract_lane.py"
DOC_FILE="$ROOT_DIR/docs/ci/strategy.md"

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected fast-gate native API parity contract lane to be executable" >&2
  exit 1
fi

if ! grep -q "scripts/framework/run_manifest_lane.sh" "$CONTRACT_LANE"; then
  echo "expected fast-gate native API parity contract lane to dispatch through manifest wrapper" >&2
  exit 1
fi

if [ ! -f "$MANIFEST" ]; then
  echo "expected fast-gate native API parity contract lane manifest to exist" >&2
  exit 1
fi

python3 - "$MANIFEST" <<'PY'
import json
import pathlib
import sys

manifest_path = pathlib.Path(sys.argv[1])
payload = json.loads(manifest_path.read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.contract-lane.manifest.v1":
    raise SystemExit("unexpected fast-gate manifest schema")
if payload.get("lane_id") != "kolme.fast_gate_native_api_parity.contract":
    raise SystemExit("unexpected fast-gate manifest lane_id")
contract_command = payload.get("phases", {}).get("contract")
if contract_command != [
    "python3",
    "scripts/kolme/contracts/fast_gate_native_api_parity_contract_lane.py",
]:
    raise SystemExit("unexpected fast-gate manifest contract command")
PY

if [ ! -f "$CONTRACT_IMPL" ]; then
  echo "expected fast-gate native API parity contract implementation to exist" >&2
  exit 1
fi

if ! grep -q "run_fast_gate_native_api_parity_contract_lane.sh" "$DOC_FILE"; then
  echo "expected CI strategy doc to reference fast-gate native API parity contract lane command" >&2
  exit 1
fi

if ! grep -q "check_fast_gate_native_api_parity_policy.py" "$DOC_FILE"; then
  echo "expected CI strategy doc to reference fast-gate native API parity policy checker command" >&2
  exit 1
fi

if ! grep -q "KAMN_KOLME_FAST_GATE_NATIVE_PARITY_MAX_SECONDS=120" "$DOC_FILE"; then
  echo "expected CI strategy doc to include fast-gate native parity runtime budget marker" >&2
  exit 1
fi

lane_output="$(bash "$CONTRACT_LANE")"
if ! printf '%s\n' "$lane_output" | grep -q "fast-gate native API parity contract lane tests passed."; then
  echo "expected fast-gate native API parity contract lane success marker" >&2
  exit 1
fi

echo "fast-gate native API parity contract lane tests passed."
