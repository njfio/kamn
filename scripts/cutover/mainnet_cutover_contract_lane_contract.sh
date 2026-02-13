#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATOR="$ROOT_DIR/scripts/cutover/validate_mainnet_cutover_manifest.py"
VALID_FIXTURE="$ROOT_DIR/fixtures/mainnet_cutover/mainnet_cutover_manifest.valid.json"
INVALID_DEP_FIXTURE="$ROOT_DIR/fixtures/mainnet_cutover/mainnet_cutover_manifest.invalid_dependency.json"
INVALID_APPROVAL_FIXTURE="$ROOT_DIR/fixtures/mainnet_cutover/mainnet_cutover_manifest.invalid_approvals.json"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

for required_file in \
  "$VALIDATOR" \
  "$VALID_FIXTURE" \
  "$INVALID_DEP_FIXTURE" \
  "$INVALID_APPROVAL_FIXTURE"; do
  if [ ! -e "$required_file" ]; then
    echo "missing required cutover contract lane artifact: $required_file" >&2
    exit 1
  fi
done

if [ ! -x "$VALIDATOR" ]; then
  echo "mainnet cutover validator must be executable" >&2
  exit 1
fi

valid_output="$(
  python3 "$VALIDATOR" \
    --manifest "$VALID_FIXTURE" \
    --output-json "$TMP_DIR/valid-report.json"
)"
if ! printf '%s\n' "$valid_output" | grep -q "^validation_decision=GO$"; then
  echo "expected valid cutover manifest decision to be GO" >&2
  exit 1
fi

set +e
invalid_dep_output="$(
  python3 "$VALIDATOR" \
    --manifest "$INVALID_DEP_FIXTURE" \
    --output-json "$TMP_DIR/invalid-dependency-report.json" 2>&1
)"
invalid_dep_code=$?
set -e
if [ "$invalid_dep_code" -eq 0 ]; then
  echo "expected dependency regression fixture to fail validation" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_dep_output" | grep -q "unresolved dependency"; then
  echo "expected unresolved dependency regression error" >&2
  exit 1
fi

set +e
invalid_approval_output="$(
  python3 "$VALIDATOR" \
    --manifest "$INVALID_APPROVAL_FIXTURE" \
    --output-json "$TMP_DIR/invalid-approvals-report.json" 2>&1
)"
invalid_approval_code=$?
set -e
if [ "$invalid_approval_code" -eq 0 ]; then
  echo "expected approvals regression fixture to fail validation" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_approval_output" | grep -q "insufficient approvals"; then
  echo "expected insufficient approvals regression error" >&2
  exit 1
fi

echo "mainnet cutover contract lane tests passed."
