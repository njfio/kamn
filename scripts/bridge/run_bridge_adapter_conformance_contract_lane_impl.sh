#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
MATRIX_SCRIPT="$ROOT_DIR/scripts/bridge/run_bridge_adapter_conformance_matrix.py"
FIXTURE_FILE="$ROOT_DIR/fixtures/bridge_adapter_conformance/request_receipt_schema_cases.json"

output_json=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-json)
      output_json="${2:-}"
      shift 2
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if [ ! -x "$MATRIX_SCRIPT" ]; then
  echo "expected bridge adapter conformance matrix runner to be executable" >&2
  exit 1
fi

if [ ! -f "$FIXTURE_FILE" ]; then
  echo "expected bridge adapter conformance fixture file to exist" >&2
  exit 1
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

start_epoch="$(date +%s)"
report_file="$TMP_DIR/bridge-adapter-conformance-contract-report.json"

matrix_output="$(
  python3 "$MATRIX_SCRIPT" \
    --fixture "$FIXTURE_FILE" \
    --max-cases 3 \
    --output-json "$report_file"
)"
if ! printf '%s\n' "$matrix_output" | grep -q '^status=pass;'; then
  echo "expected bridge adapter conformance contract matrix to pass" >&2
  exit 1
fi

cargo test -p kamn-core --test bridge_adapter -- regression_rejects_adapter_outbound_request_id_mutation --exact >/dev/null
cargo test -p kamn-core --test cross_chain_bridge_adapters_docs >/dev/null

if [[ -n "$output_json" ]]; then
  cp "$report_file" "$output_json"
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt 120 ]; then
  echo "bridge adapter conformance contract lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

echo "bridge adapter conformance contract lane tests passed."
