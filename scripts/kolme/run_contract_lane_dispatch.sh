#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)/scripts/lib/common.sh"
SCRIPT_NAME="$(basename "$0")"
WRAPPER_NAME="$SCRIPT_NAME"
RESOLVE_MANIFEST_ONLY=0
FALLBACK_REASON_TAXONOMY_VERSION="kamn.kolme.dispatch-fallback-reason-taxonomy.v1"
FALLBACK_REASON_CODES_CSV="dispatcher_unknown_wrapper,dispatcher_manifest_missing,dispatcher_phase_unmapped"

usage() {
  cat <<'EOF'
Usage:
  bash scripts/kolme/run_contract_lane_dispatch.sh --lane-wrapper <wrapper-name> [--resolve-manifest-path] [-- <lane-args...>]

Wrapper compatibility mode:
  scripts/kolme/run_<lane>_contract_lane.sh [lane-args...]
EOF
}

if [[ "$SCRIPT_NAME" == "run_contract_lane_dispatch.sh" ]]; then
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

  if [[ -z "$WRAPPER_NAME" || "$WRAPPER_NAME" == "run_contract_lane_dispatch.sh" ]]; then
    echo "--lane-wrapper is required when invoking the dispatcher directly" >&2
    usage
    exit 1
  fi
fi

resolve_manifest_record() {
  local wrapper_name="$1"
  python3 "$KAMN_ROOT/scripts/kolme/resolve_manifest.py" \
    --manifests-dir "$KAMN_ROOT/scripts/framework/manifests" \
    --wrapper-name "$wrapper_name" \
    --required-phase "contract"
}

RESOLVE_OUTPUT="$(resolve_manifest_record "$WRAPPER_NAME" || true)"
RESOLVE_STATUS="$(extract_value "$RESOLVE_OUTPUT" "status")"
if [[ "$RESOLVE_STATUS" != "ok" ]]; then
  RESOLVE_ERROR_CODE="$(extract_value "$RESOLVE_OUTPUT" "error_code")"
  RESOLVE_ERROR_DETAIL="$(extract_value "$RESOLVE_OUTPUT" "error_detail")"
  case "$RESOLVE_ERROR_CODE" in
    unknown_wrapper)
      emit_fallback_error \
        "dispatcher_unknown_wrapper" \
        "${RESOLVE_ERROR_DETAIL:-unknown lane wrapper for dispatch: $WRAPPER_NAME}"
      ;;
    invalid_phase|duplicate_wrapper|required_phase_mismatch|invalid_manifest)
      emit_fallback_error \
        "dispatcher_phase_unmapped" \
        "${RESOLVE_ERROR_DETAIL:-unable to resolve lane phase for wrapper: $WRAPPER_NAME}"
      ;;
    *)
      emit_fallback_error \
        "dispatcher_unknown_wrapper" \
        "manifest resolver failed for wrapper: $WRAPPER_NAME"
      ;;
  esac
  exit 1
fi

MANIFEST_PATH="$(extract_value "$RESOLVE_OUTPUT" "manifest_path")"
PHASE_NAME="$(extract_value "$RESOLVE_OUTPUT" "phase")"

if [[ -z "$MANIFEST_PATH" ]]; then
  emit_fallback_error \
    "dispatcher_manifest_missing" \
    "manifest resolver returned empty manifest path for wrapper: $WRAPPER_NAME"
  exit 1
fi

if [[ ! -f "$MANIFEST_PATH" ]]; then
  emit_fallback_error \
    "dispatcher_manifest_missing" \
    "resolved manifest does not exist: $MANIFEST_PATH"
  exit 1
fi

if [[ "$RESOLVE_MANIFEST_ONLY" -eq 1 ]]; then
  echo "$MANIFEST_PATH"
  exit 0
fi

if [[ "$PHASE_NAME" != "contract" ]]; then
  emit_fallback_error \
    "dispatcher_phase_unmapped" \
    "manifest resolver returned non-contract phase for wrapper: $WRAPPER_NAME"
  exit 1
fi

exec bash "$KAMN_ROOT/scripts/framework/run_manifest_lane.sh" \
  --manifest "$MANIFEST_PATH" \
  --phase "$PHASE_NAME" \
  -- \
  "$@"
