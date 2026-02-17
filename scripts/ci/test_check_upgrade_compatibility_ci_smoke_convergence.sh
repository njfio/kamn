#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$ROOT_DIR/scripts/ci/check_upgrade_compatibility_ci_smoke_convergence.py"
FAST_WORKFLOW="$ROOT_DIR/.github/workflows/ci-fast-gate.yml"
CI_TOOLS_SCRIPT="$ROOT_DIR/scripts/ci/test_ci_tools.sh"
STRATEGY_DOC="$ROOT_DIR/docs/ci/strategy.md"
PLAN_DOC="$ROOT_DIR/docs/plans/2026-02-14-production-service-next-steps.md"

if [ ! -x "$CHECKER" ]; then
  echo "expected upgrade compatibility ci smoke convergence checker to be executable" >&2
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
  echo "expected repository baseline to satisfy upgrade compatibility ci smoke checker" >&2
  exit 1
fi
if ! grep -Fq "status=pass" "$safe_log"; then
  cat "$safe_log" >&2
  echo "expected upgrade compatibility checker status=pass for repository baseline" >&2
  exit 1
fi
if ! grep -Fq "final_decision=GO" "$safe_log"; then
  cat "$safe_log" >&2
  echo "expected upgrade compatibility checker final_decision=GO for repository baseline" >&2
  exit 1
fi
if ! grep -Fq "upgrade_compatibility_ci_smoke_convergence_status=verified" "$safe_log"; then
  cat "$safe_log" >&2
  echo "expected upgrade compatibility checker verified marker for repository baseline" >&2
  exit 1
fi
if ! grep -Fq "reason_codes_value=none" "$safe_log"; then
  cat "$safe_log" >&2
  echo "expected upgrade compatibility checker reason_codes_value=none for repository baseline" >&2
  exit 1
fi
if ! grep -Fq '"final_decision": "GO"' "$safe_report"; then
  cat "$safe_report" >&2
  echo "expected upgrade compatibility checker json final_decision GO for repository baseline" >&2
  exit 1
fi

missing_evidence_ci_tools="$(mktemp)"
grep -Fv 'bash "$ROOT_DIR/scripts/kolme/test_generate_fork_compatibility_evidence.sh"' "$CI_TOOLS_SCRIPT" >"$missing_evidence_ci_tools"

missing_evidence_log="$(mktemp)"
if python3 "$CHECKER" \
  --workflow-file "$FAST_WORKFLOW" \
  --ci-tools-file "$missing_evidence_ci_tools" \
  --strategy-doc "$STRATEGY_DOC" \
  --plan-doc "$PLAN_DOC" \
  --max-seconds 120 >"$missing_evidence_log" 2>&1; then
  cat "$missing_evidence_log" >&2
  echo "expected missing fork-compatibility evidence smoke command fixture to fail checker" >&2
  exit 1
fi
if ! grep -Fq "upgrade_compatibility_fork_evidence_ci_smoke_composition_missing" "$missing_evidence_log"; then
  cat "$missing_evidence_log" >&2
  echo "expected missing fork-compatibility evidence fixture to emit deterministic reason code" >&2
  exit 1
fi

missing_policy_ci_tools="$(mktemp)"
grep -Fv 'bash "$ROOT_DIR/scripts/kolme/test_check_fork_compatibility_policy.sh"' "$CI_TOOLS_SCRIPT" >"$missing_policy_ci_tools"

missing_policy_log="$(mktemp)"
if python3 "$CHECKER" \
  --workflow-file "$FAST_WORKFLOW" \
  --ci-tools-file "$missing_policy_ci_tools" \
  --strategy-doc "$STRATEGY_DOC" \
  --plan-doc "$PLAN_DOC" \
  --max-seconds 120 >"$missing_policy_log" 2>&1; then
  cat "$missing_policy_log" >&2
  echo "expected missing fork-compatibility policy smoke command fixture to fail checker" >&2
  exit 1
fi
if ! grep -Fq "upgrade_compatibility_fork_policy_ci_smoke_composition_missing" "$missing_policy_log"; then
  cat "$missing_policy_log" >&2
  echo "expected missing fork-compatibility policy fixture to emit deterministic reason code" >&2
  exit 1
fi

