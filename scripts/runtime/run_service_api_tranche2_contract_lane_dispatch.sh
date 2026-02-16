#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT_NAME="$(basename "$0")"
WRAPPER_NAME="$SCRIPT_NAME"
RESOLVE_IMPL_ONLY=0
FALLBACK_OUTPUT_JSON=""
FALLBACK_REPORT_SCHEMA="kamn.runtime.service-api-tranche2-dispatch-fallback-report.v1"
FALLBACK_REASON_TAXONOMY_VERSION="kamn.runtime.service-api-tranche2-dispatch-fallback-reason-taxonomy.v1"
FALLBACK_REASON_CODES_CSV="dispatcher_impl_missing,dispatcher_impl_not_executable,dispatcher_unknown_wrapper"

write_fallback_report() {
  local reason_code="$1"
  local reason_detail="$2"
  if [[ -z "$FALLBACK_OUTPUT_JSON" ]]; then
    return
  fi

  python3 - "$FALLBACK_OUTPUT_JSON" "$reason_code" "$reason_detail" \
    "$FALLBACK_REPORT_SCHEMA" "$FALLBACK_REASON_TAXONOMY_VERSION" \
    "$FALLBACK_REASON_CODES_CSV" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = {
    "schema_version": sys.argv[4],
    "dispatch_status": "fail",
    "fallback_reason_taxonomy_version": sys.argv[5],
    "fallback_reason_codes_csv": sys.argv[6],
    "fallback_reason_code": sys.argv[2],
    "fallback_reason_detail": sys.argv[3],
}
path.parent.mkdir(parents=True, exist_ok=True)
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY
}

emit_fallback_error() {
  local reason_code="$1"
  local reason_detail="$2"
  write_fallback_report "$reason_code" "$reason_detail"
  echo "dispatch_status=fail" >&2
  echo "fallback_reason_taxonomy_version=$FALLBACK_REASON_TAXONOMY_VERSION" >&2
  echo "fallback_reason_codes_csv=$FALLBACK_REASON_CODES_CSV" >&2
  echo "fallback_reason_code=$reason_code" >&2
  echo "fallback_reason_detail=$reason_detail" >&2
}

usage() {
  cat <<'USAGE'
Usage:
  bash scripts/runtime/run_service_api_tranche2_contract_lane_dispatch.sh --lane-wrapper <wrapper-name> [--resolve-impl-path] [--fallback-output-json <path>] [-- <lane-args...>]

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
      --fallback-output-json)
        if (($# < 2)); then
          echo "missing value for --fallback-output-json" >&2
          usage
          exit 1
        fi
        FALLBACK_OUTPUT_JSON="$2"
        shift 2
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
  emit_fallback_error \
    "dispatcher_unknown_wrapper" \
    "unknown service api tranche-2 wrapper for dispatch: $WRAPPER_NAME"
  exit 1
fi

IMPL_PATH="$ROOT_DIR/scripts/runtime/$IMPL_NAME"
if [[ ! -f "$IMPL_PATH" ]]; then
  emit_fallback_error \
    "dispatcher_impl_missing" \
    "resolved service api tranche-2 implementation does not exist: $IMPL_PATH"
  exit 1
fi
if [[ ! -x "$IMPL_PATH" ]]; then
  emit_fallback_error \
    "dispatcher_impl_not_executable" \
    "resolved service api tranche-2 implementation is not executable: $IMPL_PATH"
  exit 1
fi

if [[ "$RESOLVE_IMPL_ONLY" -eq 1 ]]; then
  echo "$IMPL_PATH"
  exit 0
fi

exec bash "$IMPL_PATH" "$@"
