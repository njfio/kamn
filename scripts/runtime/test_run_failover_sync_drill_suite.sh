#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SUITE="$ROOT_DIR/scripts/runtime/run_failover_sync_drill_suite.sh"
TMP_REPORT="$(mktemp)"
TMP_GITHUB_OUTPUT="$(mktemp)"
trap 'rm -f "$TMP_REPORT" "$TMP_GITHUB_OUTPUT"' EXIT

if [ ! -x "$SUITE" ]; then
  echo "expected failover/sync drill suite script to be executable" >&2
  exit 1
fi

preflight_output="$(
  bash "$SUITE" \
    --event-name pull_request \
    --skip-suite \
    --output-json "$TMP_REPORT"
)"
if ! printf '%s\n' "$preflight_output" | grep -q "failover/sync drill suite tests passed."; then
  echo "expected failover/sync suite preflight success marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text())
if payload.get("schema_version") != "kamn.runtime.failover-sync-drill-suite-report.v1":
    raise SystemExit("unexpected failover/sync suite report schema")
if payload.get("selected_lane") != "preflight":
    raise SystemExit("expected preflight lane for pull_request event")
if payload.get("status") != "pass":
    raise SystemExit("expected preflight suite status to pass")
lane_report = payload.get("lane_report", {})
if lane_report.get("failover_promotion_gate_status") != "verified":
    raise SystemExit("expected failover_promotion_gate_status=verified in preflight lane report")
if lane_report.get("live_node_drift_parity_status") != "verified":
    raise SystemExit("expected live_node_drift_parity_status=verified in preflight lane report")
if lane_report.get("ci_local_promotion_budget_boundary_status") != "verified":
    raise SystemExit("expected ci_local_promotion_budget_boundary_status=verified in preflight lane report")
if lane_report.get("failover_readiness_reason_taxonomy_version") != "kamn.runtime.failover-readiness-reason-taxonomy.v1":
    raise SystemExit("expected failover_readiness_reason_taxonomy_version marker in preflight lane report")
if lane_report.get("failover_readiness_reason_codes_csv") != "failover_readiness_progress_stalled,live_node_drift_marker_parity_mismatch,ci_local_promotion_budget_boundary_exceeded":
    raise SystemExit("expected deterministic failover_readiness_reason_codes_csv marker in preflight lane report")
if lane_report.get("drift_taxonomy_mapping_status") != "verified":
    raise SystemExit("expected drift_taxonomy_mapping_status=verified in preflight lane report")
if lane_report.get("runbook_marker_parity_status") != "verified":
    raise SystemExit("expected runbook_marker_parity_status=verified in preflight lane report")
if lane_report.get("drift_taxonomy_runbook_reason_taxonomy_version") != "kamn.runtime.failover-drift-taxonomy-runbook-reason-taxonomy.v1":
    raise SystemExit("expected drift_taxonomy_runbook_reason_taxonomy_version marker in preflight lane report")
if lane_report.get("drift_taxonomy_runbook_reason_codes_csv") != "drift_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch":
    raise SystemExit("expected deterministic drift_taxonomy_runbook_reason_codes_csv marker in preflight lane report")
PY

deep_output="$(
  bash "$SUITE" \
    --event-name schedule \
    --skip-suite \
    --output-json "$TMP_REPORT"
)"
if ! printf '%s\n' "$deep_output" | grep -q "failover/sync drill suite tests passed."; then
  echo "expected failover/sync suite deep success marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text())
if payload.get("selected_lane") != "deep":
    raise SystemExit("expected deep lane for schedule event")
if payload.get("status") != "pass":
    raise SystemExit("expected deep suite status to pass")
lane_report = payload.get("lane_report", {})
if lane_report.get("lane") != "deep":
    raise SystemExit("expected deep lane report payload")
PY

ci_output="$(
  GITHUB_OUTPUT="$TMP_GITHUB_OUTPUT" \
    bash "$SUITE" \
      --event-name schedule \
      --skip-suite \
      --output-json "$TMP_REPORT"
)"
if ! printf '%s\n' "$ci_output" | grep -q "failover/sync drill suite tests passed."; then
  echo "expected failover/sync suite success marker under GitHub output env" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text())
if payload.get("selected_lane") != "deep":
    raise SystemExit("expected deep lane under GitHub output env")
if payload.get("status") != "pass":
    raise SystemExit("expected deep suite status to pass under GitHub output env")
PY

echo "failover/sync suite script tests passed."
