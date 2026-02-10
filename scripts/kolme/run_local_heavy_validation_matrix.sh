#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

MODE="dry-run"
OUTPUT_JSON="/tmp/kolme-local-heavy-validation-summary.json"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --mode)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --mode" >&2
        exit 1
      fi
      MODE="$2"
      shift 2
      ;;
    --output-json)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --output-json" >&2
        exit 1
      fi
      OUTPUT_JSON="$2"
      shift 2
      ;;
    --help|-h)
      cat <<'USAGE'
Usage: run_local_heavy_validation_matrix.sh [--mode dry-run|run] [--output-json <path>]

Modes:
  dry-run  Print and record the heavy validation command matrix without executing commands.
  run      Execute heavy local validation commands. Requires KAMN_KOLME_LOCAL_HEAVY=1.
USAGE
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if [ "$MODE" != "dry-run" ] && [ "$MODE" != "run" ]; then
  echo "mode must be one of: dry-run, run" >&2
  exit 1
fi

if [ "$MODE" = "run" ] && [ "${KAMN_KOLME_LOCAL_HEAVY:-0}" != "1" ]; then
  echo "run mode requires explicit local-only opt-in: KAMN_KOLME_LOCAL_HEAVY=1" >&2
  exit 1
fi

VERSION_REPORT="/tmp/kolme-version-report.json"
FORK_REPORT="/tmp/kolme-fork-compatibility-report.json"
FORK_POLICY_REPORT="/tmp/kolme-fork-compatibility-policy-report.json"
DEEP_REPORT="/tmp/kolme-version-compatibility-report.json"
DEVNET_MARKERS="/tmp/triadic-devnet-markers.txt"
DEVNET_REPORT="/tmp/triadic-devnet-report.json"

declare -a COMMANDS=(
  "python3 scripts/kolme/validate_version_compatibility.py --kamn-version 1.1.0 --kolme-release-tag v0.15.2 --ci-fast-gate PASS --output-json $VERSION_REPORT"
  "python3 scripts/kolme/generate_fork_compatibility_evidence.py --upstream-release-tag v0.15.2 --fork-release-tag v0.15.2 --fork-repo njfio/kolme_fork --fork-ref refs/heads/main --ci-fast-gate PASS --output-json $FORK_REPORT"
  "python3 scripts/kolme/check_fork_compatibility_policy.py --report-file $FORK_REPORT --expected-upstream-release-tag v0.15.2 --expected-fork-release-tag v0.15.2 --expected-fork-repo njfio/kolme_fork --expected-final-decision GO --ci-fast-gate PASS --output-json $FORK_POLICY_REPORT"
  "bash scripts/kolme/run_version_compatibility_replay_deep_lane.sh --output-json $DEEP_REPORT"
  "bash scripts/kolme/run_triadic_devnet_smoke.sh --output-file $DEVNET_MARKERS"
  "python3 scripts/kolme/validate_triadic_devnet_smoke.py --fixture fixtures/kolme_compatibility/devnet_smoke_markers.json --marker-file $DEVNET_MARKERS --output-json $DEVNET_REPORT"
)

declare -a ARTIFACTS=(
  "$VERSION_REPORT"
  "$FORK_REPORT"
  "$FORK_POLICY_REPORT"
  "$DEEP_REPORT"
  "$DEVNET_MARKERS"
  "$DEVNET_REPORT"
)

if [ "$MODE" = "run" ]; then
  pushd "$ROOT_DIR" >/dev/null
  for command in "${COMMANDS[@]}"; do
    eval "$command"
  done
  popd >/dev/null
fi

COMMAND_FILE="$(mktemp)"
ARTIFACT_FILE="$(mktemp)"
trap 'rm -f "$COMMAND_FILE" "$ARTIFACT_FILE"' EXIT

for command in "${COMMANDS[@]}"; do
  printf '%s\n' "$command" >>"$COMMAND_FILE"
done

for artifact in "${ARTIFACTS[@]}"; do
  printf '%s\n' "$artifact" >>"$ARTIFACT_FILE"
done

python3 - "$OUTPUT_JSON" "$MODE" "$COMMAND_FILE" "$ARTIFACT_FILE" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

output_path = pathlib.Path(sys.argv[1]).resolve()
mode = sys.argv[2]
commands_path = pathlib.Path(sys.argv[3])
artifacts_path = pathlib.Path(sys.argv[4])

commands = [line.strip() for line in commands_path.read_text(encoding="utf-8").splitlines() if line.strip()]
artifacts = [line.strip() for line in artifacts_path.read_text(encoding="utf-8").splitlines() if line.strip()]

summary = {
    "schema_version": "kamn.kolme.local-heavy-validation-summary.v1",
    "mode": mode,
    "local_only_enforced": True,
    "status": "ok",
    "commands": commands,
    "artifact_paths": artifacts,
}

output_path.parent.mkdir(parents=True, exist_ok=True)
output_path.write_text(json.dumps(summary, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

echo "status=ok"
echo "matrix_mode=$MODE"
echo "local_only_enforced=true"
echo "summary_file=$(realpath "$OUTPUT_JSON")"

