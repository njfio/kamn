#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"
FAST_SCRIPT="$ROOT_DIR/scripts/canary/run_post_cutover_slo_contract_lane.sh"
DEEP_SCRIPT="$ROOT_DIR/scripts/canary/run_post_cutover_slo_deep_lane.sh"
SHARED_CONTRACT="$ROOT_DIR/scripts/canary/post_cutover_slo_contract_lane_contract.py"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/canary_post_cutover_slo_contract_lane.json"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"

test_harness_require_executable "$FAST_SCRIPT" "expected post-cutover SLO fast-lane runner to be executable"

test_harness_require_executable "$DEEP_SCRIPT" "expected post-cutover SLO deep-lane runner to be executable"
test_harness_require_executable "$SHARED_CONTRACT" "expected post-cutover SLO shared contract-lane module to be executable"
test_harness_require_file "$MANIFEST" "expected post-cutover SLO contract-lane manifest to exist"

tmp_out="$(mktemp)"
trap 'rm -f "$tmp_out"' EXIT

bash "$FAST_SCRIPT" >"$tmp_out"
if ! grep -q "post-cutover SLO contract lane tests passed." "$tmp_out"; then
  echo "expected post-cutover SLO contract lane success marker" >&2
  exit 1
fi

if [ ! -L "$FAST_SCRIPT" ]; then
  echo "expected post-cutover SLO fast-lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi
if [ "$(readlink "$FAST_SCRIPT")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected post-cutover SLO fast-lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi
resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$FAST_SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST" ]; then
  echo "expected post-cutover SLO fast-lane wrapper to resolve post-cutover manifest via dispatcher" >&2
  exit 1
fi
if ! grep -q "post_cutover_slo_contract_lane_contract.py" "$MANIFEST"; then
  echo "expected post-cutover SLO manifest to dispatch to shared contract module" >&2
  exit 1
fi

if ! grep -q "alerts.alert_keys mismatch" "$SHARED_CONTRACT"; then
  echo "expected post-cutover SLO shared contract-lane module to enforce alert-key drift failures" >&2
  exit 1
fi
if ! grep -q "burn_rate_parity_status mismatch" "$ROOT_DIR/scripts/canary/post_cutover_slo_contract.py"; then
  echo "expected post-cutover SLO policy checker to enforce burn-rate parity drift failures" >&2
  exit 1
fi
if ! grep -q "alert_governance_reason_taxonomy_version mismatch" "$ROOT_DIR/scripts/canary/post_cutover_slo_contract.py"; then
  echo "expected post-cutover SLO policy checker to enforce alert-governance taxonomy drift failures" >&2
  exit 1
fi

if ! grep -q "KAMN_POST_CUTOVER_SLO_MAX_SECONDS" "$SHARED_CONTRACT"; then
  echo "expected post-cutover SLO shared contract-lane module to enforce runtime budget env guard" >&2
  exit 1
fi
if ! grep -q "KAMN_POST_CUTOVER_SLO_CI_LOCAL_PROMOTION_MAX_SECONDS" "$SHARED_CONTRACT"; then
  echo "expected post-cutover SLO shared contract-lane module to enforce ci-local promotion budget env guard" >&2
  exit 1
fi

if ! grep -Fq "run_post_cutover_slo_contract_lane.sh" "$DEEP_SCRIPT"; then
  echo "expected SLO deep-lane script to execute fast-lane checks first" >&2
  exit 1
fi

if ! grep -q "final_decision=NO-GO" "$DEEP_SCRIPT"; then
  echo "expected SLO deep-lane script to validate NO-GO decision path" >&2
  exit 1
fi

if ! grep -q "slo_alert_reason_codes:NO-GO:v1" "$DEEP_SCRIPT"; then
  echo "expected SLO deep-lane script to enforce NO-GO reason-key marker" >&2
  exit 1
fi

if ! grep -q "KAMN_POST_CUTOVER_SLO_DEEP_MAX_SECONDS" "$DEEP_SCRIPT"; then
  echo "expected SLO deep-lane script to enforce deep runtime budget env guard" >&2
  exit 1
fi
if ! grep -q "KAMN_POST_CUTOVER_SLO_DEEP_LOCAL_ONLY" "$DEEP_SCRIPT"; then
  echo "expected SLO deep-lane script to enforce local-only opt-in guard" >&2
  exit 1
fi

echo "post-cutover SLO contract lane script tests passed."
