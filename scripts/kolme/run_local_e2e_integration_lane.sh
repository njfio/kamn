#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

MODE="dry-run"
OUTPUT_JSON="/tmp/kolme-local-e2e-integration-summary.json"
MAX_SECONDS=300

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
    --max-seconds)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --max-seconds" >&2
        exit 1
      fi
      MAX_SECONDS="$2"
      shift 2
      ;;
    --help|-h)
      cat <<'USAGE'
Usage: run_local_e2e_integration_lane.sh [--mode dry-run|run] [--output-json <path>] [--max-seconds <seconds>]

Modes:
  dry-run  Emit deterministic E2E checkpoint plan without executing commands.
  run      Execute local-only E2E checkpoints. Requires KAMN_KOLME_LOCAL_HEAVY=1.
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

if ! [[ "$MAX_SECONDS" =~ ^[0-9]+$ ]]; then
  echo "max seconds must be a positive integer" >&2
  exit 1
fi

if [ "$MODE" = "run" ] && [ "${KAMN_KOLME_LOCAL_HEAVY:-0}" != "1" ]; then
  echo "run mode requires explicit local-only opt-in: KAMN_KOLME_LOCAL_HEAVY=1" >&2
  exit 1
fi

BOOTSTRAP_REPORT="/tmp/kolme-local-bootstrap-summary.json"

declare -a CHECKPOINT_IDS=(
  "bootstrap_health_checks"
  "runtime_commit_adapter"
  "sdk_live_transport_parity"
)

declare -a CHECKPOINT_COMMANDS=(
  "bash scripts/kolme/run_local_bootstrap_health_checks.sh --mode run --output-json $BOOTSTRAP_REPORT"
  "bash scripts/kolme/run_runtime_commit_adapter_contract_lane.sh"
  "bash scripts/sdk/run_live_transport_parity_contract_lane.sh --languages python,typescript"
)

declare -a ARTIFACTS=(
  "$BOOTSTRAP_REPORT"
)

CHECKPOINT_FILE="$(mktemp)"
ARTIFACT_FILE="$(mktemp)"
trap 'rm -f "$CHECKPOINT_FILE" "$ARTIFACT_FILE"' EXIT

overall_status="ok"
reason_code=""
already_failed=0
start_epoch="$(date +%s)"

pushd "$ROOT_DIR" >/dev/null
for index in "${!CHECKPOINT_IDS[@]}"; do
  checkpoint_id="${CHECKPOINT_IDS[$index]}"
  checkpoint_command="${CHECKPOINT_COMMANDS[$index]}"

  checkpoint_status="planned"
  if [ "$MODE" = "run" ]; then
    if [ "$already_failed" -eq 1 ]; then
      checkpoint_status="skipped"
    else
      if eval "$checkpoint_command"; then
        checkpoint_status="pass"
      else
        checkpoint_status="fail"
        overall_status="fail"
        reason_code="checkpoint_failed_${checkpoint_id}"
        already_failed=1
      fi
    fi
  fi

  printf '%s\t%s\t%s\n' "$checkpoint_id" "$checkpoint_command" "$checkpoint_status" >>"$CHECKPOINT_FILE"
done
popd >/dev/null

for artifact in "${ARTIFACTS[@]}"; do
  printf '%s\n' "$artifact" >>"$ARTIFACT_FILE"
done

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
budget_status="pass"
if [ "$elapsed_seconds" -gt "$MAX_SECONDS" ]; then
  budget_status="fail"
  if [ "$overall_status" = "ok" ]; then
    overall_status="fail"
    reason_code="runtime_budget_exceeded"
  fi
fi

python3 - "$OUTPUT_JSON" "$MODE" "$overall_status" "$reason_code" "$CHECKPOINT_FILE" "$ARTIFACT_FILE" "$elapsed_seconds" "$MAX_SECONDS" "$budget_status" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

output_path = pathlib.Path(sys.argv[1]).resolve()
mode = sys.argv[2]
status = sys.argv[3]
reason_code = sys.argv[4]
checkpoints_path = pathlib.Path(sys.argv[5])
artifacts_path = pathlib.Path(sys.argv[6])
elapsed_seconds = int(sys.argv[7])
max_seconds = int(sys.argv[8])
budget_status = sys.argv[9]

checkpoints = []
for raw_line in checkpoints_path.read_text(encoding="utf-8").splitlines():
    if not raw_line.strip():
        continue
    parts = raw_line.split("\t")
    if len(parts) != 3:
        continue
    checkpoint_id, command, checkpoint_status = parts
    checkpoints.append(
        {
            "id": checkpoint_id,
            "command": command,
            "status": checkpoint_status,
        }
    )

artifacts = [
    line.strip()
    for line in artifacts_path.read_text(encoding="utf-8").splitlines()
    if line.strip()
]

summary = {
    "schema_version": "kamn.kolme.local-e2e-integration-summary.v1",
    "mode": mode,
    "status": status,
    "reason_code": reason_code,
    "local_only_enforced": True,
    "elapsed_seconds": elapsed_seconds,
    "max_seconds": max_seconds,
    "budget_status": budget_status,
    "checkpoints": checkpoints,
    "artifact_paths": artifacts,
}

output_path.parent.mkdir(parents=True, exist_ok=True)
output_path.write_text(json.dumps(summary, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

echo "status=$overall_status"
echo "lane_mode=$MODE"
echo "local_only_enforced=true"
echo "elapsed_seconds=$elapsed_seconds"
echo "max_seconds=$MAX_SECONDS"
echo "budget_status=$budget_status"
if [ -n "$reason_code" ]; then
  echo "reason_code=$reason_code"
fi
echo "summary_file=$(realpath "$OUTPUT_JSON")"

if [ "$overall_status" != "ok" ]; then
  exit 1
fi
