#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PREFLIGHT_LANE="$ROOT_DIR/scripts/runtime/run_failover_sync_drill_preflight_contract_lane.sh"
SHARED_CONTRACT="$ROOT_DIR/scripts/runtime/failover_sync_drill_preflight_contract_lane_contract.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/runtime_failover_sync_drill_preflight_contract_lane.json"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$PREFLIGHT_LANE" ]; then
  echo "expected failover/sync preflight contract lane script to be executable" >&2
  exit 1
fi

if [ ! -x "$SHARED_CONTRACT" ]; then
  echo "expected failover/sync preflight shared contract module to be executable" >&2
  exit 1
fi

lane_output="$(bash "$PREFLIGHT_LANE" --output-json "$TMP_REPORT")"
if ! printf '%s\n' "$lane_output" | grep -q "failover/sync preflight contract lane tests passed."; then
  echo "expected failover/sync preflight contract lane success marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text())
if payload.get("schema_version") != "kamn.runtime.failover-sync-drill-report.v1":
    raise SystemExit("unexpected failover/sync preflight report schema")
if payload.get("lane") != "preflight":
    raise SystemExit("expected preflight lane report")
if payload.get("status") != "pass":
    raise SystemExit("expected preflight lane to pass")
if payload.get("failover_promotion_gate_status") != "verified":
    raise SystemExit("expected failover_promotion_gate_status=verified")
if payload.get("live_node_drift_parity_status") != "verified":
    raise SystemExit("expected live_node_drift_parity_status=verified")
if payload.get("ci_local_promotion_budget_boundary_status") != "verified":
    raise SystemExit("expected ci_local_promotion_budget_boundary_status=verified")
if payload.get("failover_readiness_reason_taxonomy_version") != "kamn.runtime.failover-readiness-reason-taxonomy.v1":
    raise SystemExit("expected deterministic failover_readiness_reason_taxonomy_version marker")
if payload.get("failover_readiness_reason_codes_csv") != "failover_readiness_progress_stalled,live_node_drift_marker_parity_mismatch,ci_local_promotion_budget_boundary_exceeded":
    raise SystemExit("expected deterministic failover_readiness_reason_codes_csv marker")
PY

set +e
over_budget_output="$(
  bash "$PREFLIGHT_LANE" \
    --skip-suite \
    --simulate-delay-seconds 1 \
    --max-seconds 0 \
    --output-json "$TMP_REPORT" 2>&1
)"
over_budget_code=$?
set -e

if [ "$over_budget_code" -eq 0 ]; then
  echo "expected failover/sync preflight budget guard to fail over-budget run" >&2
  exit 1
fi

# Regression: #788
if ! printf '%s\n' "$over_budget_output" | grep -q "exceeded runtime budget"; then
  echo "expected failover/sync preflight budget overrun signal" >&2
  exit 1
fi

set +e
drift_output="$(
  bash "$PREFLIGHT_LANE" \
    --skip-suite \
    --simulate-live-node-drift \
    --max-seconds 5 \
    --output-json "$TMP_REPORT" 2>&1
)"
drift_code=$?
set -e

if [ "$drift_code" -eq 0 ]; then
  echo "expected failover/sync preflight live-node drift simulation to fail closed" >&2
  exit 1
fi

if ! printf '%s\n' "$drift_output" | grep -q "live_node_drift_marker_parity_mismatch"; then
  echo "expected failover/sync preflight live-node drift mismatch reason marker" >&2
  exit 1
fi

set +e
stall_output="$(
  bash "$PREFLIGHT_LANE" \
    --skip-suite \
    --simulate-failover-stall \
    --max-seconds 5 \
    --output-json "$TMP_REPORT" 2>&1
)"
stall_code=$?
set -e

if [ "$stall_code" -eq 0 ]; then
  echo "expected failover/sync preflight failover stall simulation to fail closed" >&2
  exit 1
fi

if ! printf '%s\n' "$stall_output" | grep -q "failover_readiness_progress_stalled"; then
  echo "expected failover/sync preflight failover stall reason marker" >&2
  exit 1
fi

set +e
ci_boundary_output="$(
  bash "$PREFLIGHT_LANE" \
    --skip-suite \
    --simulate-delay-seconds 1 \
    --max-seconds 5 \
    --ci-local-promotion-max-seconds 0 \
    --output-json "$TMP_REPORT" 2>&1
)"
ci_boundary_code=$?
set -e

if [ "$ci_boundary_code" -eq 0 ]; then
  echo "expected failover/sync preflight ci-local promotion boundary to fail closed" >&2
  exit 1
fi

if ! printf '%s\n' "$ci_boundary_output" | grep -q "ci_local_promotion_budget_boundary_exceeded"; then
  echo "expected failover/sync preflight ci-local promotion boundary reason marker" >&2
  exit 1
fi

if [ ! -L "$PREFLIGHT_LANE" ]; then
  echo "expected failover/sync preflight wrapper to be a dispatcher symlink" >&2
  exit 1
fi

if [ "$(readlink "$PREFLIGHT_LANE")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected failover/sync preflight wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$PREFLIGHT_LANE")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected failover/sync preflight wrapper to resolve runtime manifest via dispatcher" >&2
  exit 1
fi

if ! grep -Fq "failover_sync_drill_preflight_contract_lane_contract.sh" "$MANIFEST_FILE"; then
  echo "expected failover/sync preflight manifest to dispatch shared contract module" >&2
  exit 1
fi

echo "failover/sync preflight contract lane script tests passed."
