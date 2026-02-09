#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DEMO_SCRIPT="$ROOT_DIR/scripts/sdk/run_localhost_signed_demo.sh"

if [ ! -x "$DEMO_SCRIPT" ]; then
  echo "expected localhost signed demo runner to be executable" >&2
  exit 1
fi

TMP_OUT="$(mktemp)"
TMP_HELP="$(mktemp)"
TMP_ERR="$(mktemp)"
trap 'rm -f "$TMP_OUT" "$TMP_HELP" "$TMP_ERR"' EXIT

bash "$DEMO_SCRIPT" --help >"$TMP_HELP"

if ! grep -Fq -- "Usage: run_localhost_signed_demo.sh" "$TMP_HELP"; then
  echo "expected localhost signed demo help usage banner" >&2
  exit 1
fi

if ! grep -Fq -- "--timeout-seconds" "$TMP_HELP"; then
  echo "expected localhost signed demo help to document --timeout-seconds" >&2
  exit 1
fi

set +e
bash "$DEMO_SCRIPT" --timeout-seconds 0 >"$TMP_ERR" 2>&1
error_code=$?
set -e

if [ "$error_code" -eq 0 ]; then
  echo "expected localhost signed demo script to reject invalid timeout argument" >&2
  exit 1
fi

# Regression: #875
if ! grep -Fq -- "timeout-seconds must be a positive integer" "$TMP_ERR"; then
  echo "expected explicit timeout validation failure message" >&2
  exit 1
fi

bash "$DEMO_SCRIPT" >"$TMP_OUT"

required_markers=(
  "--- sender ---"
  "--- listener ---"
  "status=ok"
  "verified=true"
  "signature=sig:ed25519:baseline-v1:"
  "localhost signed message demo completed."
)

for marker in "${required_markers[@]}"; do
  if ! grep -Fq -- "$marker" "$TMP_OUT"; then
    echo "expected localhost signed demo output marker '$marker'" >&2
    exit 1
  fi
done

echo "localhost signed demo script tests passed."
