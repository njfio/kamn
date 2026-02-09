#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SELECTOR="$ROOT_DIR/scripts/runtime/select_failover_sync_drill_lane.sh"

if [ ! -x "$SELECTOR" ]; then
  echo "expected failover/sync selector script to be executable" >&2
  exit 1
fi

pull_request_output="$(bash "$SELECTOR" --event-name pull_request)"
if ! printf '%s\n' "$pull_request_output" | grep -q '^lane=preflight$'; then
  echo "expected pull_request routing to preflight lane" >&2
  exit 1
fi
if ! printf '%s\n' "$pull_request_output" | grep -q '^run_preflight=true$'; then
  echo "expected pull_request to run preflight" >&2
  exit 1
fi
if ! printf '%s\n' "$pull_request_output" | grep -q '^run_deep=false$'; then
  echo "expected pull_request to skip deep lane" >&2
  exit 1
fi

schedule_output="$(bash "$SELECTOR" --event-name schedule)"
if ! printf '%s\n' "$schedule_output" | grep -q '^lane=deep$'; then
  echo "expected schedule routing to deep lane" >&2
  exit 1
fi
if ! printf '%s\n' "$schedule_output" | grep -q '^run_preflight=false$'; then
  echo "expected schedule to skip preflight" >&2
  exit 1
fi
if ! printf '%s\n' "$schedule_output" | grep -q '^run_deep=true$'; then
  echo "expected schedule to run deep lane" >&2
  exit 1
fi

manual_output="$(bash "$SELECTOR" --event-name workflow_dispatch)"
if ! printf '%s\n' "$manual_output" | grep -q '^lane=deep$'; then
  echo "expected workflow_dispatch routing to deep lane" >&2
  exit 1
fi

# Regression: #788
unknown_output="$(bash "$SELECTOR" --event-name release)"
if ! printf '%s\n' "$unknown_output" | grep -q '^lane=preflight$'; then
  echo "expected unknown events to fail-safe to preflight lane" >&2
  exit 1
fi

echo "failover/sync selector script tests passed."
