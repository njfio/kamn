#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_local_kolme_fork_profile_preflight_contract_lane.sh"
CHECKER="$ROOT_DIR/scripts/kolme/check_local_kolme_fork_profile_preflight_policy.py"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/kolme_local_fork_profile_preflight_contract_lane.json"
RUN_WRAPPER="$ROOT_DIR/scripts/kolme/run_local_kolme_fork_profile_preflight_lane.sh"
RUN_WRAPPER_IMPL="$ROOT_DIR/scripts/kolme/run_local_kolme_fork_profile_preflight_lane_impl.sh"
RUN_MANIFEST="$ROOT_DIR/scripts/framework/manifests/kolme_local_kolme_fork_profile_preflight_lane.json"
DISPATCHER="$ROOT_DIR/scripts/kolme/run_lane_dispatch.sh"
CONTRACT_IMPL="$ROOT_DIR/scripts/kolme/contracts/local_fork_profile_preflight_contract_lane.py"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
README_FILE="$ROOT_DIR/README.md"
TMP_REPORT="$(mktemp)"
TMP_POLICY_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT" "$TMP_POLICY_REPORT"' EXIT

if [ ! -x "$RUNNER" ]; then
  echo "expected local fork profile preflight contract lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$CHECKER" ]; then
  echo "expected local fork profile preflight policy checker to be executable" >&2
  exit 1
fi

if [ ! -x "$RUN_WRAPPER_IMPL" ]; then
  echo "expected local fork profile preflight implementation runner to be executable" >&2
  exit 1
fi

if [ ! -x "$DISPATCHER" ]; then
  echo "expected local run lane dispatcher to be executable" >&2
  exit 1
fi

if [ ! -L "$RUN_WRAPPER" ]; then
  echo "expected local fork profile preflight runner to be a symlink to shared runtime lane dispatcher" >&2
  exit 1
fi

if [ "$(readlink "$RUN_WRAPPER")" != "run_lane_dispatch.sh" ]; then
  echo "expected local fork profile preflight runner symlink target to be run_lane_dispatch.sh" >&2
  exit 1
fi

if [ ! -f "$RUN_MANIFEST" ]; then
  echo "expected local fork profile preflight run manifest to exist" >&2
  exit 1
fi

python3 - "$RUN_MANIFEST" <<'PY'
import json
import pathlib
import sys

manifest_path = pathlib.Path(sys.argv[1])
payload = json.loads(manifest_path.read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.contract-lane.manifest.v1":
    raise SystemExit("expected local fork profile preflight run manifest schema")
if payload.get("lane_id") != "kolme.local_kolme_fork_profile_preflight.run":
    raise SystemExit("expected local fork profile preflight run manifest lane_id")
run_command = payload.get("phases", {}).get("run")
if run_command != [
    "bash",
    "scripts/kolme/run_local_kolme_fork_profile_preflight_lane_impl.sh",
]:
    raise SystemExit("expected local fork profile preflight run manifest command")
PY

resolved_run_manifest_path="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$RUN_WRAPPER")" --resolve-manifest-path)"
if [ "$resolved_run_manifest_path" != "$RUN_MANIFEST" ]; then
  echo "expected local fork profile preflight wrapper to resolve deterministic run manifest" >&2
  exit 1
fi

if bash "$DISPATCHER" --lane-wrapper run_missing_local_kolme_fork_profile_preflight_lane.sh --resolve-manifest-path >/dev/null 2>&1; then
  echo "expected local run lane dispatcher to fail closed for unknown local fork profile preflight wrapper" >&2
  exit 1
fi

if ! grep -q "scripts/framework/run_manifest_lane.sh" "$RUNNER"; then
  echo "expected local fork profile preflight contract lane to dispatch through manifest wrapper" >&2
  exit 1
fi

if [ ! -f "$MANIFEST" ]; then
  echo "expected local fork profile preflight contract lane manifest to exist" >&2
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
    "scripts/kolme/contracts/local_fork_profile_preflight_contract_lane.py",
]:
    raise SystemExit("expected local fork profile preflight manifest contract command")
PY

if [ ! -f "$CONTRACT_IMPL" ]; then
  echo "expected local fork profile preflight contract implementation to exist" >&2
  exit 1
fi

required_coverage_markers=(
  "run_local_kolme_fork_profile_preflight_lane.sh"
  "check_local_kolme_fork_profile_preflight_policy.py"
  "Regression: #1697"
)
for marker in "${required_coverage_markers[@]}"; do
  if ! grep -q "$marker" "$CONTRACT_IMPL"; then
    echo "expected local fork profile preflight contract implementation to include coverage marker: $marker" >&2
    exit 1
  fi
done

if ! grep -q "check_local_kolme_fork_profile_preflight_policy.py" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local fork profile preflight policy checker" >&2
  exit 1
fi

if ! grep -q "run_local_kolme_fork_profile_preflight_contract_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local fork profile preflight contract lane" >&2
  exit 1
fi

if ! grep -q "check_local_kolme_fork_profile_preflight_policy.py" "$README_FILE"; then
  echo "expected README to reference local fork profile preflight policy checker" >&2
  exit 1
fi

if ! grep -q "run_local_kolme_fork_profile_preflight_contract_lane.sh" "$README_FILE"; then
  echo "expected README to reference local fork profile preflight contract lane" >&2
  exit 1
fi

# Regression: #1697
if ! grep -q "Regression: #1697" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to include local fork profile preflight contract-lane regression marker" >&2
  exit 1
fi

bash "$RUNNER" --output-json "$TMP_REPORT" --policy-output-json "$TMP_POLICY_REPORT" >/dev/null

python3 - "$TMP_REPORT" "$TMP_POLICY_REPORT" <<'PY'
import json
import pathlib
import sys

summary = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
policy = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
if summary.get("schema_version") != "kamn.kolme.local-fork-profile-preflight-summary.v1":
    raise SystemExit("unexpected profile preflight contract-lane summary schema")
if summary.get("status") != "ok":
    raise SystemExit("expected profile preflight contract-lane summary status ok")
if summary.get("reason_code") != "dry_run_no_commands_executed":
    raise SystemExit("expected dry-run reason code in profile preflight contract-lane summary")
if policy.get("schema_version") != "kamn.kolme.local-fork-profile-preflight-policy-report.v1":
    raise SystemExit("unexpected profile preflight contract-lane policy schema")
if policy.get("final_decision") != "GO":
    raise SystemExit("expected profile preflight contract-lane policy final_decision GO")
PY

echo "local fork profile preflight contract lane tests passed."
