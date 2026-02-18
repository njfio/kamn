#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"
CHECKER="$ROOT_DIR/scripts/ci/check_custody_rotation_ci_smoke_convergence.py"
FAST_WORKFLOW="$ROOT_DIR/.github/workflows/ci-fast-gate.yml"
CI_TOOLS_SCRIPT="$ROOT_DIR/scripts/ci/test_ci_tools.sh"
STRATEGY_DOC="$ROOT_DIR/docs/ci/strategy.md"
PLAN_DOC="$ROOT_DIR/docs/plans/2026-02-14-production-service-next-steps.md"

test_harness_require_executable "$CHECKER" "expected custody/rotation ci smoke convergence checker to be executable"

safe_report="$(mktemp)"
safe_log="$(mktemp)"
if ! python3 "$CHECKER" \
  --workflow-file "$FAST_WORKFLOW" \
  --ci-tools-file "$CI_TOOLS_SCRIPT" \
  --strategy-doc "$STRATEGY_DOC" \
  --plan-doc "$PLAN_DOC" \
  --max-seconds 120 \
  --output-json "$safe_report" >"$safe_log" 2>&1; then
  cat "$safe_log" >&2
  echo "expected repository baseline to satisfy custody/rotation ci smoke checker" >&2
  exit 1
fi
if ! grep -Fq "status=pass" "$safe_log"; then
  cat "$safe_log" >&2
  echo "expected custody/rotation checker status=pass for repository baseline" >&2
  exit 1
fi
if ! grep -Fq "final_decision=GO" "$safe_log"; then
  cat "$safe_log" >&2
  echo "expected custody/rotation checker final_decision=GO for repository baseline" >&2
  exit 1
fi
if ! grep -Fq "custody_rotation_ci_smoke_convergence_status=verified" "$safe_log"; then
  cat "$safe_log" >&2
  echo "expected custody/rotation checker verified marker for repository baseline" >&2
  exit 1
fi
if ! grep -Fq "reason_codes_value=none" "$safe_log"; then
  cat "$safe_log" >&2
  echo "expected custody/rotation checker reason_codes_value=none for repository baseline" >&2
  exit 1
fi
if ! grep -Fq '"final_decision": "GO"' "$safe_report"; then
  cat "$safe_report" >&2
  echo "expected custody/rotation checker json final_decision GO for repository baseline" >&2
  exit 1
fi

missing_docs_contract_ci_tools="$(mktemp)"
grep -Fv 'bash "$ROOT_DIR/scripts/ci/test_production_service_next_steps_contract.sh"' "$CI_TOOLS_SCRIPT" >"$missing_docs_contract_ci_tools"

missing_docs_contract_log="$(mktemp)"
if python3 "$CHECKER" \
  --workflow-file "$FAST_WORKFLOW" \
  --ci-tools-file "$missing_docs_contract_ci_tools" \
  --strategy-doc "$STRATEGY_DOC" \
  --plan-doc "$PLAN_DOC" \
  --max-seconds 120 >"$missing_docs_contract_log" 2>&1; then
  cat "$missing_docs_contract_log" >&2
  echo "expected missing custody/rotation docs smoke contract fixture to fail checker" >&2
  exit 1
fi
if ! grep -Fq "custody_rotation_smoke_contract_ci_smoke_composition_missing" "$missing_docs_contract_log"; then
  cat "$missing_docs_contract_log" >&2
  echo "expected missing custody/rotation docs smoke contract fixture to emit deterministic reason code" >&2
  exit 1
fi

missing_preflight_ci_tools="$(mktemp)"
grep -Fv 'bash "$ROOT_DIR/scripts/runtime/test_run_failover_sync_drill_preflight_contract_lane.sh"' "$CI_TOOLS_SCRIPT" >"$missing_preflight_ci_tools"

missing_preflight_log="$(mktemp)"
if python3 "$CHECKER" \
  --workflow-file "$FAST_WORKFLOW" \
  --ci-tools-file "$missing_preflight_ci_tools" \
  --strategy-doc "$STRATEGY_DOC" \
  --plan-doc "$PLAN_DOC" \
  --max-seconds 120 >"$missing_preflight_log" 2>&1; then
  cat "$missing_preflight_log" >&2
  echo "expected missing failover preflight smoke command fixture to fail checker" >&2
  exit 1
fi
if ! grep -Fq "failover_preflight_ci_smoke_composition_missing" "$missing_preflight_log"; then
  cat "$missing_preflight_log" >&2
  echo "expected missing failover preflight fixture to emit deterministic reason code" >&2
  exit 1
fi

missing_suite_ci_tools="$(mktemp)"
grep -Fv 'bash "$ROOT_DIR/scripts/runtime/test_run_failover_sync_drill_suite.sh"' "$CI_TOOLS_SCRIPT" >"$missing_suite_ci_tools"

missing_suite_log="$(mktemp)"
if python3 "$CHECKER" \
  --workflow-file "$FAST_WORKFLOW" \
  --ci-tools-file "$missing_suite_ci_tools" \
  --strategy-doc "$STRATEGY_DOC" \
  --plan-doc "$PLAN_DOC" \
  --max-seconds 120 >"$missing_suite_log" 2>&1; then
  cat "$missing_suite_log" >&2
  echo "expected missing failover suite smoke command fixture to fail checker" >&2
  exit 1
fi
if ! grep -Fq "failover_suite_ci_smoke_composition_missing" "$missing_suite_log"; then
  cat "$missing_suite_log" >&2
  echo "expected missing failover suite fixture to emit deterministic reason code" >&2
  exit 1
fi

