#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_SCRIPT="$ROOT_DIR/scripts/canary/run_post_cutover_slo_contract_lane.sh"
DEEP_SCRIPT="$ROOT_DIR/scripts/canary/run_post_cutover_slo_deep_lane.sh"
SHARED_CONTRACT="$ROOT_DIR/scripts/canary/post_cutover_slo_contract_lane_contract.py"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/canary_post_cutover_slo_contract_lane.json"

if [ ! -x "$FAST_SCRIPT" ]; then
  echo "expected post-cutover SLO fast-lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$DEEP_SCRIPT" ]; then
  echo "expected post-cutover SLO deep-lane runner to be executable" >&2
  exit 1
fi
if [ ! -x "$SHARED_CONTRACT" ]; then
  echo "expected post-cutover SLO shared contract-lane module to be executable" >&2
  exit 1
fi
if [ ! -f "$MANIFEST" ]; then
  echo "expected post-cutover SLO contract-lane manifest to exist" >&2
  exit 1
fi

tmp_out="$(mktemp)"
trap 'rm -f "$tmp_out"' EXIT

bash "$FAST_SCRIPT" >"$tmp_out"
if ! grep -q "post-cutover SLO contract lane tests passed." "$tmp_out"; then
  echo "expected post-cutover SLO contract lane success marker" >&2
  exit 1
fi

if ! grep -q "run_manifest_lane.sh" "$FAST_SCRIPT"; then
  echo "expected post-cutover SLO fast-lane wrapper to delegate via manifest runner" >&2
  exit 1
fi
if ! grep -q "canary_post_cutover_slo_contract_lane.json" "$FAST_SCRIPT"; then
  echo "expected post-cutover SLO fast-lane wrapper to reference post-cutover SLO manifest" >&2
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

if ! grep -q "KAMN_POST_CUTOVER_SLO_MAX_SECONDS" "$SHARED_CONTRACT"; then
  echo "expected post-cutover SLO shared contract-lane module to enforce runtime budget env guard" >&2
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

echo "post-cutover SLO contract lane script tests passed."
