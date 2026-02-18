#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"
FALLBACK_REASON_TAXONOMY_VERSION="kamn.framework.non-kolme-dispatch-fallback-reason-taxonomy.v1"
FALLBACK_REASON_CODES_CSV="dispatcher_unknown_wrapper,dispatcher_manifest_missing,dispatcher_phase_unmapped"

if [ ! -x "$DISPATCHER" ]; then
  echo "expected non-Kolme dispatcher script to be executable: $DISPATCHER" >&2
  exit 1
fi

bash "$ROOT_DIR/scripts/framework/test_non_kolme_contract_lane_dispatch_wrapper_matrix.sh"
bash "$ROOT_DIR/scripts/framework/test_non_kolme_compliance_contract_lane_dispatch_wrapper_matrix.sh"
bash "$ROOT_DIR/scripts/framework/test_non_kolme_manifest_backed_contract_lane_dispatch_wrapper_matrix.sh"
bash "$ROOT_DIR/scripts/framework/test_non_kolme_bridge_contract_lane_dispatch_wrapper_matrix.sh"
bash "$ROOT_DIR/scripts/framework/test_non_kolme_sdk_contract_lane_dispatch_wrapper_matrix.sh"
bash "$ROOT_DIR/scripts/framework/test_non_kolme_lightweight_contract_lane_dispatch_wrapper_matrix.sh"

set +e
unknown_wrapper_output="$(
  bash "$DISPATCHER" \
    --lane-wrapper run_missing_legacy_entrypoint_compatibility_contract_lane.sh \
    --resolve-manifest-path 2>&1
)"
unknown_wrapper_status=$?
set -e

if [ "$unknown_wrapper_status" -eq 0 ]; then
  echo "expected unknown legacy wrapper probe to fail in dispatcher compatibility harness" >&2
  exit 1
fi

if ! printf '%s\n' "$unknown_wrapper_output" | grep -q "^fallback_reason_taxonomy_version=${FALLBACK_REASON_TAXONOMY_VERSION}$"; then
  echo "expected deterministic fallback reason taxonomy marker for unknown legacy wrapper probe" >&2
  exit 1
fi

if ! printf '%s\n' "$unknown_wrapper_output" | grep -q "^fallback_reason_codes_csv=${FALLBACK_REASON_CODES_CSV}$"; then
  echo "expected deterministic fallback reason code set marker for unknown legacy wrapper probe" >&2
  exit 1
fi

if ! printf '%s\n' "$unknown_wrapper_output" | grep -q '^fallback_reason_code=dispatcher_unknown_wrapper$'; then
  echo "expected deterministic dispatcher_unknown_wrapper reason code for unknown legacy wrapper probe" >&2
  exit 1
fi

echo "legacy_entrypoint_compatibility_status=pass"
echo "wrapper dispatch parity and legacy entrypoint compatibility tests passed."
