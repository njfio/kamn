#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<USAGE
Usage: $0 --lane <fast-gate|deep-validate> --elapsed-seconds <seconds> [options]

Options:
  --test-scope <scope>       Optional scope label (docs-only/smoke/targeted/full)
  --changed-files <count>    Optional changed file count
  --job-count <count>        Approximate concurrent job multiplier (default: 1)
  --cache-hit <value>        Rust cache status (e.g., true/false/partial/unknown)
  --retry-used <value>       Whether bounded retry was used (true/false/unknown)
  --budget-file <path>       Budget config file (default: .ci/ci-budget.env)
  --output-json <path>       Optional JSON metrics output path
USAGE
}

write_output() {
  local key="$1"
  local value="$2"
  if [ -n "${GITHUB_OUTPUT:-}" ]; then
    {
      echo "${key}<<EOF"
      echo "$value"
      echo "EOF"
    } >>"$GITHUB_OUTPUT"
  fi
}

append_summary() {
  if [ -z "${GITHUB_STEP_SUMMARY:-}" ]; then
    return
  fi

  {
    echo "### CI Budget Evaluation (${LANE})"
    echo "- Status: ${STATUS}"
    echo "- Elapsed seconds: ${ELAPSED_SECONDS}"
    echo "- Elapsed minutes (rounded up): ${ELAPSED_MINUTES}"
    echo "- Approx. runner-minutes: ${RUNNER_MINUTES}"
    echo "- Job count: ${JOB_COUNT}"
    echo "- Warn threshold seconds: ${WARN_SECONDS}"
    echo "- Max threshold seconds: ${MAX_SECONDS}"
    echo "- Warn threshold runner-minutes: ${WARN_RUNNER_MINUTES}"
    echo "- Max threshold runner-minutes: ${MAX_RUNNER_MINUTES}"
    echo "- Test scope: ${TEST_SCOPE}"
    echo "- Changed files: ${CHANGED_FILES}"
    echo "- Rust cache hit: ${CACHE_HIT}"
    echo "- Retry used: ${RETRY_USED}"
    if [ -n "${BUDGET_NOTES}" ]; then
      echo "- Notes: ${BUDGET_NOTES}"
    fi
  } >>"$GITHUB_STEP_SUMMARY"
}

ceil_div() {
  local dividend="$1"
  local divisor="$2"
  echo $(( (dividend + divisor - 1) / divisor ))
}

json_escape() {
  local text="$1"
  text="${text//\\/\\\\}"
  text="${text//\"/\\\"}"
  text="${text//$'\n'/\\n}"
  printf '%s' "$text"
}

LANE=""
ELAPSED_SECONDS=""
TEST_SCOPE="unknown"
CHANGED_FILES="0"
JOB_COUNT="1"
CACHE_HIT="unknown"
RETRY_USED="unknown"
BUDGET_FILE=".ci/ci-budget.env"
OUTPUT_JSON=""

while [ "$#" -gt 0 ]; do
  case "$1" in
    --lane)
      LANE="${2:-}"
      shift 2
      ;;
    --elapsed-seconds)
      ELAPSED_SECONDS="${2:-}"
      shift 2
      ;;
    --test-scope)
      TEST_SCOPE="${2:-unknown}"
      shift 2
      ;;
    --changed-files)
      CHANGED_FILES="${2:-0}"
      shift 2
      ;;
    --job-count)
      JOB_COUNT="${2:-1}"
      shift 2
      ;;
    --cache-hit)
      CACHE_HIT="${2:-unknown}"
      shift 2
      ;;
    --retry-used)
      RETRY_USED="${2:-unknown}"
      shift 2
      ;;
    --budget-file)
      BUDGET_FILE="${2:-.ci/ci-budget.env}"
      shift 2
      ;;
    --output-json)
      OUTPUT_JSON="${2:-}"
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

if [ -z "$LANE" ] || [ -z "$ELAPSED_SECONDS" ]; then
  usage >&2
  exit 2
fi

if ! [[ "$ELAPSED_SECONDS" =~ ^[0-9]+$ ]]; then
  echo "--elapsed-seconds must be an integer" >&2
  exit 2
fi

if ! [[ "$JOB_COUNT" =~ ^[0-9]+$ ]] || [ "$JOB_COUNT" -lt 1 ]; then
  echo "--job-count must be an integer >= 1" >&2
  exit 2
fi

if ! [[ "$CHANGED_FILES" =~ ^[0-9]+$ ]]; then
  CHANGED_FILES="0"
fi

if [ ! -f "$BUDGET_FILE" ]; then
  echo "Budget file not found: $BUDGET_FILE" >&2
  exit 2
fi

# shellcheck disable=SC1090
source "$BUDGET_FILE"

case "$LANE" in
  fast-gate)
    MAX_SECONDS="${FAST_GATE_MAX_SECONDS}"
    WARN_PERCENT="${FAST_GATE_WARN_PERCENT}"
    MAX_RUNNER_MINUTES="${FAST_GATE_MAX_RUNNER_MINUTES}"
    MAX_JOB_COUNT="${FAST_GATE_MAX_JOB_COUNT}"
    ;;
  deep-validate)
    MAX_SECONDS="${DEEP_VALIDATE_MAX_SECONDS}"
    WARN_PERCENT="${DEEP_VALIDATE_WARN_PERCENT}"
    MAX_RUNNER_MINUTES="${DEEP_VALIDATE_MAX_RUNNER_MINUTES}"
    MAX_JOB_COUNT="${DEEP_VALIDATE_MAX_JOB_COUNT}"
    ;;
  *)
    echo "Unsupported lane: $LANE" >&2
    exit 2
    ;;
