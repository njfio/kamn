#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

bash "$ROOT_DIR/scripts/ci/test_check_kamn_node_extraction_threshold_common.sh" \
  "$ROOT_DIR/scripts/ci/check_kamn_sdk_service_rs_extraction_threshold.sh" \
  "$ROOT_DIR/fixtures/ci/kamn_sdk_service_rs_extraction_thresholds.json" \
  "kamn-sdk service.rs" \
  "kamn.ci.kamn-sdk-service-rs-extraction-thresholds.v1" \
  "kamn.ci.kamn-sdk-service-rs-extraction-threshold-exception.v1" \
  "service_rs_line_count_warn_threshold_exceeded" \
  "service_rs_line_count_fail_threshold_exceeded" \
  "service_rs_threshold_exception_applied" \
  "service_rs_threshold_exception_expired" \
  "#6303" \
  "kamn-sdk service.rs extraction-threshold checker tests passed."
