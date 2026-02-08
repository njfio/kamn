#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_SCRIPT="$ROOT_DIR/scripts/bridge/run_bridge_credentialed_contract_lane.sh"
DEEP_SCRIPT="$ROOT_DIR/scripts/bridge/run_bridge_credentialed_deep_lane.sh"

if [ ! -x "$FAST_SCRIPT" ]; then
  echo "expected bridge credentialed fast-lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$DEEP_SCRIPT" ]; then
  echo "expected bridge credentialed deep-lane runner to be executable" >&2
  exit 1
fi

TMP_OUT="$(mktemp)"
trap 'rm -f "$TMP_OUT"' EXIT

bash "$FAST_SCRIPT" >"$TMP_OUT"
if ! grep -q "bridge credentialed contract lane tests passed." "$TMP_OUT"; then
  echo "expected bridge credentialed contract lane success marker" >&2
  exit 1
fi

if ! grep -Fq "run_bridge_credentialed_contract_lane.sh" "$DEEP_SCRIPT"; then
  echo "expected deep-lane script to execute fast-lane credential checks first" >&2
  exit 1
fi

if ! grep -q "bridge-credential-redaction-report.json" "$DEEP_SCRIPT"; then
  echo "expected deep-lane script to emit bridge credential redaction report" >&2
  exit 1
fi

echo "bridge credentialed contract lane script tests passed."