leaked_deep_lane_ci_tools="$(mktemp)"
awk '
  BEGIN { inserted = 0 }
  {
    if (!inserted && $0 ~ /echo "Fast-mode CI tool regression tests passed\."/) {
      print "  bash \"$ROOT_DIR/scripts/runtime/run_failover_sync_drill_deep_lane.sh\""
      inserted = 1
    }
    print $0
  }
' "$CI_TOOLS_SCRIPT" >"$leaked_deep_lane_ci_tools"

leaked_deep_lane_log="$(mktemp)"
if python3 "$CHECKER" \
  --workflow-file "$FAST_WORKFLOW" \
  --ci-tools-file "$leaked_deep_lane_ci_tools" \
  --strategy-doc "$STRATEGY_DOC" \
  --plan-doc "$PLAN_DOC" \
  --max-seconds 120 >"$leaked_deep_lane_log" 2>&1; then
  cat "$leaked_deep_lane_log" >&2
  echo "expected leaked failover deep-lane command fixture in ci-tools fast mode to fail checker" >&2
  exit 1
fi
if ! grep -Fq "failover_deep_lane_command_leaked_in_fast_mode" "$leaked_deep_lane_log"; then
  cat "$leaked_deep_lane_log" >&2
  echo "expected leaked deep-lane command fixture to emit deterministic leakage reason code" >&2
  exit 1
fi

leaked_workflow="$(mktemp)"
cat "$FAST_WORKFLOW" >"$leaked_workflow"
cat >>"$leaked_workflow" <<'YAML'
      - name: Leaked failover deep lane fixture
        run: bash scripts/runtime/run_failover_sync_drill_deep_lane.sh
YAML

leaked_workflow_log="$(mktemp)"
if python3 "$CHECKER" \
  --workflow-file "$leaked_workflow" \
  --ci-tools-file "$CI_TOOLS_SCRIPT" \
  --strategy-doc "$STRATEGY_DOC" \
  --plan-doc "$PLAN_DOC" \
  --max-seconds 120 >"$leaked_workflow_log" 2>&1; then
  cat "$leaked_workflow_log" >&2
  echo "expected leaked failover deep-lane workflow fixture to fail checker" >&2
  exit 1
fi
if ! grep -Fq "ci_fast_gate_failover_deep_lane_not_excluded" "$leaked_workflow_log"; then
  cat "$leaked_workflow_log" >&2
  echo "expected leaked deep-lane workflow fixture to emit deterministic exclusion reason code" >&2
  exit 1
fi

strategy_drift_doc="$(mktemp)"
grep -Fv "### Custody/Rotation CI smoke convergence governance" "$STRATEGY_DOC" >"$strategy_drift_doc"

strategy_drift_log="$(mktemp)"
if python3 "$CHECKER" \
  --workflow-file "$FAST_WORKFLOW" \
  --ci-tools-file "$CI_TOOLS_SCRIPT" \
  --strategy-doc "$strategy_drift_doc" \
  --plan-doc "$PLAN_DOC" \
  --max-seconds 120 >"$strategy_drift_log" 2>&1; then
  cat "$strategy_drift_log" >&2
  echo "expected strategy marker drift fixture to fail checker" >&2
  exit 1
fi
if ! grep -Fq "ci_strategy_custody_rotation_convergence_markers_missing" "$strategy_drift_log"; then
  cat "$strategy_drift_log" >&2
  echo "expected strategy marker drift fixture to emit deterministic reason code" >&2
  exit 1
fi

plan_drift_doc="$(mktemp)"
grep -Fv "### R27.20 Secret Material Zeroization and Signer-Rotation Governance Closure" "$PLAN_DOC" >"$plan_drift_doc"

plan_drift_log="$(mktemp)"
if python3 "$CHECKER" \
  --workflow-file "$FAST_WORKFLOW" \
  --ci-tools-file "$CI_TOOLS_SCRIPT" \
  --strategy-doc "$STRATEGY_DOC" \
  --plan-doc "$plan_drift_doc" \
  --max-seconds 120 >"$plan_drift_log" 2>&1; then
  cat "$plan_drift_log" >&2
  echo "expected production-plan marker drift fixture to fail checker" >&2
  exit 1
fi
if ! grep -Fq "production_plan_custody_rotation_convergence_markers_missing" "$plan_drift_log"; then
  cat "$plan_drift_log" >&2
  echo "expected production-plan marker drift fixture to emit deterministic reason code" >&2
  exit 1
fi

budget_overflow_log="$(mktemp)"
if python3 "$CHECKER" \
  --workflow-file "$FAST_WORKFLOW" \
  --ci-tools-file "$CI_TOOLS_SCRIPT" \
  --strategy-doc "$STRATEGY_DOC" \
  --plan-doc "$PLAN_DOC" \
  --max-seconds 121 >"$budget_overflow_log" 2>&1; then
  cat "$budget_overflow_log" >&2
  echo "expected max-seconds overflow fixture to fail checker" >&2
  exit 1
fi
if ! grep -Fq "custody_rotation_ci_smoke_seconds_exceeded" "$budget_overflow_log"; then
  cat "$budget_overflow_log" >&2
  echo "expected max-seconds overflow fixture to emit deterministic budget reason code" >&2
  exit 1
fi

rm -f \
  "$safe_report" "$safe_log" \
  "$missing_docs_contract_ci_tools" "$missing_docs_contract_log" \
  "$missing_preflight_ci_tools" "$missing_preflight_log" \
  "$missing_suite_ci_tools" "$missing_suite_log" \
  "$leaked_deep_lane_ci_tools" "$leaked_deep_lane_log" \
  "$leaked_workflow" "$leaked_workflow_log" \
  "$strategy_drift_doc" "$strategy_drift_log" \
  "$plan_drift_doc" "$plan_drift_log" \
  "$budget_overflow_log"

echo "custody/rotation ci smoke convergence checker tests passed."
