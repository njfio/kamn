#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PYTHON_CHECKER="$ROOT_DIR/scripts/ci/kolme_wrapper_inventory_baseline.py"
SCRIPT_NAME="$(basename "$0")"
WAVE_ID=""

usage() {
  cat >&2 <<'USAGE'
Usage:
  check_non_kolme_wave<id>_wrapper_family_budget_trend.sh [checker-args...]
  check_non_kolme_wave_wrapper_family_budget_trend_impl.sh --wave-id <id> [checker-args...]
USAGE
}

if [[ "$SCRIPT_NAME" =~ ^check_non_kolme_wave([0-9]+)_wrapper_family_budget_trend\.sh$ ]]; then
  WAVE_ID="${BASH_REMATCH[1]}"
fi

while [ "$#" -gt 0 ]; do
  case "$1" in
    --wave-id)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --wave-id" >&2
        usage
        exit 1
      fi
      WAVE_ID="$2"
      shift 2
      ;;
    --)
      shift
      break
      ;;
    *)
      break
      ;;
  esac
done

if [ -z "$WAVE_ID" ]; then
  echo "wave id was not detected; use a wave checker symlink entrypoint or pass --wave-id <id>" >&2
  usage
  exit 1
fi

if [[ ! "$WAVE_ID" =~ ^[0-9]+$ ]]; then
  echo "invalid --wave-id value: $WAVE_ID" >&2
  usage
  exit 1
fi

if [ ! -x "$PYTHON_CHECKER" ]; then
  echo "expected python wrapper baseline checker to be executable: $PYTHON_CHECKER" >&2
  exit 1
fi

THRESHOLD_FILE="$ROOT_DIR/fixtures/ci/non_kolme_wave${WAVE_ID}_wrapper_family_trend_thresholds.json"
if [ ! -f "$THRESHOLD_FILE" ]; then
  echo "expected non-Kolme wave-${WAVE_ID} trend threshold file to exist: $THRESHOLD_FILE" >&2
  exit 1
fi

CHECKER_ARGS=(
  check
  --trend-mode
  --threshold-file "$THRESHOLD_FILE"
)

if [ "$WAVE_ID" = "19" ]; then
  MAX_RUNTIME_SECONDS="${KAMN_NON_KOLME_WAVE19_TREND_MAX_SECONDS:-45}"
  CHECKER_ARGS+=(--max-runtime-seconds "$MAX_RUNTIME_SECONDS")
fi

exec python3 "$PYTHON_CHECKER" "${CHECKER_ARGS[@]}" "$@"
