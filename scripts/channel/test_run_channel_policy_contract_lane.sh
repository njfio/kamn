#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/channel/run_channel_policy_contract_lane.sh"

if [ ! -x "$SCRIPT" ]; then
  echo "expected channel policy contract lane script to be executable" >&2
  exit 1
fi

TMP_OUT="$(mktemp)"
trap 'rm -f "$TMP_OUT"' EXIT

bash "$SCRIPT" >"$TMP_OUT"

if ! grep -q "channel policy contract lane tests passed." "$TMP_OUT"; then
  echo "expected channel policy contract lane success marker" >&2
  exit 1
fi

if ! grep -Fq "run_channel_retention_redaction_contract_lane.sh" "$SCRIPT"; then
  echo "expected channel policy lane to execute retention/redaction contract lane checks" >&2
  exit 1
fi

echo "channel policy contract lane script tests passed."
