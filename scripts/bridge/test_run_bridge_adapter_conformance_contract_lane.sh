#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LANE_SCRIPT="$ROOT_DIR/scripts/bridge/run_bridge_adapter_conformance_contract_lane.sh"
MATRIX_SCRIPT="$ROOT_DIR/scripts/bridge/run_bridge_adapter_conformance_matrix.py"
FIXTURE_FILE="$ROOT_DIR/fixtures/bridge_adapter_conformance/request_receipt_schema_cases.json"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$LANE_SCRIPT" ]; then
  echo "expected bridge adapter conformance contract lane script to be executable" >&2
  exit 1
fi

if [ ! -x "$MATRIX_SCRIPT" ]; then
  echo "expected bridge adapter conformance matrix script to be executable" >&2
  exit 1
fi

if [ ! -f "$FIXTURE_FILE" ]; then
  echo "expected bridge adapter conformance fixture file to exist" >&2
  exit 1
fi

report_file="$TMP_DIR/bridge-adapter-conformance-contract-report.json"
lane_output="$(bash "$LANE_SCRIPT" --output-json "$report_file")"
if ! printf '%s\n' "$lane_output" | grep -q "bridge adapter conformance contract lane tests passed."; then
  echo "expected bridge adapter conformance contract lane success marker" >&2
  exit 1
fi

if [ ! -f "$report_file" ]; then
  echo "expected bridge adapter conformance contract lane to emit report artifact" >&2
  exit 1
fi

if ! grep -q '"status": "pass"' "$report_file"; then
  echo "expected bridge adapter conformance contract report to pass" >&2
  exit 1
fi

if ! grep -q '"schema_version": "kamn.bridge.adapter-conformance.matrix-report.v1"' "$report_file"; then
  echo "expected bridge adapter conformance contract report schema marker" >&2
  exit 1
fi

if ! grep -Fq "run_bridge_adapter_conformance_matrix.py" "$LANE_SCRIPT"; then
  echo "expected bridge adapter conformance contract lane to invoke matrix runner" >&2
  exit 1
fi

if ! grep -Fq "bridge_adapter_conformance/request_receipt_schema_cases.json" "$LANE_SCRIPT"; then
  echo "expected bridge adapter conformance contract lane to use conformance fixture set" >&2
  exit 1
fi

echo "bridge adapter conformance contract lane script tests passed."