esac

if ! [[ "$MAX_SECONDS" =~ ^[0-9]+$ && "$WARN_PERCENT" =~ ^[0-9]+$ && "$MAX_RUNNER_MINUTES" =~ ^[0-9]+$ && "$MAX_JOB_COUNT" =~ ^[0-9]+$ ]]; then
  echo "Invalid numeric values in budget file: $BUDGET_FILE" >&2
  exit 2
fi

WARN_SECONDS=$(( MAX_SECONDS * WARN_PERCENT / 100 ))
WARN_RUNNER_MINUTES=$(( MAX_RUNNER_MINUTES * WARN_PERCENT / 100 ))

ELAPSED_MINUTES="$(ceil_div "$ELAPSED_SECONDS" 60)"
RUNNER_MINUTES=$(( ELAPSED_MINUTES * JOB_COUNT ))

STATUS="pass"
BUDGET_NOTES=""

warn_msgs=()
fail_msgs=()

if [ "$ELAPSED_SECONDS" -gt "$MAX_SECONDS" ]; then
  fail_msgs+=("elapsed-seconds>${MAX_SECONDS}")
elif [ "$ELAPSED_SECONDS" -ge "$WARN_SECONDS" ]; then
  warn_msgs+=("elapsed-seconds>=${WARN_SECONDS}")
fi

if [ "$RUNNER_MINUTES" -gt "$MAX_RUNNER_MINUTES" ]; then
  fail_msgs+=("runner-minutes>${MAX_RUNNER_MINUTES}")
elif [ "$RUNNER_MINUTES" -ge "$WARN_RUNNER_MINUTES" ]; then
  warn_msgs+=("runner-minutes>=${WARN_RUNNER_MINUTES}")
fi

if [ "$JOB_COUNT" -gt "$MAX_JOB_COUNT" ]; then
  fail_msgs+=("job-count>${MAX_JOB_COUNT}")
elif [ "$JOB_COUNT" -eq "$MAX_JOB_COUNT" ] && [ "$MAX_JOB_COUNT" -gt 0 ] && [ "$WARN_PERCENT" -le 100 ]; then
  BUDGET_NOTES="job-count at configured maximum"
fi

if [ "${#fail_msgs[@]}" -gt 0 ]; then
  STATUS="fail"
elif [ "${#warn_msgs[@]}" -gt 0 ]; then
  STATUS="warn"
fi

MESSAGE="status=${STATUS}; lane=${LANE}; elapsed=${ELAPSED_SECONDS}s; runner_minutes=${RUNNER_MINUTES}; cache_hit=${CACHE_HIT}; retry_used=${RETRY_USED}"
if [ "${#warn_msgs[@]}" -gt 0 ]; then
  MESSAGE+="; warnings=$(IFS=,; echo "${warn_msgs[*]}")"
fi
if [ "${#fail_msgs[@]}" -gt 0 ]; then
  MESSAGE+="; failures=$(IFS=,; echo "${fail_msgs[*]}")"
fi

echo "$MESSAGE"

write_output "budget_status" "$STATUS"
write_output "budget_lane" "$LANE"
write_output "budget_elapsed_seconds" "$ELAPSED_SECONDS"
write_output "budget_runner_minutes" "$RUNNER_MINUTES"
write_output "budget_cache_hit" "$CACHE_HIT"
write_output "budget_retry_used" "$RETRY_USED"
write_output "budget_message" "$MESSAGE"

append_summary

if [ -n "$OUTPUT_JSON" ]; then
  ts="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  workflow="$(json_escape "${GITHUB_WORKFLOW:-unknown}")"
  job="$(json_escape "${GITHUB_JOB:-unknown}")"
  ref="$(json_escape "${GITHUB_REF:-unknown}")"
  sha="$(json_escape "${GITHUB_SHA:-unknown}")"
  run_id="$(json_escape "${GITHUB_RUN_ID:-unknown}")"
  run_attempt="$(json_escape "${GITHUB_RUN_ATTEMPT:-unknown}")"
  scope_json="$(json_escape "$TEST_SCOPE")"
  status_json="$(json_escape "$STATUS")"
  cache_json="$(json_escape "$CACHE_HIT")"
  retry_json="$(json_escape "$RETRY_USED")"
  message_json="$(json_escape "$MESSAGE")"

  cat > "$OUTPUT_JSON" <<JSON
{
  "timestamp_utc": "$ts",
  "lane": "$LANE",
  "status": "$status_json",
  "message": "$message_json",
  "elapsed_seconds": $ELAPSED_SECONDS,
  "elapsed_minutes": $ELAPSED_MINUTES,
  "runner_minutes": $RUNNER_MINUTES,
  "job_count": $JOB_COUNT,
  "changed_files": $CHANGED_FILES,
  "test_scope": "$scope_json",
  "cache_hit": "$cache_json",
  "retry_used": "$retry_json",
  "thresholds": {
    "warn_percent": $WARN_PERCENT,
    "max_seconds": $MAX_SECONDS,
    "warn_seconds": $WARN_SECONDS,
    "max_runner_minutes": $MAX_RUNNER_MINUTES,
    "warn_runner_minutes": $WARN_RUNNER_MINUTES,
    "max_job_count": $MAX_JOB_COUNT
  },
  "github": {
    "workflow": "$workflow",
    "job": "$job",
    "ref": "$ref",
    "sha": "$sha",
    "run_id": "$run_id",
    "run_attempt": "$run_attempt"
  }
}
JSON
fi

if [ "$STATUS" = "fail" ]; then
  exit 1
fi
