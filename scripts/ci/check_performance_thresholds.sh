#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<USAGE
Usage: $0 --report-json <path> [--profile-file <path>] [--lane <smoke|deep>]
USAGE
}

REPORT_JSON=""
PROFILE_FILE=".ci/performance-targets.env"
LANE="smoke"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --report-json)
      REPORT_JSON="${2:-}"
      shift 2
      ;;
    --profile-file)
      PROFILE_FILE="${2:-.ci/performance-targets.env}"
      shift 2
      ;;
    --lane)
      LANE="${2:-smoke}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if [ -z "$REPORT_JSON" ]; then
  usage >&2
  exit 2
fi

if [ ! -f "$REPORT_JSON" ]; then
  echo "report file not found: $REPORT_JSON" >&2
  exit 2
fi

if [ ! -f "$PROFILE_FILE" ]; then
  echo "profile file not found: $PROFILE_FILE" >&2
  exit 2
fi

# shellcheck disable=SC1090
source "$PROFILE_FILE"

case "$LANE" in
  smoke)
    MAX_P50="$PERF_SMOKE_MAX_LATENCY_P50_MS"
    MAX_P99="$PERF_SMOKE_MAX_LATENCY_P99_MS"
    MIN_THROUGHPUT="$PERF_SMOKE_MIN_THROUGHPUT_TPS"
    MIN_AVAILABILITY="$PERF_SMOKE_MIN_AVAILABILITY_PCT"
    ;;
  deep)
    MAX_P50="$PERF_DEEP_MAX_LATENCY_P50_MS"
    MAX_P99="$PERF_DEEP_MAX_LATENCY_P99_MS"
    MIN_THROUGHPUT="$PERF_DEEP_MIN_THROUGHPUT_TPS"
    MIN_AVAILABILITY="$PERF_DEEP_MIN_AVAILABILITY_PCT"
    ;;
  *)
    echo "Unsupported lane: $LANE" >&2
    exit 2
    ;;
esac

extract_metric() {
  local key="$1"
  local value
  value="$(
    grep -Eo "\"${key}\"[[:space:]]*:[[:space:]]*-?[0-9]+([.][0-9]+)?" "$REPORT_JSON" \
      | head -n 1 \
      | sed -E 's/.*:[[:space:]]*//'
  )"

  if [ -z "$value" ]; then
    echo "missing required metric: ${key}" >&2
    exit 2
  fi

  printf '%s' "$value"
}

metric_lt() {
  local observed="$1"
  local threshold="$2"
  awk -v observed="$observed" -v threshold="$threshold" 'BEGIN { exit !(observed < threshold) }'
}

metric_gte() {
  local observed="$1"
  local threshold="$2"
  awk -v observed="$observed" -v threshold="$threshold" 'BEGIN { exit !(observed >= threshold) }'
}

LATENCY_P50="$(extract_metric latency_p50_ms)"
LATENCY_P99="$(extract_metric latency_p99_ms)"
THROUGHPUT="$(extract_metric throughput_tps)"
AVAILABILITY="$(extract_metric availability_pct)"

failures=()

if ! metric_lt "$LATENCY_P50" "$MAX_P50"; then
  failures+=("latency_p50_ms>=${MAX_P50}")
fi

if ! metric_lt "$LATENCY_P99" "$MAX_P99"; then
  failures+=("latency_p99_ms>=${MAX_P99}")
fi

if ! metric_gte "$THROUGHPUT" "$MIN_THROUGHPUT"; then
  failures+=("throughput_tps<${MIN_THROUGHPUT}")
fi

if ! metric_gte "$AVAILABILITY" "$MIN_AVAILABILITY"; then
  failures+=("availability_pct<${MIN_AVAILABILITY}")
fi

if [ "${#failures[@]}" -gt 0 ]; then
  echo "status=fail; lane=${LANE}; failures=$(IFS=,; echo "${failures[*]}")"
  exit 1
fi

echo "status=pass; lane=${LANE}; latency_p50_ms=${LATENCY_P50}; latency_p99_ms=${LATENCY_P99}; throughput_tps=${THROUGHPUT}; availability_pct=${AVAILABILITY}"
