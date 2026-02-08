#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_SCRIPT="$ROOT_DIR/scripts/sdk/run_live_transport_parity_contract_lane.sh"
DEEP_SCRIPT="$ROOT_DIR/scripts/sdk/run_live_transport_parity_deep_lane.sh"
PROFILE_DRIFT_SCRIPT="$ROOT_DIR/scripts/sdk/run_transport_profile_parity_matrix.sh"

if [ ! -x "$FAST_SCRIPT" ]; then
  echo "expected live transport parity fast-lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$DEEP_SCRIPT" ]; then
  echo "expected live transport parity deep-lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$PROFILE_DRIFT_SCRIPT" ]; then
  echo "expected transport profile parity drift matrix runner to be executable" >&2
  exit 1
fi

TMP_OUT="$(mktemp)"
trap 'rm -f "$TMP_OUT"' EXIT

bash "$FAST_SCRIPT" --languages python,typescript >"$TMP_OUT"
if ! grep -q "live transport parity contract lane tests passed for languages: python,typescript." "$TMP_OUT"; then
  echo "expected live transport parity contract lane success marker" >&2
  exit 1
fi

if ! grep -q "running python live transport contract lane tests" "$TMP_OUT"; then
  echo "expected parity fast lane to run python subset tests" >&2
  exit 1
fi

if ! grep -q "running typescript live transport contract lane tests" "$TMP_OUT"; then
  echo "expected parity fast lane to run typescript subset tests" >&2
  exit 1
fi

if grep -q "running rust live transport contract lane tests" "$TMP_OUT"; then
  echo "expected parity fast lane python/typescript subset to skip rust tests" >&2
  exit 1
fi

if ! grep -q 'run_transport_profile_parity_matrix.sh" --languages "\$SELECTED_LANGUAGES"' "$FAST_SCRIPT"; then
  echo "expected parity fast lane to run transport profile parity drift matrix for selected languages" >&2
  exit 1
fi

if ! grep -Fq "test_sdk_live_transport_deep.py" "$DEEP_SCRIPT"; then
  echo "expected deep-lane script to execute python deep-lane parity test" >&2
  exit 1
fi

if ! grep -Fq "live_transport_client.deep.ts" "$DEEP_SCRIPT"; then
  echo "expected deep-lane script to execute typescript deep-lane parity test" >&2
  exit 1
fi

echo "live transport parity contract lane script tests passed."
