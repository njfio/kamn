#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"
CHECKER="$ROOT_DIR/scripts/ci/check_rehearsal_promotion_ci_smoke_convergence.py"
FAST_WORKFLOW="$ROOT_DIR/.github/workflows/ci-fast-gate.yml"
CI_TOOLS_SCRIPT="$ROOT_DIR/scripts/ci/test_ci_tools.sh"
STRATEGY_DOC="$ROOT_DIR/docs/ci/strategy.md"
PLAN_DOC="$ROOT_DIR/docs/plans/2026-02-14-production-service-next-steps.md"

test_harness_require_executable "$CHECKER" "expected rehearsal/promotion ci smoke convergence checker to be executable"

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
  echo "expected repository baseline to satisfy rehearsal/promotion ci smoke checker" >&2
  exit 1
fi
if ! grep -Fq "status=pass" "$safe_log"; then
  cat "$safe_log" >&2
  echo "expected rehearsal/promotion checker status=pass for repository baseline" >&2
  exit 1
fi
if ! grep -Fq "final_decision=GO" "$safe_log"; then
  cat "$safe_log" >&2
  echo "expected rehearsal/promotion checker final_decision=GO for repository baseline" >&2
  exit 1
fi
if ! grep -Fq "rehearsal_promotion_ci_smoke_convergence_status=verified" "$safe_log"; then
  cat "$safe_log" >&2
  echo "expected rehearsal/promotion checker verified marker for repository baseline" >&2
  exit 1
fi
if ! grep -Fq "reason_codes_value=none" "$safe_log"; then
  cat "$safe_log" >&2
  echo "expected rehearsal/promotion checker reason_codes_value=none for repository baseline" >&2
  exit 1
fi
if ! grep -Fq '"final_decision": "GO"' "$safe_report"; then
  cat "$safe_report" >&2
  echo "expected rehearsal/promotion checker json final_decision GO for repository baseline" >&2
  exit 1
fi

missing_bundle_workflow="$(mktemp)"
grep -Fv "bash scripts/deploy/test_generate_staging_rehearsal_bundle.sh" "$FAST_WORKFLOW" >"$missing_bundle_workflow"

missing_bundle_log="$(mktemp)"
if python3 "$CHECKER" \
  --workflow-file "$missing_bundle_workflow" \
  --ci-tools-file "$CI_TOOLS_SCRIPT" \
  --strategy-doc "$STRATEGY_DOC" \
  --plan-doc "$PLAN_DOC" \
  --max-seconds 120 >"$missing_bundle_log" 2>&1; then
  cat "$missing_bundle_log" >&2
  echo "expected missing staging rehearsal bundle smoke command fixture to fail checker" >&2
  exit 1
fi
if ! grep -Fq "rehearsal_bundle_ci_smoke_composition_missing" "$missing_bundle_log"; then
  cat "$missing_bundle_log" >&2
  echo "expected missing staging rehearsal bundle fixture to emit deterministic reason code" >&2
  exit 1
fi

missing_contract_workflow="$(mktemp)"
grep -Fv "bash scripts/deploy/test_run_staging_rehearsal_contract_lane.sh" "$FAST_WORKFLOW" >"$missing_contract_workflow"

missing_contract_log="$(mktemp)"
if python3 "$CHECKER" \
  --workflow-file "$missing_contract_workflow" \
  --ci-tools-file "$CI_TOOLS_SCRIPT" \
  --strategy-doc "$STRATEGY_DOC" \
  --plan-doc "$PLAN_DOC" \
  --max-seconds 120 >"$missing_contract_log" 2>&1; then
  cat "$missing_contract_log" >&2
  echo "expected missing rehearsal contract lane smoke command fixture to fail checker" >&2
  exit 1
fi
if ! grep -Fq "rehearsal_contract_lane_ci_smoke_composition_missing" "$missing_contract_log"; then
  cat "$missing_contract_log" >&2
  echo "expected missing rehearsal contract lane fixture to emit deterministic reason code" >&2
  exit 1
fi

