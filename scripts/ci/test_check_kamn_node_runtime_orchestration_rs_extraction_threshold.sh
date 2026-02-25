#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

bash "$ROOT_DIR/scripts/ci/test_check_kamn_node_extraction_threshold_common.sh" \
  "$ROOT_DIR/scripts/ci/check_kamn_node_runtime_orchestration_rs_extraction_threshold.sh" \
  "$ROOT_DIR/fixtures/ci/kamn_node_runtime_orchestration_rs_extraction_thresholds.json" \
  "kamn-node runtime_orchestration.rs" \
  "kamn.ci.kamn-node-runtime-orchestration-rs-extraction-thresholds.v1" \
  "kamn.ci.kamn-node-runtime-orchestration-rs-extraction-threshold-exception.v1" \
  "runtime_orchestration_rs_line_count_warn_threshold_exceeded" \
  "runtime_orchestration_rs_line_count_fail_threshold_exceeded" \
  "runtime_orchestration_rs_threshold_exception_applied" \
  "runtime_orchestration_rs_threshold_exception_expired" \
  "#3733" \
  "kamn-node runtime_orchestration.rs extraction-threshold checker tests passed."
