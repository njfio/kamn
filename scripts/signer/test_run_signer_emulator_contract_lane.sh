#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_SCRIPT="$ROOT_DIR/scripts/signer/run_signer_emulator_contract_lane.sh"
DEEP_SCRIPT="$ROOT_DIR/scripts/signer/run_signer_provider_deep_lane.sh"

if [ ! -x "$FAST_SCRIPT" ]; then
  echo "expected signer emulator fast-lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$DEEP_SCRIPT" ]; then
  echo "expected signer provider deep-lane runner to be executable" >&2
  exit 1
fi

TMP_OUT="$(mktemp)"
trap 'rm -f "$TMP_OUT"' EXIT

bash "$FAST_SCRIPT" >"$TMP_OUT"
if ! grep -q "signer emulator contract lane tests passed." "$TMP_OUT"; then
  echo "expected signer emulator contract lane success marker" >&2
  exit 1
fi

if ! grep -q "functional_provider_handshake_matrix_routes_operator_fallback_for_unavailable_provider" "$FAST_SCRIPT"; then
  echo "expected signer emulator contract lane to include provider handshake fallback functional coverage" >&2
  exit 1
fi

if ! grep -q "regression_provider_handshake_policy_block_rejects_without_fallback" "$FAST_SCRIPT"; then
  echo "expected signer emulator contract lane to include provider handshake policy-block regression coverage" >&2
  exit 1
fi

if ! grep -Fq "performance_signer_emulator_bulk_signing_deep_lane -- --ignored" "$DEEP_SCRIPT"; then
  echo "expected deep-lane script to execute ignored signer provider stress test" >&2
  exit 1
fi

echo "signer emulator contract lane script tests passed."