leaked_deep_ci_tools="$(mktemp)"
awk '
  BEGIN { inserted = 0 }
  {
    if (!inserted && $0 ~ /echo "Fast-mode CI tool regression tests passed\."/) {
      print "  bash \"$ROOT_DIR/scripts/deploy/run_staging_rehearsal_deep_lane.sh\" --max-seconds 900"
      inserted = 1
    }
    print $0
  }
' "$CI_TOOLS_SCRIPT" >"$leaked_deep_ci_tools"

leaked_deep_log="$(mktemp)"
if python3 "$CHECKER" \
  --workflow-file "$FAST_WORKFLOW" \
  --ci-tools-file "$leaked_deep_ci_tools" \
  --strategy-doc "$STRATEGY_DOC" \
  --plan-doc "$PLAN_DOC" \
  --max-seconds 120 >"$leaked_deep_log" 2>&1; then
  cat "$leaked_deep_log" >&2
  echo "expected leaked rehearsal deep-lane command fixture in ci-tools fast mode to fail checker" >&2
  exit 1
fi
if ! grep -Fq "rehearsal_deep_lane_command_leaked_in_fast_mode" "$leaked_deep_log"; then
  cat "$leaked_deep_log" >&2
  echo "expected leaked deep-lane fixture to emit deterministic leakage reason code" >&2
  exit 1
fi

leaked_workflow="$(mktemp)"
cat "$FAST_WORKFLOW" >"$leaked_workflow"
cat >>"$leaked_workflow" <<'YAML'
      - name: Leaked rehearsal deep-lane fixture
        run: bash scripts/deploy/run_staging_rehearsal_deep_lane.sh --max-seconds 900
YAML

leaked_workflow_log="$(mktemp)"
if python3 "$CHECKER" \
  --workflow-file "$leaked_workflow" \
  --ci-tools-file "$CI_TOOLS_SCRIPT" \
  --strategy-doc "$STRATEGY_DOC" \
  --plan-doc "$PLAN_DOC" \
  --max-seconds 120 >"$leaked_workflow_log" 2>&1; then
  cat "$leaked_workflow_log" >&2
  echo "expected leaked deep-lane workflow fixture to fail checker" >&2
  exit 1
fi
if ! grep -Fq "ci_fast_gate_rehearsal_deep_lane_not_excluded" "$leaked_workflow_log"; then
  cat "$leaked_workflow_log" >&2
  echo "expected leaked deep-lane workflow fixture to emit deterministic exclusion reason code" >&2
  exit 1
fi

strategy_drift_doc="$(mktemp)"
grep -Fv "### Rehearsal/Promotion CI smoke convergence governance" "$STRATEGY_DOC" >"$strategy_drift_doc"

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
if ! grep -Fq "ci_strategy_rehearsal_promotion_convergence_markers_missing" "$strategy_drift_log"; then
  cat "$strategy_drift_log" >&2
  echo "expected strategy marker drift fixture to emit deterministic reason code" >&2
  exit 1
fi

plan_drift_doc="$(mktemp)"
grep -Fv "### R27.19 Rehearsal/Rollback CI Smoke Governance Closure" "$PLAN_DOC" >"$plan_drift_doc"

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
if ! grep -Fq "production_plan_rehearsal_promotion_convergence_markers_missing" "$plan_drift_log"; then
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
if ! grep -Fq "rehearsal_promotion_ci_smoke_seconds_exceeded" "$budget_overflow_log"; then
  cat "$budget_overflow_log" >&2
  echo "expected max-seconds overflow fixture to emit deterministic budget reason code" >&2
  exit 1
fi

rm -f \
  "$safe_report" "$safe_log" \
  "$missing_bundle_workflow" "$missing_bundle_log" \
  "$missing_contract_workflow" "$missing_contract_log" \
  "$leaked_deep_ci_tools" "$leaked_deep_log" \
  "$leaked_workflow" "$leaked_workflow_log" \
  "$strategy_drift_doc" "$strategy_drift_log" \
  "$plan_drift_doc" "$plan_drift_log" \
  "$budget_overflow_log"

echo "rehearsal/promotion ci smoke convergence checker tests passed."
