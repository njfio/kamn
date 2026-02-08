#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_SCRIPT="$ROOT_DIR/scripts/bridge/run_telegram_ingress_contract_lane.sh"
DEEP_SCRIPT="$ROOT_DIR/scripts/bridge/run_telegram_ingress_deep_lane.sh"

if [ ! -x "$FAST_SCRIPT" ]; then
  echo "expected telegram ingress fast-lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$DEEP_SCRIPT" ]; then
  echo "expected telegram ingress deep-lane runner to be executable" >&2
  exit 1
fi

TMP_OUT="$(mktemp)"
trap 'rm -f "$TMP_OUT"' EXIT

bash "$FAST_SCRIPT" --skip-replay >"$TMP_OUT"
if ! grep -q "telegram ingress contract lane tests passed." "$TMP_OUT"; then
  echo "expected telegram ingress contract lane success marker" >&2
  exit 1
fi

if ! grep -Fq "run_telegram_ingress_contract_lane.sh" "$DEEP_SCRIPT"; then
  echo "expected deep-lane script to execute fast-lane telegram checks first" >&2
  exit 1
fi

if ! grep -q "telegram-ingress-report.json" "$DEEP_SCRIPT"; then
  echo "expected deep-lane script to emit telegram ingress report artifact" >&2
  exit 1
fi

echo "telegram ingress contract lane script tests passed."
