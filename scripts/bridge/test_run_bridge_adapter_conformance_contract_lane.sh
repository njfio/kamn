#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LANE_SCRIPT="$ROOT_DIR/scripts/bridge/run_bridge_adapter_conformance_contract_lane.sh"
LANE_IMPL_SCRIPT="$ROOT_DIR/scripts/bridge/run_bridge_adapter_conformance_contract_lane_impl.sh"
MATRIX_SCRIPT="$ROOT_DIR/scripts/bridge/run_bridge_adapter_conformance_matrix.py"
FIXTURE_FILE="$ROOT_DIR/fixtures/bridge_adapter_conformance/request_receipt_schema_cases.json"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/bridge_bridge_adapter_conformance_contract_lane.json"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"
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

if [ ! -x "$LANE_IMPL_SCRIPT" ]; then
  echo "expected bridge adapter conformance contract lane implementation script to be executable" >&2
  exit 1
fi

if [ ! -f "$MANIFEST_FILE" ]; then
  echo "expected bridge adapter conformance contract lane manifest to exist" >&2
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

if [ ! -L "$LANE_SCRIPT" ]; then
  echo "expected bridge adapter conformance contract lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi

if [ "$(readlink "$LANE_SCRIPT")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected bridge adapter conformance contract lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$LANE_SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected bridge adapter conformance wrapper to resolve bridge manifest via dispatcher" >&2
  exit 1
fi

if ! grep -Fq "run_bridge_adapter_conformance_contract_lane_impl.sh" "$MANIFEST_FILE"; then
  echo "expected bridge adapter conformance manifest to dispatch to implementation script" >&2
  exit 1
fi

if ! grep -Fq "run_bridge_adapter_conformance_matrix.py" "$LANE_IMPL_SCRIPT"; then
  echo "expected bridge adapter conformance implementation lane to invoke matrix runner" >&2
  exit 1
fi

if ! grep -Fq "bridge_adapter_conformance/request_receipt_schema_cases.json" "$LANE_IMPL_SCRIPT"; then
  echo "expected bridge adapter conformance implementation lane to use conformance fixture set" >&2
  exit 1
fi

echo "bridge adapter conformance contract lane script tests passed."
