#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/sdk/run_transport_profile_parity_matrix.sh"
BACKEND_ADAPTER_FIXTURE="$ROOT_DIR/fixtures/sdk_parity/live_backend_adapter_profile_expectations.json"

if [ ! -x "$SCRIPT" ]; then
  echo "expected transport profile parity matrix runner to be executable" >&2
  exit 1
fi

if [ ! -f "$BACKEND_ADAPTER_FIXTURE" ]; then
  echo "expected backend adapter parity fixture to exist" >&2
  exit 1
fi

if ! grep -q '"backend_adapter_error_reason"' "$BACKEND_ADAPTER_FIXTURE"; then
  echo "expected backend adapter parity fixture to define reason-code contract key" >&2
  exit 1
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

PASS_REPORT="$TMP_DIR/pass-report.json"
bash "$SCRIPT" --output-json "$PASS_REPORT" >"$TMP_DIR/pass.out"
if ! grep -q '"status": "pass"' "$PASS_REPORT"; then
  echo "expected transport profile parity matrix pass report" >&2
  exit 1
fi

if ! grep -q "status=pass" "$TMP_DIR/pass.out"; then
  echo "expected transport profile parity matrix pass status output" >&2
  exit 1
fi

SUBSET_REPORT="$TMP_DIR/subset-report.json"
bash "$SCRIPT" --languages python,typescript --output-json "$SUBSET_REPORT" >"$TMP_DIR/subset.out"
if ! grep -q '"status": "pass"' "$SUBSET_REPORT"; then
  echo "expected python/typescript transport profile parity subset to pass" >&2
  exit 1
fi

if ! grep -q '"backend_adapter_error_reason": "backend_timeout"' "$SUBSET_REPORT"; then
  echo "expected adapter reason-code parity evidence in subset report" >&2
  exit 1
fi

set +e
bash "$SCRIPT" --expect-default-mode live >"$TMP_DIR/fail.out" 2>&1
fail_code=$?
set -e

if [ "$fail_code" -eq 0 ]; then
  echo "expected transport profile parity matrix to fail for mismatched expected mode" >&2
  exit 1
fi

# Regression: #679
if ! grep -q "status=fail" "$TMP_DIR/fail.out"; then
  echo "expected transport profile parity matrix mismatch to emit fail status" >&2
  exit 1
fi

set +e
bash "$SCRIPT" --languages python,typescript --expect-adapter-error-reason policy_denied >"$TMP_DIR/fail-adapter.out" 2>&1
adapter_fail_code=$?
set -e

if [ "$adapter_fail_code" -eq 0 ]; then
  echo "expected transport profile parity matrix to fail for adapter reason-code drift" >&2
  exit 1
fi

if ! grep -q "backend_adapter_error_reason" "$TMP_DIR/fail-adapter.out"; then
  echo "expected adapter reason-code drift failure details in output" >&2
  exit 1
fi

echo "transport profile parity matrix tests passed."
