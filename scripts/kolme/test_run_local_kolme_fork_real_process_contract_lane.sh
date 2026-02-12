#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/kolme/run_local_kolme_fork_real_process_contract_lane.sh"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/kolme_local_kolme_fork_real_process_contract_lane.json"
CONTRACT_IMPL="$ROOT_DIR/scripts/kolme/contracts/local_kolme_fork_real_process_contract_lane.py"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
README_FILE="$ROOT_DIR/README.md"

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected local fork real-process contract lane script to be executable" >&2
  exit 1
fi

if ! grep -q "scripts/framework/run_manifest_lane.sh" "$CONTRACT_LANE"; then
  echo "expected local fork real-process contract lane to dispatch through manifest wrapper" >&2
  exit 1
fi

if [ ! -f "$MANIFEST" ]; then
  echo "expected local fork real-process contract lane manifest to exist" >&2
  exit 1
fi

python3 - "$MANIFEST" <<'PY'
import json
import pathlib
import sys

manifest_path = pathlib.Path(sys.argv[1])
payload = json.loads(manifest_path.read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.contract-lane.manifest.v1":
    raise SystemExit("unexpected local fork real-process manifest schema")
if payload.get("lane_id") != "kolme.local_kolme_fork_real_process.contract":
    raise SystemExit("unexpected local fork real-process manifest lane_id")
contract_command = payload.get("phases", {}).get("contract")
if contract_command != [
    "python3",
    "scripts/kolme/contracts/local_kolme_fork_real_process_contract_lane.py",
]:
    raise SystemExit("unexpected local fork real-process manifest contract command")
PY

if [ ! -f "$CONTRACT_IMPL" ]; then
  echo "expected local fork real-process contract implementation to exist" >&2
  exit 1
fi

# Regression: #1975
required_lifecycle_finality_markers=(
  "--lifecycle-runtime-commit-finality-command"
  "--lifecycle-runtime-commit-finality-max-seconds"
  "--lifecycle-runtime-commit-finality-output-file"
  "--integration-bootstrap-max-seconds"
  "--integration-conformance-max-seconds"
  "--integration-runtime-commit-max-seconds"
)
for marker in "${required_lifecycle_finality_markers[@]}"; do
  if ! grep -q -- "$marker" "$CONTRACT_IMPL"; then
    echo "expected local fork real-process contract implementation to include lifecycle pass-through marker: $marker" >&2
    exit 1
  fi
done

if ! grep -q "run_local_kolme_fork_real_process_contract_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local fork real-process contract lane" >&2
  exit 1
fi

if ! grep -q "check_local_kolme_fork_real_process_policy.py" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local fork real-process policy checker" >&2
  exit 1
fi

if ! grep -q "Regression: #1644" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to include local fork real-process regression marker" >&2
  exit 1
fi

if ! grep -q -- "--lifecycle-runtime-commit-finality-command" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to include lifecycle runtime finality pass-through command option" >&2
  exit 1
fi

if ! grep -q "run_local_kolme_fork_real_process_contract_lane.sh" "$README_FILE"; then
  echo "expected README to reference local fork real-process contract lane" >&2
  exit 1
fi

if ! grep -q -- "--lifecycle-runtime-commit-finality-command" "$README_FILE"; then
  echo "expected README to include lifecycle runtime finality pass-through command option" >&2
  exit 1
fi

lane_output="$(
  bash "$CONTRACT_LANE" \
    --mode dry-run \
    --max-seconds 180
)"
if ! printf '%s\n' "$lane_output" | grep -q "local fork real-process wrapper contract lane tests passed."; then
  echo "expected local fork real-process contract lane success marker" >&2
  exit 1
fi

TMP_SUMMARY="$(mktemp)"
TMP_POLICY="$(mktemp)"
TMP_FINALITY_OUTPUT="$(mktemp)"
trap 'rm -f "$TMP_SUMMARY" "$TMP_POLICY" "$TMP_FINALITY_OUTPUT"' EXIT

KAMN_KOLME_LOCAL_HEAVY=1 python3 "$CONTRACT_IMPL" \
  --mode run \
  --max-seconds 180 \
  --lifecycle-runtime-commit-finality-command "printf 'finality=final\n'" \
  --lifecycle-runtime-commit-finality-max-seconds 13 \
  --lifecycle-runtime-commit-finality-output-file "$TMP_FINALITY_OUTPUT" \
  --output-json "$TMP_SUMMARY" \
  --policy-output-json "$TMP_POLICY" >/dev/null

python3 - "$TMP_SUMMARY" "$TMP_FINALITY_OUTPUT" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

summary = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
checks = summary.get("checks", [])
lifecycle_commands = [
    check.get("command", "")
    for check in checks
    if isinstance(check, dict) and check.get("id") == "process_lifecycle_lane"
]
if len(lifecycle_commands) != 1:
    raise SystemExit("expected exactly one process_lifecycle_lane command entry")
command = lifecycle_commands[0]
if "--integration-bootstrap-max-seconds" not in command:
    raise SystemExit("expected lifecycle command to use integration-bootstrap-max-seconds option name")
if "--integration-conformance-max-seconds" not in command:
    raise SystemExit("expected lifecycle command to use integration-conformance-max-seconds option name")
if "--integration-runtime-commit-max-seconds" not in command:
    raise SystemExit("expected lifecycle command to use integration-runtime-commit-max-seconds option name")
if "--integration-runtime-commit-finality-command" not in command:
    raise SystemExit("expected lifecycle command to include runtime finality command pass-through")
if "--integration-runtime-commit-finality-max-seconds 13" not in command:
    raise SystemExit("expected lifecycle command to include runtime finality max seconds pass-through")
finality_output_path = pathlib.Path(sys.argv[2]).resolve()
if f"--integration-runtime-commit-finality-output-file {finality_output_path}" not in command:
    raise SystemExit("expected lifecycle command to include runtime finality output pass-through")
PY

echo "local fork real-process contract lane tests passed."
