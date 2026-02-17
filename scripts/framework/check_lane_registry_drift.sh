#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
GENERATOR="$SCRIPT_DIR/generate_lane_artifacts.py"
REPO_ROOT="$ROOT_DIR"
REGISTRY_FILE="$ROOT_DIR/scripts/framework/lane_registry.json"
REASON_TAXONOMY_VERSION="kamn.framework.lane-registry-drift-reason-taxonomy.v1"

extract_line_value() {
  local payload="$1"
  local key="$2"
  printf '%s\n' "$payload" | awk -F= -v key="$key" '$1 == key {print substr($0, index($0, "=") + 1); exit}'
}

usage() {
  cat <<'USAGE'
Usage:
  bash scripts/framework/check_lane_registry_drift.sh [--repo-root <path>] [--registry-file <path>]
USAGE
}

while (($# > 0)); do
  case "$1" in
    --repo-root)
      if (($# < 2)); then
        echo "missing value for --repo-root" >&2
        usage
        exit 1
      fi
      REPO_ROOT="$2"
      shift 2
      ;;
    --registry-file)
      if (($# < 2)); then
        echo "missing value for --registry-file" >&2
        usage
        exit 1
      fi
      REGISTRY_FILE="$2"
      shift 2
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage
      exit 1
      ;;
  esac
done

if [[ ! -x "$GENERATOR" ]]; then
  echo "status=fail"
  echo "final_decision=NO-GO"
  echo "reason_taxonomy_version=$REASON_TAXONOMY_VERSION"
  echo "reason_codes=lane_registry_generator_missing"
  echo "error_detail=lane artifact generator not executable: $GENERATOR"
  exit 1
fi

set +e
CHECK_OUTPUT="$(
  python3 "$GENERATOR" \
    --registry-file "$REGISTRY_FILE" \
    --repo-root "$REPO_ROOT" \
    --mode check 2>&1
)"
CHECK_STATUS=$?
set -e

if [[ "$CHECK_STATUS" -eq 0 ]]; then
  MANIFEST_ENTRIES="$(extract_line_value "$CHECK_OUTPUT" "manifest_entries")"
  WRAPPER_ENTRIES="$(extract_line_value "$CHECK_OUTPUT" "wrapper_entries")"
  echo "status=ok"
  echo "final_decision=GO"
  echo "reason_taxonomy_version=$REASON_TAXONOMY_VERSION"
  echo "reason_codes=none"
  echo "manifest_entries=${MANIFEST_ENTRIES:-0}"
  echo "wrapper_entries=${WRAPPER_ENTRIES:-0}"
  exit 0
fi

REASON_CODE="lane_registry_check_failed"
if printf '%s\n' "$CHECK_OUTPUT" | grep -q "manifest drift detected"; then
  REASON_CODE="lane_registry_manifest_drift_detected"
elif printf '%s\n' "$CHECK_OUTPUT" | grep -q "wrapper drift detected"; then
  REASON_CODE="lane_registry_wrapper_drift_detected"
elif printf '%s\n' "$CHECK_OUTPUT" | grep -q "registry schema_version mismatch"; then
  REASON_CODE="lane_registry_schema_mismatch"
elif printf '%s\n' "$CHECK_OUTPUT" | grep -Eq "not found|missing"; then
  REASON_CODE="lane_registry_artifact_missing"
fi

ERROR_DETAIL="$(extract_line_value "$CHECK_OUTPUT" "error")"
if [[ -z "$ERROR_DETAIL" ]]; then
  ERROR_DETAIL="$(printf '%s\n' "$CHECK_OUTPUT" | tail -n 1)"
fi

echo "status=fail"
echo "final_decision=NO-GO"
echo "reason_taxonomy_version=$REASON_TAXONOMY_VERSION"
echo "reason_codes=$REASON_CODE"
echo "error_detail=$ERROR_DETAIL"
exit 1
