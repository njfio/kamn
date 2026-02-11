#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
MANIFEST_RUNNER="$ROOT_DIR/scripts/framework/run_manifest_lane.sh"

if [ ! -f "$MANIFEST_RUNNER" ]; then
  echo "expected manifest wrapper runner to exist" >&2
  exit 1
fi

lane_wrapper_shell_loc() {
  local lane_script_path="$1"
  if [ -L "$lane_script_path" ]; then
    echo 1
    return
  fi

  wc -l <"$lane_script_path"
}

lane_scripts=(
  "run_fast_gate_native_api_parity_contract_lane.sh"
  "run_local_native_api_parity_live_proof_contract_lane.sh"
  "run_local_signed_to_kolme_demo_contract_lane.sh"
  "run_local_kolme_fork_checkout_bootstrap_contract_lane.sh"
  "run_local_kolme_fork_real_process_contract_lane.sh"
)
manifest_files=(
  "scripts/framework/manifests/kolme_fast_gate_native_api_parity_contract_lane.json"
  "scripts/framework/manifests/kolme_local_native_api_parity_live_proof_contract_lane.json"
  "scripts/framework/manifests/kolme_local_signed_to_kolme_demo_contract_lane.json"
  "scripts/framework/manifests/kolme_local_kolme_fork_checkout_bootstrap_contract_lane.json"
  "scripts/framework/manifests/kolme_local_kolme_fork_real_process_contract_lane.json"
)
lane_ids=(
  "kolme.fast_gate_native_api_parity.contract"
  "kolme.local_native_api_parity_live_proof.contract"
  "kolme.local_signed_to_kolme_demo.contract"
  "kolme.local_kolme_fork_checkout_bootstrap.contract"
  "kolme.local_kolme_fork_real_process.contract"
)
contract_scripts=(
  "scripts/kolme/contracts/fast_gate_native_api_parity_contract_lane.py"
  "scripts/kolme/contracts/local_native_api_parity_live_proof_contract_lane.py"
  "scripts/kolme/contracts/local_signed_to_kolme_demo_contract_lane.py"
  "scripts/kolme/contracts/local_kolme_fork_checkout_bootstrap_contract_lane.py"
  "scripts/kolme/contracts/local_kolme_fork_real_process_contract_lane.py"
)

total_shell_loc=0
for i in "${!lane_scripts[@]}"; do
  lane_script_path="$ROOT_DIR/scripts/kolme/${lane_scripts[$i]}"
  manifest_path="$ROOT_DIR/${manifest_files[$i]}"
  expected_lane_id="${lane_ids[$i]}"
  expected_contract_script="${contract_scripts[$i]}"

  if [ ! -x "$lane_script_path" ]; then
    echo "expected migrated lane script to be executable: ${lane_scripts[$i]}" >&2
    exit 1
  fi

  if ! grep -q "scripts/framework/run_manifest_lane.sh" "$lane_script_path"; then
    echo "expected migrated lane script to dispatch through manifest wrapper: ${lane_scripts[$i]}" >&2
    exit 1
  fi

  if [ ! -f "$manifest_path" ]; then
    echo "expected manifest file for migrated lane: ${manifest_files[$i]}" >&2
    exit 1
  fi

  if ! python3 - "$manifest_path" "$expected_lane_id" "$expected_contract_script" <<'PY'
import json
import pathlib
import sys

manifest_path = pathlib.Path(sys.argv[1])
expected_lane_id = sys.argv[2]
expected_contract_script = sys.argv[3]

payload = json.loads(manifest_path.read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.contract-lane.manifest.v1":
    raise SystemExit("unexpected manifest schema version")
if payload.get("lane_id") != expected_lane_id:
    raise SystemExit("unexpected lane_id for migrated lane manifest")
phases = payload.get("phases")
if not isinstance(phases, dict) or "contract" not in phases:
    raise SystemExit("manifest missing contract phase")
command = phases["contract"]
if not isinstance(command, list) or len(command) < 2:
    raise SystemExit("manifest contract phase command must be a non-empty list")
if command[0] != "python3" or command[1] != expected_contract_script:
    raise SystemExit("manifest contract phase must invoke expected python contract lane script")
PY
  then
    echo "expected manifest metadata to match migration contract for ${lane_scripts[$i]}" >&2
    exit 1
  fi

  lane_loc="$(lane_wrapper_shell_loc "$lane_script_path")"
  total_shell_loc="$((total_shell_loc + lane_loc))"
done

if [ "$total_shell_loc" -gt 200 ]; then
  echo "expected parity+demo+real-process migrated shell LOC to stay at or below 200 lines, got ${total_shell_loc}" >&2
  exit 1
fi

echo "Kolme parity+demo+real-process manifest migration contract lane checks passed."
