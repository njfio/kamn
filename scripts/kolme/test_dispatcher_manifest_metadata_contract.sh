#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUN_DISPATCHER="$ROOT_DIR/scripts/kolme/run_lane_dispatch.sh"
CONTRACT_DISPATCHER="$ROOT_DIR/scripts/kolme/run_contract_lane_dispatch.sh"
FALLBACK_REASON_TAXONOMY_VERSION="kamn.kolme.dispatch-fallback-reason-taxonomy.v1"
FALLBACK_REASON_CODES_CSV="dispatcher_unknown_wrapper,dispatcher_manifest_missing,dispatcher_phase_unmapped"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$RUN_DISPATCHER" ]; then
  echo "expected run dispatcher to be executable: $RUN_DISPATCHER" >&2
  exit 1
fi

if [ ! -x "$CONTRACT_DISPATCHER" ]; then
  echo "expected contract dispatcher to be executable: $CONTRACT_DISPATCHER" >&2
  exit 1
fi

if grep -q "resolve_manifest_name()" "$RUN_DISPATCHER"; then
  echo "expected run dispatcher to resolve wrappers via manifest metadata, not resolve_manifest_name case map" >&2
  exit 1
fi

if grep -q "resolve_manifest_name()" "$CONTRACT_DISPATCHER"; then
  echo "expected contract dispatcher to resolve wrappers via manifest metadata, not resolve_manifest_name case map" >&2
  exit 1
fi

grep -En '^[[:space:]]*run_.*\) echo "[^"]+" ;;' "$RUN_DISPATCHER" \
  | sed -E 's#^.*(run_[^)]+)\) echo "([^"]+)" ;.*#\1 \2#' >"$TMP_DIR/run_map.txt" || true
grep -En '^[[:space:]]*run_.*\) echo "[^"]+" ;;' "$CONTRACT_DISPATCHER" \
  | sed -E 's#^.*(run_[^)]+)\) echo "([^"]+)" ;.*#\1 \2#' >"$TMP_DIR/contract_map.txt" || true

if [ -s "$TMP_DIR/run_map.txt" ] || [ -s "$TMP_DIR/contract_map.txt" ]; then
  echo "expected dispatchers to avoid hardcoded wrapper->manifest case entries" >&2
  exit 1
fi

mapfile -t run_wrappers < <(
  find "$ROOT_DIR/scripts/kolme" -maxdepth 1 -type l -name 'run_*_lane.sh' \
    ! -name 'run_*_contract_lane.sh' \
    ! -name 'run_lane_dispatch.sh' \
    ! -name 'run_contract_lane_dispatch.sh' \
    -printf '%f\n' | sort
)
mapfile -t contract_wrappers < <(find "$ROOT_DIR/scripts/kolme" -maxdepth 1 -type l -name 'run_*_contract_lane.sh' -printf '%f\n' | sort)

if [ "${#run_wrappers[@]}" -eq 0 ] || [ "${#contract_wrappers[@]}" -eq 0 ]; then
  echo "expected run and contract wrapper symlink inventory to be non-empty" >&2
  exit 1
fi

check_wrapper_manifest_metadata() {
  local dispatcher="$1"
  local wrapper="$2"
  local expected_phase="$3"
  local manifest_path
  manifest_path="$(bash "$dispatcher" --lane-wrapper "$wrapper" --resolve-manifest-path)"
  if [ ! -f "$manifest_path" ]; then
    echo "expected dispatcher to resolve manifest for $wrapper: $manifest_path" >&2
    exit 1
  fi
  python3 - "$manifest_path" "$wrapper" "$expected_phase" <<'PY'
import json
import pathlib
import sys

manifest = pathlib.Path(sys.argv[1])
wrapper = sys.argv[2]
expected_phase = sys.argv[3]
payload = json.loads(manifest.read_text(encoding="utf-8"))
if payload.get("wrapper_name") != wrapper:
    raise SystemExit(f"expected wrapper_name={wrapper!r} in {manifest.name}")
phase = payload.get("phase")
if phase != expected_phase:
    raise SystemExit(f"expected phase={expected_phase!r} in {manifest.name}, got {phase!r}")
phases = payload.get("phases")
if not isinstance(phases, dict) or expected_phase not in phases:
    raise SystemExit(f"expected phase {expected_phase!r} to exist in phases map for {manifest.name}")
PY
}

for wrapper in "${run_wrappers[@]}"; do
  check_wrapper_manifest_metadata "$RUN_DISPATCHER" "$wrapper" "run"
done

for wrapper in "${contract_wrappers[@]}"; do
  check_wrapper_manifest_metadata "$CONTRACT_DISPATCHER" "$wrapper" "contract"
done

set +e
unknown_output="$(
  bash "$RUN_DISPATCHER" --lane-wrapper run_missing_dispatch_metadata_lane.sh --resolve-manifest-path 2>&1
)"
unknown_code=$?
set -e

if [ "$unknown_code" -eq 0 ]; then
  echo "expected run dispatcher to fail closed for unknown wrapper" >&2
  exit 1
fi

if ! printf '%s\n' "$unknown_output" | grep -q '^dispatch_status=fail$'; then
  echo "expected deterministic dispatch_status=fail for unknown wrapper" >&2
  exit 1
fi
if ! printf '%s\n' "$unknown_output" | grep -q "^fallback_reason_taxonomy_version=$FALLBACK_REASON_TAXONOMY_VERSION$"; then
  echo "expected deterministic fallback reason taxonomy marker for unknown wrapper" >&2
  exit 1
fi
if ! printf '%s\n' "$unknown_output" | grep -q "^fallback_reason_codes_csv=$FALLBACK_REASON_CODES_CSV$"; then
  echo "expected deterministic fallback reason codes CSV marker for unknown wrapper" >&2
  exit 1
fi

echo "Kolme dispatcher manifest metadata contract tests passed."
