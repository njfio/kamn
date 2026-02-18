#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"
LEGACY_LANE_SCRIPT="$ROOT_DIR/scripts/ci/run_kamn_core_rustdoc_artifact_contract_lane.sh"
MANIFEST_RUNNER="$ROOT_DIR/scripts/framework/run_manifest_lane.sh"
SHARED_IMPL="$ROOT_DIR/scripts/ci/kamn_core_rustdoc_artifact_contract_lane_impl.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/ci_kamn_core_rustdoc_artifact_contract_lane.json"
POLICY_SCRIPT="$ROOT_DIR/scripts/ci/check_kamn_core_rustdoc_artifact_policy.sh"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ -e "$LEGACY_LANE_SCRIPT" ]; then
  echo "expected superseded rustdoc artifact wrapper to be deleted: $LEGACY_LANE_SCRIPT" >&2
  exit 1
fi

test_harness_require_executable "$MANIFEST_RUNNER" "expected manifest lane runner to be executable"

test_harness_require_executable "$SHARED_IMPL" "expected rustdoc artifact shared impl script to be executable"

test_harness_require_executable "$POLICY_SCRIPT" "expected rustdoc artifact policy checker script to be executable"

REPORT_FILE="$TMP_DIR/rustdoc-report.json"
ARTIFACT_DIR="$TMP_DIR/artifacts"

lane_output="$(
  bash "$MANIFEST_RUNNER" \
    --manifest "$MANIFEST_FILE" \
    --phase contract \
    -- \
    --output-json "$REPORT_FILE" \
    --artifact-dir "$ARTIFACT_DIR" \
    --max-runtime-seconds 600
)"

if ! printf '%s\n' "$lane_output" | grep -q '^kamn_core_rustdoc_artifact_status=pass$'; then
  echo "expected rustdoc artifact lane pass status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^rustdoc_navigation_ratio_status=within$'; then
  echo "expected rustdoc navigation ratio status marker to remain within on default lane run" >&2
  exit 1
fi

test_harness_require_file "$REPORT_FILE" "expected rustdoc artifact report file"

if ! grep -q '"schema_version": "kamn.ci.kamn-core-rustdoc-artifact-report.v1"' "$REPORT_FILE"; then
  echo "expected rustdoc artifact report schema version marker" >&2
  exit 1
fi

if ! grep -q '"reason_key": "kamn.ci.kamn-core-rustdoc-artifact.ok"' "$REPORT_FILE"; then
  echo "expected rustdoc artifact report success reason key" >&2
  exit 1
fi
if ! grep -q '"docs_contract_test_count":' "$REPORT_FILE"; then
  echo "expected rustdoc artifact report docs_contract_test_count marker" >&2
  exit 1
fi
if ! grep -q '"behavioral_test_count":' "$REPORT_FILE"; then
  echo "expected rustdoc artifact report behavioral_test_count marker" >&2
  exit 1
fi
if ! grep -q '"docs_contract_to_behavioral_ratio":' "$REPORT_FILE"; then
  echo "expected rustdoc artifact report docs_contract_to_behavioral_ratio marker" >&2
  exit 1
fi
if ! grep -q '"max_docs_contract_to_behavioral_ratio":' "$REPORT_FILE"; then
  echo "expected rustdoc artifact report max_docs_contract_to_behavioral_ratio marker" >&2
  exit 1
fi
if ! grep -q '"rustdoc_navigation_ratio_status": "within"' "$REPORT_FILE"; then
  echo "expected rustdoc artifact report ratio status marker to be within on default lane run" >&2
  exit 1
fi

bash "$POLICY_SCRIPT" --report-file "$REPORT_FILE" >"$TMP_DIR/policy.out"
grep -q '^kamn_core_rustdoc_artifact_policy=ok$' "$TMP_DIR/policy.out"
grep -q '^rustdoc_navigation_ratio_status=within$' "$TMP_DIR/policy.out"
grep -q '^runtime_budget_status=within$' "$TMP_DIR/policy.out"

if ! grep -Fq "kamn_core_rustdoc_artifact_contract_lane_impl.sh" "$MANIFEST_FILE"; then
  echo "expected rustdoc artifact manifest to dispatch shared impl script" >&2
  exit 1
fi

echo "kamn-core rustdoc artifact contract lane script tests passed."
