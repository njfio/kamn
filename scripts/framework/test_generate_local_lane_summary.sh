#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
HELPER="$ROOT_DIR/scripts/framework/generate_local_lane_summary.py"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$HELPER" ]; then
  echo "expected local-lane summary helper to be executable" >&2
  exit 1
fi

COMMANDS_FILE="$TMP_DIR/commands.txt"
ARTIFACTS_FILE="$TMP_DIR/artifacts.txt"
CHECKPOINTS_FILE="$TMP_DIR/checkpoints.txt"
COMMAND_OUTPUT="$TMP_DIR/commands-summary.json"
CHECKPOINT_OUTPUT="$TMP_DIR/checkpoints-summary.json"

cat >"$COMMANDS_FILE" <<'EOF'
bash scripts/kolme/run_local_bootstrap_health_checks.sh --mode run --output-json /tmp/bootstrap.json
bash scripts/kolme/run_runtime_commit_adapter_contract_lane.sh
EOF

cat >"$ARTIFACTS_FILE" <<'EOF'
/tmp/bootstrap.json
/tmp/runtime-commit.json
EOF

cat >"$CHECKPOINTS_FILE" <<'EOF'
bootstrap_health_checks	bash scripts/kolme/run_local_bootstrap_health_checks.sh --mode run --output-json /tmp/bootstrap.json	pass
runtime_commit_adapter	bash scripts/kolme/run_runtime_commit_adapter_contract_lane.sh	planned
EOF

python3 "$HELPER" \
  --schema-version "kamn.kolme.local-heavy-validation-summary.v1" \
  --summary-type commands \
  --mode dry-run \
  --status ok \
  --local-only-enforced true \
  --commands-file "$COMMANDS_FILE" \
  --artifacts-file "$ARTIFACTS_FILE" \
  --output-json "$COMMAND_OUTPUT"

python3 - "$COMMAND_OUTPUT" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text())
if report.get("schema_version") != "kamn.kolme.local-heavy-validation-summary.v1":
    raise SystemExit("unexpected schema for commands summary")
if report.get("summary_type") != "commands":
    raise SystemExit("expected summary_type=commands")
if report.get("local_only_enforced") is not True:
    raise SystemExit("expected local_only_enforced=true in commands summary")
commands = report.get("commands")
if not isinstance(commands, list) or len(commands) != 2:
    raise SystemExit("expected two command entries")
artifacts = report.get("artifact_paths")
if not isinstance(artifacts, list) or len(artifacts) != 2:
    raise SystemExit("expected two artifact entries")
PY

python3 "$HELPER" \
  --schema-version "kamn.kolme.local-e2e-integration-summary.v1" \
  --summary-type checkpoints \
  --mode run \
  --status fail \
  --reason-code checkpoint_failed_runtime_commit_adapter \
  --local-only-enforced true \
  --checkpoints-file "$CHECKPOINTS_FILE" \
  --artifacts-file "$ARTIFACTS_FILE" \
  --elapsed-seconds 12 \
  --max-seconds 300 \
  --budget-status pass \
  --output-json "$CHECKPOINT_OUTPUT"

python3 - "$CHECKPOINT_OUTPUT" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text())
if report.get("schema_version") != "kamn.kolme.local-e2e-integration-summary.v1":
    raise SystemExit("unexpected schema for checkpoints summary")
if report.get("summary_type") != "checkpoints":
    raise SystemExit("expected summary_type=checkpoints")
if report.get("status") != "fail":
    raise SystemExit("expected fail status in checkpoints summary")
if report.get("reason_code") != "checkpoint_failed_runtime_commit_adapter":
    raise SystemExit("unexpected reason_code in checkpoints summary")
if report.get("elapsed_seconds") != 12:
    raise SystemExit("unexpected elapsed_seconds in checkpoints summary")
if report.get("max_seconds") != 300:
    raise SystemExit("unexpected max_seconds in checkpoints summary")
if report.get("budget_status") != "pass":
    raise SystemExit("unexpected budget_status in checkpoints summary")
checkpoints = report.get("checkpoints")
if not isinstance(checkpoints, list) or len(checkpoints) != 2:
    raise SystemExit("expected two checkpoint entries")
if checkpoints[0].get("id") != "bootstrap_health_checks":
    raise SystemExit("unexpected first checkpoint id")
PY

echo "local-lane summary helper tests passed."
