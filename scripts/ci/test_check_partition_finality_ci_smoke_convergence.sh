#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$ROOT_DIR/scripts/ci/check_partition_finality_ci_smoke_convergence.py"
FAST_WORKFLOW="$ROOT_DIR/.github/workflows/ci-fast-gate.yml"
CI_TOOLS_SCRIPT="$ROOT_DIR/scripts/ci/test_ci_tools.sh"
STRATEGY_DOC="$ROOT_DIR/docs/ci/strategy.md"
PLAN_DOC="$ROOT_DIR/docs/plans/2026-02-14-production-service-next-steps.md"

if [ ! -x "$CHECKER" ]; then
  echo "expected partition-finality ci smoke convergence checker to be executable" >&2
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
  echo "expected repository baseline to satisfy partition-finality ci smoke convergence checker" >&2
  exit 1
fi
if ! grep -Fq "status=pass" "$safe_log"; then
  cat "$safe_log" >&2
  echo "expected partition-finality checker status=pass for repository baseline" >&2
  exit 1
fi
if ! grep -Fq "final_decision=GO" "$safe_log"; then
  cat "$safe_log" >&2
  echo "expected partition-finality checker final_decision=GO for repository baseline" >&2
  exit 1
fi
if ! grep -Fq "partition_finality_ci_smoke_convergence_status=verified" "$safe_log"; then
  cat "$safe_log" >&2
  echo "expected partition-finality checker verified marker for repository baseline" >&2
  exit 1
fi
if ! grep -Fq "reason_codes_value=none" "$safe_log"; then
  cat "$safe_log" >&2
  echo "expected partition-finality checker reason_codes_value=none for repository baseline" >&2
  exit 1
fi

missing_ci_tools="$(mktemp)"
grep -Fv 'bash "$ROOT_DIR/scripts/runtime/test_check_libp2p_convergence_process_isolated_live_evidence_convergence.sh"' "$CI_TOOLS_SCRIPT" >"$missing_ci_tools"

missing_log="$(mktemp)"
if python3 "$CHECKER" \
  --workflow-file "$FAST_WORKFLOW" \
  --ci-tools-file "$missing_ci_tools" \
  --strategy-doc "$STRATEGY_DOC" \
  --plan-doc "$PLAN_DOC" \
  --max-seconds 120 >"$missing_log" 2>&1; then
  cat "$missing_log" >&2
  echo "expected missing libp2p evidence smoke command fixture to fail checker" >&2
  exit 1
fi
if ! grep -Fq "libp2p_evidence_ci_smoke_composition_missing" "$missing_log"; then
  cat "$missing_log" >&2
  echo "expected missing libp2p evidence smoke command fixture to emit deterministic reason code" >&2
  exit 1
fi

leaked_deep_lane_ci_tools="$(mktemp)"
awk '
  BEGIN { inserted = 0 }
  {
    if (!inserted && $0 ~ /echo "Fast-mode CI tool regression tests passed\."/) {
      print "  bash \"$ROOT_DIR/scripts/runtime/validate_libp2p_convergence_process_isolated_live.sh\" --mode run --lane-profile deep --ci-fast-gate FAIL --output-json /tmp/libp2p-convergence-process-isolated-live-deep-summary.json"
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
  echo "expected leaked partition-finality deep run command fixture to fail checker" >&2
  exit 1
fi
if ! grep -Fq "partition_finality_run_mode_command_leaked_in_fast_mode" "$leaked_deep_lane_log"; then
  cat "$leaked_deep_lane_log" >&2
  echo "expected leaked partition-finality deep run command fixture to emit deterministic leakage reason code" >&2
  exit 1
fi

leaked_deep_lane_workflow="$(mktemp)"
cat "$FAST_WORKFLOW" >"$leaked_deep_lane_workflow"
cat >>"$leaked_deep_lane_workflow" <<'YAML'
      - name: Leaked partition finality deep lane fixture
        run: bash scripts/runtime/validate_libp2p_convergence_process_isolated_live.sh --mode run --lane-profile deep --ci-fast-gate FAIL --output-json /tmp/libp2p-convergence-process-isolated-live-deep-summary.json
YAML

leaked_workflow_log="$(mktemp)"
if python3 "$CHECKER" \
  --workflow-file "$leaked_deep_lane_workflow" \
  --ci-tools-file "$CI_TOOLS_SCRIPT" \
  --strategy-doc "$STRATEGY_DOC" \
  --plan-doc "$PLAN_DOC" \
  --max-seconds 120 >"$leaked_workflow_log" 2>&1; then
  cat "$leaked_workflow_log" >&2
  echo "expected leaked partition-finality deep lane workflow fixture to fail checker" >&2
  exit 1
fi
if ! grep -Fq "ci_fast_gate_partition_finality_run_mode_not_excluded" "$leaked_workflow_log"; then
  cat "$leaked_workflow_log" >&2
  echo "expected leaked partition-finality deep lane workflow fixture to emit deterministic exclusion reason code" >&2
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
if ! grep -Fq "partition_finality_ci_smoke_seconds_exceeded" "$budget_overflow_log"; then
  cat "$budget_overflow_log" >&2
  echo "expected max-seconds overflow fixture to emit deterministic budget reason code" >&2
  exit 1
fi

rm -f \
  "$safe_report" "$safe_log" \
  "$missing_ci_tools" "$missing_log" \
  "$leaked_deep_lane_ci_tools" "$leaked_deep_lane_log" \
  "$leaked_deep_lane_workflow" "$leaked_workflow_log" \
  "$budget_overflow_log"

echo "partition-finality ci smoke convergence checker tests passed."
