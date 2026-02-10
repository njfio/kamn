#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LANE_SCRIPT="$ROOT_DIR/scripts/kolme/run_notifications_consumer_contract_lane.sh"
DOC_FILE="$ROOT_DIR/docs/foundation/runtime-network.md"

if [ ! -x "$LANE_SCRIPT" ]; then
  echo "expected notifications consumer contract lane script to be executable" >&2
  exit 1
fi

if [ ! -f "$DOC_FILE" ]; then
  echo "expected runtime network documentation to exist" >&2
  exit 1
fi

if ! grep -q "run_notifications_consumer_contract_lane.sh" "$DOC_FILE"; then
  echo "expected runtime network documentation to reference notifications consumer lane command" >&2
  exit 1
fi

if ! grep -q 'Regression: #1463' "$DOC_FILE"; then
  echo "expected runtime network documentation to include notifications consumer regression marker" >&2
  exit 1
fi

KAMN_KOLME_NOTIFICATIONS_CONSUMER_MAX_SECONDS=60 bash "$LANE_SCRIPT" >/tmp/kolme-notifications-consumer-lane.log

if ! grep -q "Kolme notifications consumer contract lane tests passed." /tmp/kolme-notifications-consumer-lane.log; then
  echo "expected notifications consumer contract lane success output" >&2
  exit 1
fi

echo "notifications consumer contract lane test passed."
