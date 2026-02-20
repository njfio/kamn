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

extract_string_marker() {
  local key="$1"
  local value
  value="$(
    grep -Eo "\"${key}\"[[:space:]]*:[[:space:]]*\"[^\"]+\"" "$REPORT_JSON" \
      | head -n 1 \
      | sed -E 's/^[^:]+:[[:space:]]*\"([^\"]+)\"$/\1/'
  )"

  if [ -z "$value" ]; then
    echo "missing required baseline marker: ${key}" >&2
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
BASELINE_PROVENANCE_ARTIFACT_VERSION="$(extract_string_marker baseline_provenance_artifact_version)"
BASELINE_PROVENANCE_SOURCE_COMMIT="$(extract_string_marker baseline_provenance_source_commit)"
BASELINE_PROVENANCE_SOURCE_RUN_ID="$(extract_string_marker baseline_provenance_source_run_id)"
BASELINE_PROVENANCE_GENERATED_AT_UTC="$(extract_string_marker baseline_provenance_generated_at_utc)"
BASELINE_PROVENANCE_GENERATOR="$(extract_string_marker baseline_provenance_generator)"
DRIFT_THRESHOLD_SEED_ID="$(extract_string_marker drift_threshold_seed_id)"
DRIFT_THRESHOLD_SEED_MAX_P50="$(extract_metric drift_threshold_seed_max_latency_p50_ms)"
DRIFT_THRESHOLD_SEED_MAX_P99="$(extract_metric drift_threshold_seed_max_latency_p99_ms)"
DRIFT_THRESHOLD_SEED_MIN_THROUGHPUT="$(extract_metric drift_threshold_seed_min_throughput_tps)"
DRIFT_THRESHOLD_SEED_MIN_AVAILABILITY="$(extract_metric drift_threshold_seed_min_availability_pct)"

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

if ! metric_lt 0 "$DRIFT_THRESHOLD_SEED_MAX_P50"; then
  failures+=("drift_threshold_seed_max_latency_p50_ms<=0")
fi

if ! metric_lt 0 "$DRIFT_THRESHOLD_SEED_MAX_P99"; then
  failures+=("drift_threshold_seed_max_latency_p99_ms<=0")
fi

if ! metric_lt 0 "$DRIFT_THRESHOLD_SEED_MIN_THROUGHPUT"; then
  failures+=("drift_threshold_seed_min_throughput_tps<=0")
fi

if ! metric_gte "$DRIFT_THRESHOLD_SEED_MIN_AVAILABILITY" 0; then
  failures+=("drift_threshold_seed_min_availability_pct<0")
fi

if ! metric_lt "$DRIFT_THRESHOLD_SEED_MIN_AVAILABILITY" 101; then
  failures+=("drift_threshold_seed_min_availability_pct>100")
fi

if [ "${#failures[@]}" -gt 0 ]; then
  echo "status=fail; lane=${LANE}; failures=$(IFS=,; echo "${failures[*]}")"
  exit 1
fi

echo "status=pass; lane=${LANE}; latency_p50_ms=${LATENCY_P50}; latency_p99_ms=${LATENCY_P99}; throughput_tps=${THROUGHPUT}; availability_pct=${AVAILABILITY}; baseline_version=${BASELINE_PROVENANCE_ARTIFACT_VERSION}; baseline_commit=${BASELINE_PROVENANCE_SOURCE_COMMIT}; baseline_run_id=${BASELINE_PROVENANCE_SOURCE_RUN_ID}; baseline_generated_at_utc=${BASELINE_PROVENANCE_GENERATED_AT_UTC}; baseline_generator=${BASELINE_PROVENANCE_GENERATOR}; drift_threshold_seed_id=${DRIFT_THRESHOLD_SEED_ID}"
