#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER_PATH="$ROOT_DIR/scripts/framework/test_non_kolme_wave_lightweight_contract_lane_dispatch_wrapper_matrix.sh"
DEFINITIONS_DIR="$ROOT_DIR/scripts/framework/wave_definitions"

if [ ! -x "$RUNNER_PATH" ]; then
  echo "expected shared non-Kolme wave lightweight wrapper matrix runner: $RUNNER_PATH" >&2
  exit 1
fi

for wave in {10..19}; do
  wave_entrypoint="$ROOT_DIR/scripts/framework/test_non_kolme_wave${wave}_lightweight_contract_lane_dispatch_wrapper_matrix.sh"
  definitions_file="$DEFINITIONS_DIR/non_kolme_wave${wave}_lightweight_wrappers.txt"

  if [ -e "$wave_entrypoint" ]; then
    echo "expected legacy wave-specific wrapper to be removed after shared-runner migration: $wave_entrypoint" >&2
    exit 1
  fi

  if ! bash "$RUNNER_PATH" --wave "$wave" >/dev/null; then
    echo "expected shared wave runner to execute successfully for wave $wave" >&2
    exit 1
  fi

  if [ ! -f "$definitions_file" ]; then
    echo "expected wave wrapper definition file: $definitions_file" >&2
    exit 1
  fi

  if ! grep -Eq '^[[:space:]]*scripts/.+\.sh[[:space:]]*$' "$definitions_file"; then
    echo "expected at least one wrapper entry in $definitions_file" >&2
    exit 1
  fi
done

echo "non-Kolme wave lightweight wrapper runner contract tests passed."
