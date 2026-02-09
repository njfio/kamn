#!/usr/bin/env bash
set -euo pipefail

write_output() {
  local key="$1"
  local value="$2"
  if [ -n "${GITHUB_OUTPUT:-}" ]; then
    {
      echo "${key}=${value}"
    } >>"$GITHUB_OUTPUT"
  else
    printf '%s=%s\n' "$key" "$value"
  fi
}

event_name="${GITHUB_EVENT_NAME:-pull_request}"
force_lane=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --event-name)
      event_name="${2:-}"
      shift 2
      ;;
    --force-lane)
      force_lane="${2:-}"
      shift 2
      ;;
    --help|-h)
      cat <<'USAGE'
Usage:
  bash scripts/runtime/select_failover_sync_drill_lane.sh \
    [--event-name <github-event>] \
    [--force-lane preflight|deep]
USAGE
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

selected_lane=""

if [[ -n "$force_lane" ]]; then
  case "$force_lane" in
    preflight|deep)
      selected_lane="$force_lane"
      ;;
    *)
      echo "invalid forced lane: $force_lane (expected preflight or deep)" >&2
      exit 1
      ;;
  esac
else
  case "$event_name" in
    schedule|workflow_dispatch)
      selected_lane="deep"
      ;;
    pull_request|pull_request_target|push|merge_group)
      selected_lane="preflight"
      ;;
    *)
      selected_lane="preflight"
      ;;
  esac
fi

run_preflight="false"
run_deep="false"
cadence="pr-fast"

if [[ "$selected_lane" == "deep" ]]; then
  run_deep="true"
  cadence="scheduled"
else
  run_preflight="true"
fi

write_output "event_name" "$event_name"
write_output "lane" "$selected_lane"
write_output "run_preflight" "$run_preflight"
write_output "run_deep" "$run_deep"
write_output "cadence" "$cadence"
