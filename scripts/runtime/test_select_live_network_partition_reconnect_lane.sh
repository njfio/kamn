#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SELECTOR="$ROOT_DIR/scripts/runtime/select_live_network_partition_reconnect_lane.sh"
TMP_GITHUB_OUTPUT="$(mktemp)"
trap 'rm -f "$TMP_GITHUB_OUTPUT"' EXIT

if [ ! -x "$SELECTOR" ]; then
  echo "expected partition/reconnect lane selector to be executable" >&2
  exit 1
fi

pull_request_output="$(bash "$SELECTOR" --event-name pull_request)"
if ! printf '%s\n' "$pull_request_output" | grep -q '^lane=smoke$'; then
  echo "expected pull_request routing to smoke lane" >&2
  exit 1
fi
if ! printf '%s\n' "$pull_request_output" | grep -q '^run_smoke=true$'; then
  echo "expected pull_request to run smoke lane" >&2
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
if ! printf '%s\n' "$schedule_output" | grep -q '^cadence=scheduled$'; then
  echo "expected schedule cadence marker" >&2
  exit 1
fi

manual_output="$(bash "$SELECTOR" --event-name workflow_dispatch)"
if ! printf '%s\n' "$manual_output" | grep -q '^lane=deep$'; then
  echo "expected workflow_dispatch routing to deep lane" >&2
  exit 1
fi
if ! printf '%s\n' "$manual_output" | grep -q '^cadence=manual$'; then
  echo "expected workflow_dispatch cadence marker" >&2
  exit 1
fi

# Regression: #982
fallback_output="$(bash "$SELECTOR" --event-name release)"
if ! printf '%s\n' "$fallback_output" | grep -q '^lane=smoke$'; then
  echo "expected unknown events to fail-safe to smoke lane" >&2
  exit 1
fi

github_output_mode="$(
  GITHUB_OUTPUT="$TMP_GITHUB_OUTPUT" \
    bash "$SELECTOR" --event-name pull_request
)"
if ! printf '%s\n' "$github_output_mode" | grep -q '^lane=smoke$'; then
  echo "expected selector stdout output under GitHub output env" >&2
  exit 1
fi
if ! grep -q '^lane=smoke$' "$TMP_GITHUB_OUTPUT"; then
  echo "expected selector to mirror lane output to GITHUB_OUTPUT file" >&2
  exit 1
fi

echo "partition/reconnect lane selector script tests passed."
