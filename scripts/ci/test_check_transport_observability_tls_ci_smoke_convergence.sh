#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$ROOT_DIR/scripts/ci/check_transport_observability_tls_ci_smoke_convergence.py"
FAST_WORKFLOW="$ROOT_DIR/.github/workflows/ci-fast-gate.yml"
CI_TOOLS_SCRIPT="$ROOT_DIR/scripts/ci/test_ci_tools.sh"
STRATEGY_DOC="$ROOT_DIR/docs/ci/strategy.md"
PLAN_DOC="$ROOT_DIR/docs/plans/2026-02-14-production-service-next-steps.md"

if [ ! -x "$CHECKER" ]; then
  echo "expected transport/observability/tls ci smoke convergence checker to be executable" >&2
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
  echo "expected repository baseline to satisfy transport/observability/tls ci smoke convergence checker" >&2
  exit 1
fi
if ! grep -Fq "status=pass" "$safe_log"; then
  cat "$safe_log" >&2
  echo "expected convergence checker status=pass for repository baseline" >&2
  exit 1
fi
if ! grep -Fq "final_decision=GO" "$safe_log"; then
  cat "$safe_log" >&2
  echo "expected convergence checker final_decision=GO for repository baseline" >&2
  exit 1
fi
if ! grep -Fq "transport_observability_tls_ci_smoke_convergence_status=verified" "$safe_log"; then
  cat "$safe_log" >&2
  echo "expected convergence checker verified marker for repository baseline" >&2
  exit 1
fi
if ! grep -Fq "reason_codes_value=none" "$safe_log"; then
  cat "$safe_log" >&2
  echo "expected convergence checker reason_codes_value=none for repository baseline" >&2
  exit 1
fi
if ! grep -Fq '"final_decision": "GO"' "$safe_report"; then
  cat "$safe_report" >&2
  echo "expected convergence checker json final_decision GO for repository baseline" >&2
  exit 1
fi

transport_missing_ci_tools="$(mktemp)"
grep -Fv 'bash "$ROOT_DIR/scripts/ci/test_live_transport_fault_matrix_ci_exclusion_policy.sh"' "$CI_TOOLS_SCRIPT" >"$transport_missing_ci_tools"

transport_missing_log="$(mktemp)"
if python3 "$CHECKER" \
  --workflow-file "$FAST_WORKFLOW" \
  --ci-tools-file "$transport_missing_ci_tools" \
  --strategy-doc "$STRATEGY_DOC" \
  --plan-doc "$PLAN_DOC" \
  --max-seconds 120 >"$transport_missing_log" 2>&1; then
  cat "$transport_missing_log" >&2
  echo "expected missing transport composition command fixture to fail checker" >&2
  exit 1
fi
if ! grep -Fq "transport_ci_smoke_composition_missing" "$transport_missing_log"; then
  cat "$transport_missing_log" >&2
  echo "expected missing transport composition command fixture to emit deterministic transport reason code" >&2
  exit 1
fi

observability_missing_ci_tools="$(mktemp)"
grep -Fv 'bash "$ROOT_DIR/scripts/ci/test_check_observability_endpoint_drift_contract.sh"' "$CI_TOOLS_SCRIPT" >"$observability_missing_ci_tools"

observability_missing_log="$(mktemp)"
if python3 "$CHECKER" \
  --workflow-file "$FAST_WORKFLOW" \
  --ci-tools-file "$observability_missing_ci_tools" \
  --strategy-doc "$STRATEGY_DOC" \
  --plan-doc "$PLAN_DOC" \
  --max-seconds 120 >"$observability_missing_log" 2>&1; then
  cat "$observability_missing_log" >&2
  echo "expected missing observability composition command fixture to fail checker" >&2
  exit 1
fi
if ! grep -Fq "observability_ci_smoke_composition_missing" "$observability_missing_log"; then
  cat "$observability_missing_log" >&2
  echo "expected missing observability composition command fixture to emit deterministic observability reason code" >&2
  exit 1
fi

tls_missing_ci_tools="$(mktemp)"
grep -Fv 'bash "$ROOT_DIR/scripts/ci/test_check_kamn_core_live_https_dependency_posture.sh"' "$CI_TOOLS_SCRIPT" >"$tls_missing_ci_tools"

tls_missing_log="$(mktemp)"
if python3 "$CHECKER" \
  --workflow-file "$FAST_WORKFLOW" \
  --ci-tools-file "$tls_missing_ci_tools" \
  --strategy-doc "$STRATEGY_DOC" \
  --plan-doc "$PLAN_DOC" \
  --max-seconds 120 >"$tls_missing_log" 2>&1; then
  cat "$tls_missing_log" >&2
  echo "expected missing tls composition command fixture to fail checker" >&2
  exit 1
fi
if ! grep -Fq "tls_ci_smoke_composition_missing" "$tls_missing_log"; then
  cat "$tls_missing_log" >&2
  echo "expected missing tls composition command fixture to emit deterministic tls reason code" >&2
  exit 1
fi

leaked_transport_heavy_ci_tools="$(mktemp)"
awk '
  BEGIN { inserted = 0 }
  {
    if (!inserted && $0 ~ /echo "Fast-mode CI tool regression tests passed\."/) {
      print "  bash \"$ROOT_DIR/scripts/runtime/test_validate_live_transport_fault_matrix_live_contract_lane.sh\""
      inserted = 1
    }
    print $0
  }
' "$CI_TOOLS_SCRIPT" >"$leaked_transport_heavy_ci_tools"

leaked_transport_heavy_log="$(mktemp)"
if python3 "$CHECKER" \
  --workflow-file "$FAST_WORKFLOW" \
  --ci-tools-file "$leaked_transport_heavy_ci_tools" \
  --strategy-doc "$STRATEGY_DOC" \
  --plan-doc "$PLAN_DOC" \
  --max-seconds 120 >"$leaked_transport_heavy_log" 2>&1; then
  cat "$leaked_transport_heavy_log" >&2
  echo "expected leaked transport local-heavy command in ci-tools fast mode fixture to fail checker" >&2
  exit 1
fi
if ! grep -Fq "transport_local_heavy_command_leaked_in_fast_mode" "$leaked_transport_heavy_log"; then
  cat "$leaked_transport_heavy_log" >&2
  echo "expected leaked transport local-heavy command fixture to emit deterministic leakage reason code" >&2
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
if ! grep -Fq "transport_observability_tls_ci_smoke_seconds_exceeded" "$budget_overflow_log"; then
  cat "$budget_overflow_log" >&2
  echo "expected max-seconds overflow fixture to emit deterministic budget reason code" >&2
  exit 1
fi

rm -f \
  "$safe_report" "$safe_log" \
  "$transport_missing_ci_tools" "$transport_missing_log" \
  "$observability_missing_ci_tools" "$observability_missing_log" \
  "$tls_missing_ci_tools" "$tls_missing_log" \
  "$leaked_transport_heavy_ci_tools" "$leaked_transport_heavy_log" \
  "$budget_overflow_log"

echo "transport/observability/tls ci smoke convergence checker tests passed."
