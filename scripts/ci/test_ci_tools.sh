#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

bash "$ROOT_DIR/scripts/ci/test_evaluate_budget.sh"
bash "$ROOT_DIR/scripts/ci/test_run_with_retry.sh"
bash "$ROOT_DIR/scripts/ci/test_run_invariant_harness.sh"
bash "$ROOT_DIR/scripts/ci/test_run_cargo_test_with_quarantine.sh"
bash "$ROOT_DIR/scripts/ci/test_select_targets.sh"
bash "$ROOT_DIR/scripts/ci/test_check_flaky_registry.sh"
bash "$ROOT_DIR/scripts/ci/test_summarize_budget_artifacts.sh"
bash "$ROOT_DIR/scripts/ci/test_check_pr_ci_declaration.sh"
bash "$ROOT_DIR/scripts/ci/test_post_flaky_report_comment.sh"
bash "$ROOT_DIR/scripts/ci/test_sync_flaky_registry_issues.sh"
bash "$ROOT_DIR/scripts/ci/test_generate_performance_smoke_report.sh"
bash "$ROOT_DIR/scripts/ci/test_check_performance_thresholds.sh"
bash "$ROOT_DIR/scripts/ci/test_workflow_retry_policy.sh"
bash "$ROOT_DIR/scripts/ci/test_workflow_cache_policy.sh"
bash "$ROOT_DIR/scripts/ci/test_workflow_scope_policy.sh"
bash "$ROOT_DIR/scripts/ci/test_workflow_performance_policy.sh"

echo "All CI tool regression tests passed."
