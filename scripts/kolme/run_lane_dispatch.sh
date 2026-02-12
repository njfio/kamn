#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT_NAME="$(basename "$0")"
WRAPPER_NAME="$SCRIPT_NAME"
RESOLVE_MANIFEST_ONLY=0

usage() {
  cat <<'EOF'
Usage:
  bash scripts/kolme/run_lane_dispatch.sh --lane-wrapper <wrapper-name> [--resolve-manifest-path] [-- <lane-args...>]

Wrapper compatibility mode:
  scripts/kolme/run_<lane>.sh [lane-args...]
EOF
}

if [[ "$SCRIPT_NAME" == "run_lane_dispatch.sh" ]]; then
  while (($# > 0)); do
    case "$1" in
      --lane-wrapper)
        if (($# < 2)); then
          echo "missing value for --lane-wrapper" >&2
          usage
          exit 1
        fi
        WRAPPER_NAME="$2"
        shift 2
        ;;
      --resolve-manifest-path)
        RESOLVE_MANIFEST_ONLY=1
        shift
        ;;
      --)
        shift
        break
        ;;
      *)
        echo "unknown dispatcher argument: $1" >&2
        usage
        exit 1
        ;;
    esac
  done

  if [[ -z "$WRAPPER_NAME" || "$WRAPPER_NAME" == "run_lane_dispatch.sh" ]]; then
    echo "--lane-wrapper is required when invoking the dispatcher directly" >&2
    usage
    exit 1
  fi
fi

resolve_manifest_name() {
  case "$1" in
    run_local_runtime_commit_live_lane.sh) echo "kolme_local_runtime_commit_live_lane.json" ;;
    *)
      return 1
      ;;
  esac
}

MANIFEST_FILE="$(resolve_manifest_name "$WRAPPER_NAME" || true)"
if [[ -z "$MANIFEST_FILE" ]]; then
  echo "unknown lane wrapper for dispatch: $WRAPPER_NAME" >&2
  exit 1
fi

MANIFEST_PATH="$ROOT_DIR/scripts/framework/manifests/$MANIFEST_FILE"
if [[ ! -f "$MANIFEST_PATH" ]]; then
  echo "resolved manifest does not exist: $MANIFEST_PATH" >&2
  exit 1
fi

if [[ "$RESOLVE_MANIFEST_ONLY" -eq 1 ]]; then
  echo "$MANIFEST_PATH"
  exit 0
fi

exec bash "$ROOT_DIR/scripts/framework/run_manifest_lane.sh" \
  --manifest "$MANIFEST_PATH" \
  --phase run \
  -- \
  "$@"