leaked_replay_ci_tools="$(mktemp)"
awk '
  BEGIN { inserted = 0 }
  {
    if (!inserted && $0 ~ /echo "Fast-mode CI tool regression tests passed\."/) {
      print "  bash \"$ROOT_DIR/scripts/kolme/run_version_compatibility_replay_deep_lane.sh\" --output-json /tmp/kolme-version-compatibility-deep-lane-report.json"
      inserted = 1
    }
    print $0
  }
' "$CI_TOOLS_SCRIPT" >"$leaked_replay_ci_tools"

leaked_replay_log="$(mktemp)"
if python3 "$CHECKER" \
  --workflow-file "$FAST_WORKFLOW" \
  --ci-tools-file "$leaked_replay_ci_tools" \
  --strategy-doc "$STRATEGY_DOC" \
  --plan-doc "$PLAN_DOC" \
  --max-seconds 120 >"$leaked_replay_log" 2>&1; then
  cat "$leaked_replay_log" >&2
  echo "expected leaked upgrade compatibility replay command fixture in ci-tools fast mode to fail checker" >&2
  exit 1
fi
if ! grep -Fq "upgrade_compatibility_replay_command_leaked_in_fast_mode" "$leaked_replay_log"; then
  cat "$leaked_replay_log" >&2
  echo "expected leaked replay command fixture to emit deterministic leakage reason code" >&2
  exit 1
fi

leaked_workflow="$(mktemp)"
cat "$FAST_WORKFLOW" >"$leaked_workflow"
cat >>"$leaked_workflow" <<'YAML'
      - name: Leaked upgrade compatibility replay fixture
        run: bash scripts/kolme/run_version_compatibility_replay_deep_lane.sh --output-json /tmp/kolme-version-compatibility-deep-lane-report.json
YAML

leaked_workflow_log="$(mktemp)"
if python3 "$CHECKER" \
  --workflow-file "$leaked_workflow" \
  --ci-tools-file "$CI_TOOLS_SCRIPT" \
  --strategy-doc "$STRATEGY_DOC" \
  --plan-doc "$PLAN_DOC" \
  --max-seconds 120 >"$leaked_workflow_log" 2>&1; then
  cat "$leaked_workflow_log" >&2
  echo "expected leaked replay command workflow fixture to fail checker" >&2
  exit 1
fi
if ! grep -Fq "ci_fast_gate_upgrade_compatibility_replay_command_not_excluded" "$leaked_workflow_log"; then
  cat "$leaked_workflow_log" >&2
  echo "expected leaked replay workflow fixture to emit deterministic exclusion reason code" >&2
  exit 1
fi

strategy_drift_doc="$(mktemp)"
grep -Fv "### Upgrade Compatibility CI smoke convergence governance" "$STRATEGY_DOC" >"$strategy_drift_doc"

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
if ! grep -Fq "ci_strategy_upgrade_compatibility_convergence_markers_missing" "$strategy_drift_log"; then
  cat "$strategy_drift_log" >&2
  echo "expected strategy marker drift fixture to emit deterministic reason code" >&2
  exit 1
fi

plan_drift_doc="$(mktemp)"
grep -Fv "### R27.21 Upgrade Compatibility CI Smoke Governance Closure" "$PLAN_DOC" >"$plan_drift_doc"

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
if ! grep -Fq "production_plan_upgrade_compatibility_convergence_markers_missing" "$plan_drift_log"; then
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
if ! grep -Fq "upgrade_compatibility_ci_smoke_seconds_exceeded" "$budget_overflow_log"; then
  cat "$budget_overflow_log" >&2
  echo "expected max-seconds overflow fixture to emit deterministic budget reason code" >&2
  exit 1
fi

rm -f \
  "$safe_report" "$safe_log" \
  "$missing_evidence_ci_tools" "$missing_evidence_log" \
  "$missing_policy_ci_tools" "$missing_policy_log" \
  "$leaked_replay_ci_tools" "$leaked_replay_log" \
  "$leaked_workflow" "$leaked_workflow_log" \
  "$strategy_drift_doc" "$strategy_drift_log" \
  "$plan_drift_doc" "$plan_drift_log" \
  "$budget_overflow_log"

echo "upgrade compatibility ci smoke convergence checker tests passed."
