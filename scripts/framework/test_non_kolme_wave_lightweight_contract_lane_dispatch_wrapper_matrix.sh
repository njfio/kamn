#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)/scripts/lib/common.sh"
source "$KAMN_ROOT/scripts/lib/test_harness.sh"

SCRIPT_NAME="$(basename "$0")"
WAVE_NUMBER=""

if [[ "$SCRIPT_NAME" =~ ^test_non_kolme_wave([0-9]+)_lightweight_contract_lane_dispatch_wrapper_matrix\.sh$ ]]; then
  WAVE_NUMBER="${BASH_REMATCH[1]}"
fi

while (($# > 0)); do
  case "$1" in
    --wave)
      if (($# < 2)); then
        echo "missing value for --wave" >&2
        exit 1
      fi
      WAVE_NUMBER="$2"
      shift 2
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if [[ -z "$WAVE_NUMBER" ]]; then
  echo "wave number was not detected; use a wave symlink entrypoint or pass --wave <N>" >&2
  exit 1
fi

if [[ ! "$WAVE_NUMBER" =~ ^[0-9]+$ ]]; then
  echo "invalid wave number: $WAVE_NUMBER" >&2
  exit 1
fi

DISPATCHER="$KAMN_ROOT/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"
WRAPPERS_FILE="$KAMN_ROOT/scripts/framework/wave_definitions/non_kolme_wave${WAVE_NUMBER}_lightweight_wrappers.txt"

if ! test_harness_require_executable "$DISPATCHER" \
  "expected non-Kolme contract-lane dispatcher to be executable: $DISPATCHER"; then
  exit 1
fi

if ! test_harness_require_file "$WRAPPERS_FILE" \
  "expected non-Kolme wave wrapper definition file: $WRAPPERS_FILE"; then
  exit 1
fi

lane_wrappers=()
while IFS= read -r wrapper_rel_path; do
  [[ -z "$wrapper_rel_path" ]] && continue
  [[ "$wrapper_rel_path" == \#* ]] && continue
  lane_wrappers+=("$wrapper_rel_path")
done < "$WRAPPERS_FILE"

if ((${#lane_wrappers[@]} == 0)); then
  echo "expected at least one wrapper definition in $WRAPPERS_FILE" >&2
  exit 1
fi

for wrapper_rel_path in "${lane_wrappers[@]}"; do
  wrapper_path="$KAMN_ROOT/$wrapper_rel_path"
  wrapper_name="$(basename "$wrapper_path")"

  if ! test_harness_require_executable "$wrapper_path" \
    "expected lightweight wrapper to be executable: $wrapper_path"; then
    exit 1
  fi

  if [[ ! -L "$wrapper_path" ]]; then
    echo "expected lightweight wrapper to be a symlink to shared dispatcher: $wrapper_path" >&2
    exit 1
  fi

  manifest_path="$(bash "$DISPATCHER" --lane-wrapper "$wrapper_name" --resolve-manifest-path)"
  if [[ ! -f "$manifest_path" ]]; then
    echo "expected dispatcher to resolve existing manifest for $wrapper_name: $manifest_path" >&2
    exit 1
  fi
done

unknown_wrapper="run_missing_non_kolme_wave${WAVE_NUMBER}_lightweight_contract_lane.sh"
set +e
unknown_wrapper_output="$(
  bash "$DISPATCHER" --lane-wrapper "$unknown_wrapper" --resolve-manifest-path 2>&1
)"
unknown_wrapper_code=$?
set -e

if [[ "$unknown_wrapper_code" -eq 0 ]]; then
  echo "expected non-Kolme dispatcher to fail for unknown wave-${WAVE_NUMBER} lightweight wrapper" >&2
  exit 1
fi

if ! printf '%s\n' "$unknown_wrapper_output" | grep -q '^dispatch_status=fail$'; then
  echo "expected deterministic dispatcher fallback status marker for unknown wave-${WAVE_NUMBER} lightweight wrapper" >&2
  exit 1
fi
if ! printf '%s\n' "$unknown_wrapper_output" | grep -q '^fallback_reason_taxonomy_version=kamn.framework.non-kolme-dispatch-fallback-reason-taxonomy.v1$'; then
  echo "expected deterministic dispatcher fallback taxonomy marker for unknown wave-${WAVE_NUMBER} lightweight wrapper" >&2
  exit 1
fi
if ! printf '%s\n' "$unknown_wrapper_output" | grep -q '^fallback_reason_codes_csv=dispatcher_unknown_wrapper,dispatcher_manifest_missing,dispatcher_phase_unmapped$'; then
  echo "expected deterministic dispatcher fallback reason code set marker for unknown wave-${WAVE_NUMBER} lightweight wrapper" >&2
  exit 1
fi
if ! printf '%s\n' "$unknown_wrapper_output" | grep -q '^fallback_reason_code=dispatcher_unknown_wrapper$'; then
  echo "expected deterministic dispatcher fallback reason code marker for unknown wave-${WAVE_NUMBER} lightweight wrapper" >&2
  exit 1
fi

echo "non-Kolme wave-${WAVE_NUMBER} lightweight contract lane dispatcher wrapper matrix tests passed."
