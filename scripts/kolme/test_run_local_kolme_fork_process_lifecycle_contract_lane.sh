#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_local_kolme_fork_process_lifecycle_contract_lane.sh"
CHECKER="$ROOT_DIR/scripts/kolme/check_local_kolme_fork_process_lifecycle_policy.py"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/kolme_local_kolme_fork_process_lifecycle_contract_lane.json"
CONTRACT_IMPL="$ROOT_DIR/scripts/kolme/contracts/local_kolme_fork_process_lifecycle_contract_lane.py"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
README_FILE="$ROOT_DIR/README.md"
TMP_REPORT="$(mktemp)"
TMP_POLICY_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT" "$TMP_POLICY_REPORT"' EXIT

if [ ! -x "$RUNNER" ]; then
  echo "expected local fork process lifecycle contract lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$CHECKER" ]; then
  echo "expected local fork process lifecycle policy checker to be executable" >&2
  exit 1
fi

if ! grep -q "scripts/framework/run_manifest_lane.sh" "$RUNNER"; then
  echo "expected local fork process lifecycle contract lane to dispatch through manifest wrapper" >&2
  exit 1
fi

# Regression: #1973
required_integration_finality_markers=(
  "--integration-runtime-commit-finality-command"
  "--integration-runtime-commit-finality-max-seconds"
  "--integration-runtime-commit-finality-output-file"
)
for marker in "${required_integration_finality_markers[@]}"; do
  if ! grep -q -- "$marker" "$ROOT_DIR/scripts/kolme/run_local_kolme_fork_process_lifecycle_lane.sh"; then
    echo "expected local fork process lifecycle runner to expose integration finality pass-through marker: $marker" >&2
    exit 1
  fi
done

if [ ! -f "$MANIFEST" ]; then
  echo "expected local fork process lifecycle contract lane manifest to exist" >&2
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
    "scripts/kolme/contracts/local_kolme_fork_process_lifecycle_contract_lane.py",
]:
    raise SystemExit("expected local fork process lifecycle manifest contract command")
PY

if [ ! -f "$CONTRACT_IMPL" ]; then
  echo "expected local fork process lifecycle contract implementation to exist" >&2
  exit 1
fi

required_coverage_markers=(
  "run_local_kolme_fork_process_lifecycle_lane.sh"
  "check_local_kolme_fork_process_lifecycle_policy.py"
  "run_local_kamn_live_runtime_integration_lane.sh"
  "Regression: #1494"
  "Regression: #1973"
)
for marker in "${required_coverage_markers[@]}"; do
  if ! grep -q "$marker" "$CONTRACT_IMPL"; then
    echo "expected local fork process lifecycle contract implementation to include coverage marker: $marker" >&2
    exit 1
  fi
done

if ! grep -q "check_local_kolme_fork_process_lifecycle_policy.py" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local fork process lifecycle policy checker" >&2
  exit 1
fi

if ! grep -q "run_local_kolme_fork_process_lifecycle_contract_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local fork process lifecycle contract lane" >&2
  exit 1
fi

if ! grep -q -- "--integration-runtime-commit-finality-command" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to document process lifecycle integration finality pass-through command option" >&2
  exit 1
fi

if ! grep -q "check_local_kolme_fork_process_lifecycle_policy.py" "$README_FILE"; then
  echo "expected README to reference local fork process lifecycle policy checker" >&2
  exit 1
fi

if ! grep -q "run_local_kolme_fork_process_lifecycle_contract_lane.sh" "$README_FILE"; then
  echo "expected README to reference local fork process lifecycle contract lane" >&2
  exit 1
fi

if ! grep -q -- "--integration-runtime-commit-finality-command" "$README_FILE"; then
  echo "expected README to document process lifecycle integration finality pass-through command option" >&2
  exit 1
fi

# Regression: #1494
if ! grep -q "Regression: #1494" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to include local fork process lifecycle regression marker" >&2
  exit 1
fi

bash "$RUNNER" --output-json "$TMP_REPORT" --policy-output-json "$TMP_POLICY_REPORT" >/dev/null

python3 - "$TMP_REPORT" "$TMP_POLICY_REPORT" <<'PY'
import json
import pathlib
import sys

summary = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
policy = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
if summary.get("schema_version") != "kamn.kolme.local-fork-process-lifecycle-summary.v1":
    raise SystemExit("unexpected local fork process lifecycle contract-lane summary schema")
if summary.get("status") != "ok":
    raise SystemExit("expected local fork process lifecycle contract-lane summary status ok")
if summary.get("reason_code") != "dry_run_no_commands_executed":
    raise SystemExit("expected dry_run_no_commands_executed reason code in contract-lane summary")
if policy.get("schema_version") != "kamn.kolme.local-fork-process-lifecycle-policy-report.v1":
    raise SystemExit("unexpected local fork process lifecycle contract-lane policy schema")
if policy.get("final_decision") != "GO":
    raise SystemExit("expected local fork process lifecycle contract-lane policy final_decision GO")
PY

TMP_DIRECT_SUMMARY="$(mktemp)"
TMP_DIRECT_PROCESS_OUTPUT="$(mktemp)"
TMP_DIRECT_INTEGRATION_REPORT="$(mktemp)"
TMP_DIRECT_FINALITY_OUTPUT="$(mktemp)"
trap 'rm -f "$TMP_REPORT" "$TMP_POLICY_REPORT" "$TMP_DIRECT_SUMMARY" "$TMP_DIRECT_PROCESS_OUTPUT" "$TMP_DIRECT_INTEGRATION_REPORT" "$TMP_DIRECT_FINALITY_OUTPUT"' EXIT

bash "$ROOT_DIR/scripts/kolme/run_local_kolme_fork_process_lifecycle_lane.sh" \
  --mode dry-run \
  --integration-runtime-commit-finality-command "printf 'finality=final\n'" \
  --integration-runtime-commit-finality-max-seconds 11 \
  --integration-runtime-commit-finality-output-file "$TMP_DIRECT_FINALITY_OUTPUT" \
  --process-output-file "$TMP_DIRECT_PROCESS_OUTPUT" \
  --integration-report "$TMP_DIRECT_INTEGRATION_REPORT" \
  --output-json "$TMP_DIRECT_SUMMARY" >/dev/null

python3 - "$TMP_DIRECT_SUMMARY" "$TMP_DIRECT_FINALITY_OUTPUT" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

summary = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
checks = summary.get("checks", [])
integration_commands = [
    check.get("command", "")
    for check in checks
    if isinstance(check, dict) and check.get("id") == "kamn_live_integration"
]
if len(integration_commands) != 1:
    raise SystemExit("expected exactly one kamn_live_integration check command")
integration_command = integration_commands[0]
if "--runtime-commit-finality-command" not in integration_command:
    raise SystemExit("expected nested integration command to include runtime finality command pass-through")
if "--runtime-commit-finality-max-seconds 11" not in integration_command:
    raise SystemExit("expected nested integration command to include runtime finality max seconds pass-through")
finality_output_path = pathlib.Path(sys.argv[2]).resolve()
if f"--runtime-commit-finality-output-file {finality_output_path}" not in integration_command:
    raise SystemExit("expected nested integration command to include runtime finality output pass-through")
if str(finality_output_path) not in summary.get("artifact_paths", []):
    raise SystemExit("expected process lifecycle summary artifact paths to include integration finality output file")
PY

echo "local fork process lifecycle contract lane tests passed."
