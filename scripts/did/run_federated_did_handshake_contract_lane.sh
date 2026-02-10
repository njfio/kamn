#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
MATRIX_SCRIPT="$ROOT_DIR/scripts/did/run_federated_did_handshake_matrix.py"
FIXTURE_FILE="$ROOT_DIR/fixtures/federated_did_handshake/partition_replay_cases.json"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

cd "$ROOT_DIR"

if [ ! -x "$MATRIX_SCRIPT" ]; then
  echo "expected federated DID handshake matrix runner to be executable" >&2
  exit 1
fi

if [ ! -f "$FIXTURE_FILE" ]; then
  echo "expected federated DID handshake fixture file to exist" >&2
  exit 1
fi

start_epoch="$(date +%s)"
report_file="$TMP_DIR/federated-did-handshake-contract-report.json"

matrix_output="$(
  python3 "$MATRIX_SCRIPT" \
    --fixture "$FIXTURE_FILE" \
    --max-cases 2 \
    --output-json "$report_file"
)"
if ! printf '%s\n' "$matrix_output" | grep -q '^status=pass;'; then
  echo "expected federated DID handshake contract matrix to pass" >&2
  exit 1
fi

cargo test -p kamn-core --test did_method >/dev/null
bash scripts/ci/run_cargo_test_with_quarantine.sh -- cargo test -p kamn-core --test federated_did_handshake_runtime >/dev/null
bash scripts/ci/run_cargo_test_with_quarantine.sh -- cargo test -p kamn-core --test did_method_docs regression_requires_federated_runtime_trust_store_guard_marker -- --exact >/dev/null
bash scripts/ci/run_cargo_test_with_quarantine.sh -- cargo test -p kamn-core --test release_gonogo_checklist_docs regression_requires_federated_runtime_trust_store_guard_marker -- --exact >/dev/null

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt 120 ]; then
  echo "federated DID handshake contract lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

echo "federated DID handshake contract lane tests passed."
