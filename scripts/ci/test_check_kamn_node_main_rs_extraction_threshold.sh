#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

bash "$ROOT_DIR/scripts/ci/test_check_kamn_node_extraction_threshold_common.sh" \
  "$ROOT_DIR/scripts/ci/check_kamn_node_main_rs_extraction_threshold.sh" \
  "$ROOT_DIR/fixtures/ci/kamn_node_main_rs_extraction_thresholds.json" \
  "kamn-node main.rs" \
  "kamn.ci.kamn-node-main-rs-extraction-thresholds.v1" \
  "kamn.ci.kamn-node-main-rs-extraction-threshold-exception.v1" \
  "main_rs_line_count_warn_threshold_exceeded" \
  "main_rs_line_count_fail_threshold_exceeded" \
  "main_rs_threshold_exception_applied" \
  "main_rs_threshold_exception_expired" \
  "#3261" \
  "kamn-node main.rs extraction-threshold checker tests passed."
