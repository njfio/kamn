#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"
CONFIG_FILE="$ROOT_DIR/fixtures/ci/kolme_manifest_migration_contract_groups.json"
MANIFEST_RUNNER="$ROOT_DIR/scripts/framework/run_manifest_lane.sh"
GROUP_KEY="tranche1"
MAX_SECONDS_ENV="KAMN_KOLME_TRANCHE1_DISPATCH_PARITY_MAX_SECONDS"
DEFAULT_MAX_SECONDS=240

test_harness_require_file "$CONFIG_FILE" "expected migration group config file to exist: $CONFIG_FILE"

test_harness_require_file "$MANIFEST_RUNNER" "expected manifest runner script to exist: $MANIFEST_RUNNER"

max_seconds="${!MAX_SECONDS_ENV:-$DEFAULT_MAX_SECONDS}"
if ! [[ "$max_seconds" =~ ^[0-9]+$ ]] || [ "$max_seconds" -le 0 ]; then
  echo "$MAX_SECONDS_ENV must be a positive integer" >&2
  exit 1
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

mapfile -t lane_specs < <(python3 - "$CONFIG_FILE" "$GROUP_KEY" <<'PY'
import json
import sys
from pathlib import Path

config_path = Path(sys.argv[1])
group_key = sys.argv[2]
payload = json.loads(config_path.read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.kolme-manifest-migration-contract-groups.v1":
    raise SystemExit("unexpected migration group schema")
group = payload.get("groups", {}).get(group_key)
if not isinstance(group, dict):
    raise SystemExit(f"missing migration group: {group_key}")
lanes = group.get("lanes")
if not isinstance(lanes, list) or not lanes:
    raise SystemExit(f"migration group {group_key} lanes must be non-empty")
for lane in lanes:
    lane_script = lane.get("lane_script")
    manifest_file = lane.get("manifest_file")
    lane_id = lane.get("lane_id")
    if not all(isinstance(field, str) and field for field in (lane_script, manifest_file, lane_id)):
        raise SystemExit(f"invalid lane entry in group {group_key}")
    print(f"{lane_script}|{manifest_file}|{lane_id}")
PY
)

if [ "${#lane_specs[@]}" -eq 0 ]; then
  echo "expected non-empty lane specs for group $GROUP_KEY" >&2
  exit 1
fi

normalize_output() {
  python3 - "$1" <<'PY'
import re
import sys
from pathlib import Path

patterns = (
    re.compile(r"^\s*(Compiling|Checking|Documenting)\s+.+$"),
    re.compile(r"^\s*Finished `.* target\(s\) in .*s$"),
    re.compile(r"^\s*Running tests/.+\(target/.+\)$"),
    re.compile(r"^\s*Blocking waiting for file lock on .+$"),
)
for raw in Path(sys.argv[1]).read_text(encoding="utf-8").splitlines():
    line = raw.rstrip()
    if not line:
        continue
    if any(pattern.match(line) for pattern in patterns):
        continue
    print(line)
PY
}

start_epoch="$(date +%s)"

for spec in "${lane_specs[@]}"; do
  IFS='|' read -r lane_script manifest_file lane_id <<<"$spec"

  lane_script_path="$ROOT_DIR/$lane_script"
  manifest_path="$ROOT_DIR/$manifest_file"

  if [ ! -f "$lane_script_path" ]; then
    echo "expected lane script file for parity check: $lane_script_path" >&2
    exit 1
  fi

  if [ ! -f "$manifest_path" ]; then
    echo "expected manifest file for parity check: $manifest_path" >&2
    exit 1
  fi

  wrapper_output="$TMP_DIR/${lane_id//./_}.wrapper.out"
  direct_output="$TMP_DIR/${lane_id//./_}.direct.out"
  wrapper_normalized="$TMP_DIR/${lane_id//./_}.wrapper.normalized.out"
  direct_normalized="$TMP_DIR/${lane_id//./_}.direct.normalized.out"

  if ! bash "$lane_script_path" >"$wrapper_output" 2>&1; then
    echo "expected wrapper lane command to pass for parity check: $lane_script" >&2
    cat "$wrapper_output" >&2 || true
    exit 1
  fi

  if ! bash "$MANIFEST_RUNNER" --manifest "$manifest_path" --phase contract >"$direct_output" 2>&1; then
    echo "expected direct manifest lane command to pass for parity check: $manifest_file" >&2
    cat "$direct_output" >&2 || true
    exit 1
  fi

  normalize_output "$wrapper_output" >"$wrapper_normalized"
  normalize_output "$direct_output" >"$direct_normalized"

  if ! grep -Fxq "lane_id=$lane_id" "$wrapper_normalized"; then
    echo "expected wrapper output to include lane_id marker: $lane_id" >&2
    cat "$wrapper_normalized" >&2 || true
    exit 1
  fi

  if ! grep -Fxq "lane_id=$lane_id" "$direct_normalized"; then
    echo "expected direct output to include lane_id marker: $lane_id" >&2
    cat "$direct_normalized" >&2 || true
    exit 1
  fi

  if ! grep -Fxq "status=ok" "$wrapper_normalized"; then
    echo "expected wrapper output to include status=ok for lane: $lane_id" >&2
    cat "$wrapper_normalized" >&2 || true
    exit 1
  fi

  if ! grep -Fxq "status=ok" "$direct_normalized"; then
    echo "expected direct output to include status=ok for lane: $lane_id" >&2
    cat "$direct_normalized" >&2 || true
    exit 1
  fi

  if ! diff -u "$wrapper_normalized" "$direct_normalized" >/dev/null; then
    echo "expected wrapper/direct normalized outputs to match for lane: $lane_id" >&2
    diff -u "$wrapper_normalized" "$direct_normalized" >&2 || true
    exit 1
  fi

  elapsed_seconds=$(( $(date +%s) - start_epoch ))
  if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
    echo "tranche-1 dispatch execution parity contract exceeded runtime budget: ${elapsed_seconds}s" >&2
    exit 1
  fi

done

echo "Kolme tranche-1 dispatch execution parity contract tests passed."
