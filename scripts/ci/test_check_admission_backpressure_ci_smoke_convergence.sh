#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$ROOT_DIR/scripts/ci/check_admission_backpressure_ci_smoke_convergence.py"
FAST_WORKFLOW="$ROOT_DIR/.github/workflows/ci-fast-gate.yml"
CI_TOOLS_SCRIPT="$ROOT_DIR/scripts/ci/test_ci_tools.sh"
STRATEGY_DOC="$ROOT_DIR/docs/ci/strategy.md"
PLAN_DOC="$ROOT_DIR/docs/plans/2026-02-14-production-service-next-steps.md"

if [ ! -x "$CHECKER" ]; then
  echo "expected admission/backpressure ci smoke convergence checker to be executable" >&2
  exit 1
fi

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
  echo "expected repository baseline to satisfy admission/backpressure ci smoke checker" >&2
  exit 1
fi
if ! grep -Fq "status=pass" "$safe_log"; then
  cat "$safe_log" >&2
  echo "expected admission/backpressure checker status=pass for repository baseline" >&2
  exit 1
fi
if ! grep -Fq "final_decision=GO" "$safe_log"; then
  cat "$safe_log" >&2
  echo "expected admission/backpressure checker final_decision=GO for repository baseline" >&2
  exit 1
fi
if ! grep -Fq "admission_backpressure_ci_smoke_convergence_status=verified" "$safe_log"; then
  cat "$safe_log" >&2
  echo "expected admission/backpressure checker verified marker for repository baseline" >&2
  exit 1
fi
if ! grep -Fq "reason_codes_value=none" "$safe_log"; then
  cat "$safe_log" >&2
  echo "expected admission/backpressure checker reason_codes_value=none for repository baseline" >&2
  exit 1
fi
if ! grep -Fq '"final_decision": "GO"' "$safe_report"; then
  cat "$safe_report" >&2
  echo "expected admission/backpressure checker json final_decision GO for repository baseline" >&2
  exit 1
fi

missing_contract_ci_tools="$(mktemp)"
grep -Fv 'bash "$ROOT_DIR/scripts/runtime/test_validate_service_api_axum_ingress_live_contract_lane.sh"' "$CI_TOOLS_SCRIPT" >"$missing_contract_ci_tools"

missing_contract_log="$(mktemp)"
if python3 "$CHECKER" \
  --workflow-file "$FAST_WORKFLOW" \
  --ci-tools-file "$missing_contract_ci_tools" \
  --strategy-doc "$STRATEGY_DOC" \
  --plan-doc "$PLAN_DOC" \
  --max-seconds 120 >"$missing_contract_log" 2>&1; then
  cat "$missing_contract_log" >&2
  echo "expected missing service-api-axum contract-lane smoke command fixture to fail checker" >&2
  exit 1
fi
if ! grep -Fq "service_api_axum_contract_lane_ci_smoke_composition_missing" "$missing_contract_log"; then
  cat "$missing_contract_log" >&2
  echo "expected missing service-api-axum contract-lane smoke command fixture to emit deterministic reason code" >&2
  exit 1
fi

leaked_run_ci_tools="$(mktemp)"
awk '
  BEGIN { inserted = 0 }
  {
    if (!inserted && $0 ~ /echo "Fast-mode CI tool regression tests passed\."/) {
      print "  bash \"$ROOT_DIR/scripts/runtime/validate_service_api_axum_ingress_live.sh\" --output-json /tmp/service-api-axum-ingress-live-summary.json"
      inserted = 1
    }
    print $0
  }
' "$CI_TOOLS_SCRIPT" >"$leaked_run_ci_tools"

leaked_run_log="$(mktemp)"
if python3 "$CHECKER" \
  --workflow-file "$FAST_WORKFLOW" \
  --ci-tools-file "$leaked_run_ci_tools" \
  --strategy-doc "$STRATEGY_DOC" \
  --plan-doc "$PLAN_DOC" \
  --max-seconds 120 >"$leaked_run_log" 2>&1; then
  cat "$leaked_run_log" >&2
  echo "expected leaked service-api-axum run command fixture in ci-tools fast mode to fail checker" >&2
  exit 1
fi
if ! grep -Fq "service_api_axum_run_command_leaked_in_fast_mode" "$leaked_run_log"; then
  cat "$leaked_run_log" >&2
  echo "expected leaked service-api-axum run command fixture to emit deterministic leakage reason code" >&2
  exit 1
fi

leaked_workflow="$(mktemp)"
cat "$FAST_WORKFLOW" >"$leaked_workflow"
cat >>"$leaked_workflow" <<'YAML'
      - name: Leaked service-api-axum run fixture
        run: bash scripts/runtime/validate_service_api_axum_ingress_live.sh --output-json /tmp/service-api-axum-ingress-live-summary.json
YAML

leaked_workflow_log="$(mktemp)"
if python3 "$CHECKER" \
  --workflow-file "$leaked_workflow" \
  --ci-tools-file "$CI_TOOLS_SCRIPT" \
  --strategy-doc "$STRATEGY_DOC" \
  --plan-doc "$PLAN_DOC" \
  --max-seconds 120 >"$leaked_workflow_log" 2>&1; then
  cat "$leaked_workflow_log" >&2
  echo "expected leaked service-api-axum run command workflow fixture to fail checker" >&2
  exit 1
fi
if ! grep -Fq "ci_fast_gate_service_api_axum_run_command_not_excluded" "$leaked_workflow_log"; then
  cat "$leaked_workflow_log" >&2
  echo "expected leaked service-api-axum workflow fixture to emit deterministic exclusion reason code" >&2
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
if ! grep -Fq "admission_backpressure_ci_smoke_seconds_exceeded" "$budget_overflow_log"; then
  cat "$budget_overflow_log" >&2
  echo "expected max-seconds overflow fixture to emit deterministic budget reason code" >&2
  exit 1
fi

rm -f \
  "$safe_report" "$safe_log" \
  "$missing_contract_ci_tools" "$missing_contract_log" \
  "$leaked_run_ci_tools" "$leaked_run_log" \
  "$leaked_workflow" "$leaked_workflow_log" \
  "$budget_overflow_log"

echo "admission/backpressure ci smoke convergence checker tests passed."
