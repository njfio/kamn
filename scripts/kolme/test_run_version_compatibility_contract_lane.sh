#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/kolme/run_version_compatibility_contract_lane.sh"
DISPATCHER="$ROOT_DIR/scripts/kolme/run_lane_dispatch.sh"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/kolme_version_compatibility_contract_lane.json"
DEEP_LANE_MANIFEST="$ROOT_DIR/scripts/framework/manifests/kolme_version_compatibility_replay_deep_lane.json"
CONTRACT_IMPL="$ROOT_DIR/scripts/kolme/contracts/version_compatibility_contract_lane.py"
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

if [ ! -x "$DISPATCHER" ]; then
  echo "expected Kolme run lane dispatcher script to be executable" >&2
  exit 1
fi

if [ ! -L "$DEEP_LANE" ]; then
  echo "expected Kolme version compatibility deep lane script to be a symlink to shared run lane dispatcher" >&2
  exit 1
fi

if [ "$(readlink "$DEEP_LANE")" != "run_lane_dispatch.sh" ]; then
  echo "expected Kolme version compatibility deep lane script symlink target to be run_lane_dispatch.sh" >&2
  exit 1
fi

if ! grep -q "scripts/framework/run_manifest_lane.sh" "$CONTRACT_LANE"; then
  echo "expected Kolme version compatibility contract lane to dispatch through manifest wrapper" >&2
  exit 1
fi

if [ ! -f "$MANIFEST" ]; then
  echo "expected Kolme version compatibility contract lane manifest to exist" >&2
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
    "scripts/kolme/contracts/version_compatibility_contract_lane.py",
]:
    raise SystemExit("expected version compatibility manifest contract command")
PY

if [ ! -f "$CONTRACT_IMPL" ]; then
  echo "expected Kolme version compatibility contract implementation to exist" >&2
  exit 1
fi

if [ ! -f "$DEEP_LANE_MANIFEST" ]; then
  echo "expected Kolme version compatibility replay deep lane manifest to exist" >&2
  exit 1
fi

python3 - "$DEEP_LANE_MANIFEST" <<'PY'
import json
import pathlib
import sys

manifest_path = pathlib.Path(sys.argv[1])
payload = json.loads(manifest_path.read_text(encoding="utf-8"))
run_command = payload.get("phases", {}).get("run")
if run_command != [
    "bash",
    "scripts/kolme/run_version_compatibility_replay_deep_lane_impl.sh",
]:
    raise SystemExit("expected version compatibility replay deep lane manifest run command")
PY

resolved_deep_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$DEEP_LANE")" --resolve-manifest-path)"
if [ "$resolved_deep_manifest" != "$DEEP_LANE_MANIFEST" ]; then
  echo "expected Kolme version compatibility deep lane wrapper to resolve deterministic manifest path" >&2
  exit 1
fi

required_coverage_markers=(
  "run_runtime_commit_contract_lane.sh"
  "run_runtime_commit_replay_contract_lane.sh"
  "run_nonce_broadcast_parity_contract_lane.sh"
  "run_block_fallback_reconciliation_contract_lane.sh"
  "run_local_runtime_commit_live_lane.sh"
  "check_local_runtime_commit_live_evidence_policy.py"
  "check_kamn_core_live_https_dependency_posture.sh"
  "dry_run_no_commands_executed"
  "ci-fast-gate and ci-tools fast mode"
  "generate_fork_compatibility_evidence.py"
  "check_fork_compatibility_policy.py"
  "kamn.kolme.fork-compatibility-reason-taxonomy.v1"
  "upgrade_rehearsal_bypass_guard_status"
)
for marker in "${required_coverage_markers[@]}"; do
  if ! grep -q "$marker" "$CONTRACT_IMPL"; then
    echo "expected Kolme version compatibility contract implementation to include coverage marker: $marker" >&2
    exit 1
  fi
done

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
