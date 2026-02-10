#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNTIME_NETWORK_DOC="$ROOT_DIR/docs/foundation/runtime-network.md"
TEST_TARGET="kolme_runtime_commit_notifications"
MAX_SECONDS="${KAMN_KOLME_NOTIFICATIONS_CONSUMER_MAX_SECONDS:-60}"

if ! [[ "$MAX_SECONDS" =~ ^[0-9]+$ ]] || [ "$MAX_SECONDS" -le 0 ]; then
  echo "KAMN_KOLME_NOTIFICATIONS_CONSUMER_MAX_SECONDS must be a positive integer" >&2
  exit 1
fi

if [ ! -f "$RUNTIME_NETWORK_DOC" ]; then
  echo "expected runtime network documentation to exist" >&2
  exit 1
fi

start_epoch="$(date +%s)"

cargo test -p kamn-core --test "$TEST_TARGET" >/dev/null

if ! grep -q "run_notifications_consumer_contract_lane.sh" "$RUNTIME_NETWORK_DOC"; then
  echo "expected runtime network documentation to reference notifications consumer lane command" >&2
  exit 1
fi

if ! grep -q 'Regression: #1463' "$RUNTIME_NETWORK_DOC"; then
  echo "expected runtime network documentation to include notifications consumer regression marker" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$MAX_SECONDS" ]; then
  echo "Kolme notifications consumer contract lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

echo "Kolme notifications consumer contract lane tests passed."
