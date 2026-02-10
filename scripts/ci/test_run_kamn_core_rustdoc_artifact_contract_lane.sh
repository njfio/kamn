#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LANE_SCRIPT="$ROOT_DIR/scripts/ci/run_kamn_core_rustdoc_artifact_contract_lane.sh"
POLICY_SCRIPT="$ROOT_DIR/scripts/ci/check_kamn_core_rustdoc_artifact_policy.sh"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$LANE_SCRIPT" ]; then
  echo "expected rustdoc artifact contract lane script to be executable" >&2
  exit 1
fi

if [ ! -x "$POLICY_SCRIPT" ]; then
  echo "expected rustdoc artifact policy checker script to be executable" >&2
  exit 1
fi

REPORT_FILE="$TMP_DIR/rustdoc-report.json"
ARTIFACT_DIR="$TMP_DIR/artifacts"

lane_output="$(
  bash "$LANE_SCRIPT" \
    --output-json "$REPORT_FILE" \
    --artifact-dir "$ARTIFACT_DIR" \
    --max-runtime-seconds 600
)"

if ! printf '%s\n' "$lane_output" | grep -q '^kamn_core_rustdoc_artifact_status=pass$'; then
  echo "expected rustdoc artifact lane pass status marker" >&2
  exit 1
fi

if [ ! -f "$REPORT_FILE" ]; then
  echo "expected rustdoc artifact report file" >&2
  exit 1
fi

if ! grep -q '"schema_version": "kamn.ci.kamn-core-rustdoc-artifact-report.v1"' "$REPORT_FILE"; then
  echo "expected rustdoc artifact report schema version marker" >&2
  exit 1
fi

if ! grep -q '"reason_key": "kamn.ci.kamn-core-rustdoc-artifact.ok"' "$REPORT_FILE"; then
  echo "expected rustdoc artifact report success reason key" >&2
  exit 1
fi

bash "$POLICY_SCRIPT" --report-file "$REPORT_FILE" >"$TMP_DIR/policy.out"
grep -q '^kamn_core_rustdoc_artifact_policy=ok$' "$TMP_DIR/policy.out"

echo "kamn-core rustdoc artifact contract lane script tests passed."
