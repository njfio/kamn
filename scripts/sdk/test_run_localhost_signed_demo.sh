#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DEMO_SCRIPT="$ROOT_DIR/scripts/sdk/run_localhost_signed_demo.sh"

if [ ! -x "$DEMO_SCRIPT" ]; then
  echo "expected localhost signed demo runner to be executable" >&2
  exit 1
fi

TMP_OUT="$(mktemp)"
trap 'rm -f "$TMP_OUT"' EXIT

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
