#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$ROOT_DIR/scripts/ci/check_workflow_kolme_heavy_exclusion_policy.py"
FAST_WORKFLOW="$ROOT_DIR/.github/workflows/ci-fast-gate.yml"
SAFE_FIXTURE="$ROOT_DIR/fixtures/ci/workflow_kolme_heavy_policy_safe.yml"
UNSAFE_MISSING_INPUT_FIXTURE="$ROOT_DIR/fixtures/ci/workflow_kolme_heavy_policy_unsafe_missing_input.yml"
UNSAFE_FORCED_TRUE_FIXTURE="$ROOT_DIR/fixtures/ci/workflow_kolme_heavy_policy_unsafe_forced_true.yml"

if [ ! -x "$CHECKER" ]; then
  echo "expected workflow policy checker to be executable" >&2
  exit 1
fi

safe_report="$(mktemp)"
safe_log="$(mktemp)"
if ! python3 "$CHECKER" --workflow-file "$SAFE_FIXTURE" --output-json "$safe_report" >"$safe_log" 2>&1; then
  cat "$safe_log" >&2
  echo "expected safe workflow fixture to pass policy checker" >&2
  exit 1
fi
if ! grep -Fq "status=pass" "$safe_log"; then
  cat "$safe_log" >&2
  echo "expected safe workflow fixture status=pass" >&2
  exit 1
fi
if ! grep -Fq '"final_decision": "GO"' "$safe_report"; then
  cat "$safe_report" >&2
  echo "expected safe workflow fixture JSON decision GO" >&2
  exit 1
fi

real_report="$(mktemp)"
real_log="$(mktemp)"
if ! python3 "$CHECKER" --workflow-file "$FAST_WORKFLOW" --output-json "$real_report" >"$real_log" 2>&1; then
  cat "$real_log" >&2
  echo "expected ci-fast-gate workflow to satisfy local-heavy exclusion policy" >&2
  exit 1
fi
if ! grep -Fq "status=pass" "$real_log"; then
  cat "$real_log" >&2
  echo "expected ci-fast-gate workflow policy status=pass" >&2
  exit 1
fi

missing_input_log="$(mktemp)"
if python3 "$CHECKER" --workflow-file "$UNSAFE_MISSING_INPUT_FIXTURE" >"$missing_input_log" 2>&1; then
  cat "$missing_input_log" >&2
  echo "expected missing-input fixture to fail policy checker" >&2
  exit 1
fi
if ! grep -Fq "workflow_dispatch_input_missing" "$missing_input_log"; then
  cat "$missing_input_log" >&2
  echo "expected missing-input fixture to report workflow_dispatch_input_missing" >&2
  exit 1
fi

forced_true_log="$(mktemp)"
if python3 "$CHECKER" --workflow-file "$UNSAFE_FORCED_TRUE_FIXTURE" >"$forced_true_log" 2>&1; then
  cat "$forced_true_log" >&2
  echo "expected forced-true fixture to fail policy checker" >&2
  exit 1
fi
if ! grep -Fq "selector_opt_in_env_forced_true_literal" "$forced_true_log"; then
  cat "$forced_true_log" >&2
  echo "expected forced-true fixture to report selector_opt_in_env_forced_true_literal" >&2
  exit 1
fi

rm -f "$safe_report" "$safe_log" "$real_report" "$real_log" "$missing_input_log" "$forced_true_log"
echo "workflow Kolme local-heavy exclusion policy checker tests passed."
