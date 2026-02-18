#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
INVENTORY="$ROOT_DIR/fixtures/ci/superseded_script_inventory_baseline.json"
MAX_WRAPPER_LINES=20

if [ ! -f "$INVENTORY" ]; then
  echo "expected superseded script inventory baseline fixture: $INVENTORY" >&2
  exit 1
fi

mapfile -t WRAPPERS < <(
  python3 - "$INVENTORY" <<'PY'
import json
import sys
from pathlib import Path

payload = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
for entry in payload.get("superseded_scripts", []):
    path = entry.get("script_path", "")
    if isinstance(path, str) and path.startswith("scripts/kolme/") and path.endswith(".sh"):
        print(path)
PY
)

if [ "${#WRAPPERS[@]}" -eq 0 ]; then
  echo "expected at least one superseded Kolme wrapper entry in inventory" >&2
  exit 1
fi

for wrapper in "${WRAPPERS[@]}"; do
  wrapper_path="$ROOT_DIR/$wrapper"
  if [ ! -x "$wrapper_path" ]; then
    echo "expected wrapper script to be executable: $wrapper" >&2
    exit 1
  fi
  if ! grep -Fq "scripts/kolme/run_contract_lane_dispatch.sh" "$wrapper_path"; then
    echo "expected wrapper to dispatch via run_contract_lane_dispatch.sh: $wrapper" >&2
    exit 1
  fi
  wrapper_lines="$(wc -l <"$wrapper_path")"
  if [ "$wrapper_lines" -gt "$MAX_WRAPPER_LINES" ]; then
    echo "expected compact wrapper (<= $MAX_WRAPPER_LINES lines): $wrapper has $wrapper_lines lines" >&2
    exit 1
  fi
done

echo "Kolme contract-lane wrapper compaction tests passed."
