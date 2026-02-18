#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
INVENTORY="$ROOT_DIR/fixtures/ci/superseded_script_inventory_baseline.json"

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
  if [ -e "$wrapper_path" ]; then
    echo "expected superseded wrapper script to be removed: $wrapper" >&2
    exit 1
  fi
done

python3 - "$INVENTORY" "$ROOT_DIR" <<'PY'
import json
import sys
from pathlib import Path

inventory_path = Path(sys.argv[1])
root = Path(sys.argv[2])
payload = json.loads(inventory_path.read_text(encoding="utf-8"))
expected_runner = "scripts/framework/run_manifest_lane.sh"

errors = []
for entry in payload.get("superseded_scripts", []):
    path = entry.get("script_path")
    if not (isinstance(path, str) and path.startswith("scripts/kolme/")):
        continue
    evidence = entry.get("replacement_evidence")
    if not isinstance(evidence, dict):
        errors.append(f"missing replacement evidence for {path}")
        continue
    runner = evidence.get("replacement_runner")
    if runner != expected_runner:
        errors.append(
            f"unexpected replacement runner for {path}: {runner!r} (expected {expected_runner!r})"
        )
    for key in ("manifest_file", "contract_script"):
        value = evidence.get(key)
        if not isinstance(value, str) or not value.strip():
            errors.append(f"missing {key} replacement evidence for {path}")
            continue
        asset = (root / value).resolve()
        if not asset.is_file():
            errors.append(f"{key} replacement asset missing for {path}: {value}")

if errors:
    raise SystemExit("\n".join(errors))
PY

echo "Kolme contract-lane wrapper compaction tests passed (post-deletion invariant)."
