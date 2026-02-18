#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DOC_FILE="$ROOT_DIR/docs/foundation/runtime-network.md"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/kolme_notifications_consumer_contract_lane.json"

if [ ! -f "$MANIFEST" ]; then
  echo "expected notifications consumer contract lane manifest to exist" >&2
  exit 1
fi

python3 - "$MANIFEST" <<'PY'
import json
import pathlib
import sys

manifest_path = pathlib.Path(sys.argv[1])
payload = json.loads(manifest_path.read_text(encoding="utf-8"))
contract_command = payload.get("phases", {}).get("contract")
if contract_command != [
    "python3",
    "scripts/kolme/contracts/notifications_consumer_contract_lane.py",
]:
    raise SystemExit("expected notifications consumer manifest contract command")
PY

if [ ! -f "$DOC_FILE" ]; then
  echo "expected runtime network documentation to exist" >&2
  exit 1
fi

if ! grep -q "run_notifications_consumer_contract_lane.sh" "$DOC_FILE" \
  && ! grep -q "run_manifest_lane.sh --manifest scripts/framework/manifests/kolme_notifications_consumer_contract_lane.json --phase contract" "$DOC_FILE"; then
  echo "expected runtime network documentation to reference notifications consumer lane command" >&2
  exit 1
fi

if ! grep -q 'Regression: #1463' "$DOC_FILE"; then
  echo "expected runtime network documentation to include notifications consumer regression marker" >&2
  exit 1
fi

KAMN_KOLME_NOTIFICATIONS_CONSUMER_MAX_SECONDS=60 \
  bash "$ROOT_DIR/scripts/framework/run_manifest_lane.sh" \
    --manifest "$MANIFEST" \
    --phase contract \
    >/tmp/kolme-notifications-consumer-lane.log

if ! grep -q "Kolme notifications consumer contract lane tests passed." /tmp/kolme-notifications-consumer-lane.log; then
  echo "expected notifications consumer contract lane success output" >&2
  exit 1
fi

echo "notifications consumer contract lane test passed."
