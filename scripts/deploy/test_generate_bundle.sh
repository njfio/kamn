#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/deploy/generate_bundle.sh"
PREFLIGHT="$ROOT_DIR/scripts/deploy/preflight_topology.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

BUNDLE_FILE="$TMP_DIR/topology.bundle.env"

generated_output="$(
  bash "$GENERATOR" \
    --output-file "$BUNDLE_FILE" \
    --processors 3 \
    --listeners 3 \
    --approvers 3 \
    --required-approvals 2
)"

if ! printf '%s\n' "$generated_output" | grep -q "status=generated"; then
  echo "expected generated status output from bundle generator" >&2
  exit 1
fi

if [ ! -f "$BUNDLE_FILE" ]; then
  echo "expected generated bundle file to exist" >&2
  exit 1
fi

if ! grep -q "^PROCESSORS=3$" "$BUNDLE_FILE"; then
  echo "expected generated bundle to include processor count" >&2
  exit 1
fi

bundle_preflight_output="$(bash "$PREFLIGHT" --bundle-file "$BUNDLE_FILE")"
if ! printf '%s\n' "$bundle_preflight_output" | grep -q "^status=ok$"; then
  echo "expected generated bundle to pass preflight validation" >&2
  exit 1
fi

set +e
invalid_output="$(
  bash "$GENERATOR" \
    --output-file "$TMP_DIR/invalid.bundle.env" \
    --processors 1 \
    --listeners 1 \
    --approvers 2 \
    --required-approvals 3 2>&1
)"
invalid_code=$?
set -e

if [ "$invalid_code" -eq 0 ]; then
  echo "expected generator to fail invalid quorum topology" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_output" | grep -q "required-approvals must be between 1 and approvers"; then
  echo "expected quorum validation error to bubble from preflight" >&2
  exit 1
fi

echo "deployment bundle generator tests passed."
