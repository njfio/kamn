#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT_NAME="$(basename "$0")"
WRAPPER_NAME="$SCRIPT_NAME"
RESOLVE_IMPL_ONLY=0

usage() {
  cat <<'USAGE'
Usage:
  bash scripts/runtime/run_service_api_tranche2_contract_lane_dispatch.sh --lane-wrapper <wrapper-name> [--resolve-impl-path] [-- <lane-args...>]

Wrapper compatibility mode:
  scripts/runtime/validate_service_api_<lane>_contract_lane.sh [lane-args...]
USAGE
}

if [[ "$SCRIPT_NAME" == "run_service_api_tranche2_contract_lane_dispatch.sh" ]]; then
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
      --resolve-impl-path)
        RESOLVE_IMPL_ONLY=1
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

  if [[ -z "$WRAPPER_NAME" || "$WRAPPER_NAME" == "run_service_api_tranche2_contract_lane_dispatch.sh" ]]; then
    echo "--lane-wrapper is required when invoking the dispatcher directly" >&2
    usage
    exit 1
  fi
fi

resolve_impl_name() {
  case "$1" in
    validate_service_api_prometheus_metrics_live_contract_lane.sh)
      echo "validate_service_api_prometheus_metrics_live_contract_lane_impl.sh"
      ;;
    validate_service_api_graceful_shutdown_drain_live_contract_lane.sh)
      echo "validate_service_api_graceful_shutdown_drain_live_contract_lane_impl.sh"
      ;;
    validate_service_api_shutdown_abrupt_close_regression_live_contract_lane.sh)
      echo "validate_service_api_shutdown_abrupt_close_regression_live_contract_lane_impl.sh"
      ;;
    validate_service_api_validation_negative_matrix_live_contract_lane.sh)
      echo "validate_service_api_validation_negative_matrix_live_contract_lane_impl.sh"
      ;;
    *)
      return 1
      ;;
  esac
}

IMPL_NAME="$(resolve_impl_name "$WRAPPER_NAME" || true)"
if [[ -z "$IMPL_NAME" ]]; then
  echo "unknown service api tranche-2 wrapper for dispatch: $WRAPPER_NAME" >&2
  exit 1
fi

IMPL_PATH="$ROOT_DIR/scripts/runtime/$IMPL_NAME"
if [[ ! -f "$IMPL_PATH" ]]; then
  echo "resolved service api tranche-2 implementation does not exist: $IMPL_PATH" >&2
  exit 1
fi
if [[ ! -x "$IMPL_PATH" ]]; then
  echo "resolved service api tranche-2 implementation is not executable: $IMPL_PATH" >&2
  exit 1
fi

if [[ "$RESOLVE_IMPL_ONLY" -eq 1 ]]; then
  echo "$IMPL_PATH"
  exit 0
fi

exec bash "$IMPL_PATH" "$@"
