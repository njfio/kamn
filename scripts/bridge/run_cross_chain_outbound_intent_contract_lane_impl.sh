#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
MATRIX_SCRIPT="$ROOT_DIR/scripts/bridge/run_cross_chain_outbound_intent_matrix.py"
FIXTURE_FILE="$ROOT_DIR/fixtures/bridge_outbound_intent/approval_retry_cases.json"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

cd "$ROOT_DIR"

if [ ! -x "$MATRIX_SCRIPT" ]; then
  echo "expected outbound intent matrix runner to be executable" >&2
  exit 1
fi

if [ ! -f "$FIXTURE_FILE" ]; then
  echo "expected outbound intent fixture file to exist" >&2
  exit 1
fi

start_epoch="$(date +%s)"
report_file="$TMP_DIR/bridge-outbound-intent-contract-report.json"

matrix_output="$(
  python3 "$MATRIX_SCRIPT" \
    --fixture "$FIXTURE_FILE" \
    --max-cases 2 \
    --output-json "$report_file"
)"
if ! printf '%s\n' "$matrix_output" | grep -q '^status=pass;'; then
  echo "expected outbound intent contract matrix to pass" >&2
  exit 1
fi

cargo test -p kamn-core --test cross_chain_bridge -- outbound_rejects_unauthorized_approver --exact >/dev/null
cargo test -p kamn-core --test cross_chain_receipt_finality >/dev/null
cargo test -p kamn-core --test cross_chain_bridge_adapters_docs >/dev/null

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt 120 ]; then
  echo "cross-chain outbound intent contract lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

echo "cross-chain outbound intent contract lane tests passed."
