#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$ROOT_DIR/scripts/ci/check_workflow_kolme_heavy_exclusion_policy.py"
FAST_WORKFLOW="$ROOT_DIR/.github/workflows/ci-fast-gate.yml"
SELECTOR_FILE="$ROOT_DIR/scripts/ci/select_targets.sh"
SAFE_FIXTURE="$ROOT_DIR/fixtures/ci/workflow_kolme_heavy_policy_safe.yml"
UNSAFE_MISSING_INPUT_FIXTURE="$ROOT_DIR/fixtures/ci/workflow_kolme_heavy_policy_unsafe_missing_input.yml"
UNSAFE_FORCED_TRUE_FIXTURE="$ROOT_DIR/fixtures/ci/workflow_kolme_heavy_policy_unsafe_forced_true.yml"
UNSAFE_VERSION_LANE_MATRIX_FIXTURE="$ROOT_DIR/fixtures/ci/workflow_kolme_heavy_policy_unsafe_version_lane_matrix.yml"
UNSAFE_MISSING_LOCAL_HEAVY_COMMAND_FIXTURE="$ROOT_DIR/fixtures/ci/workflow_kolme_heavy_policy_unsafe_missing_local_heavy_command.yml"
UNSAFE_SELECTOR_MISSING_LOCAL_HEAVY_COMMAND_FIXTURE="$ROOT_DIR/fixtures/ci/workflow_kolme_heavy_policy_selector_unsafe_missing_local_heavy_command.sh"

if [ ! -x "$CHECKER" ]; then
  echo "expected workflow policy checker to be executable" >&2
  exit 1
fi

safe_report="$(mktemp)"
safe_log="$(mktemp)"
if ! python3 "$CHECKER" --workflow-file "$SAFE_FIXTURE" --selector-file "$SELECTOR_FILE" --output-json "$safe_report" >"$safe_log" 2>&1; then
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
if ! grep -Fq "reason_codes=none" "$safe_log"; then
  cat "$safe_log" >&2
  echo "expected safe workflow fixture reason_codes=none" >&2
  exit 1
fi

real_report="$(mktemp)"
real_log="$(mktemp)"
if ! python3 "$CHECKER" --workflow-file "$FAST_WORKFLOW" --selector-file "$SELECTOR_FILE" --output-json "$real_report" >"$real_log" 2>&1; then
  cat "$real_log" >&2
  echo "expected ci-fast-gate workflow to satisfy local-heavy exclusion policy" >&2
  exit 1
fi
if ! grep -Fq "status=pass" "$real_log"; then
  cat "$real_log" >&2
  echo "expected ci-fast-gate workflow policy status=pass" >&2
  exit 1
fi
if ! grep -Fq "reason_codes=none" "$real_log"; then
  cat "$real_log" >&2
  echo "expected ci-fast-gate workflow reason_codes=none" >&2
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

missing_local_heavy_command_log="$(mktemp)"
if python3 "$CHECKER" --workflow-file "$UNSAFE_MISSING_LOCAL_HEAVY_COMMAND_FIXTURE" >"$missing_local_heavy_command_log" 2>&1; then
  cat "$missing_local_heavy_command_log" >&2
  echo "expected missing-local-heavy-command fixture to fail policy checker" >&2
  exit 1
fi
if ! grep -Fq "local_heavy_lane_commands_missing" "$missing_local_heavy_command_log"; then
  cat "$missing_local_heavy_command_log" >&2
  echo "expected missing-local-heavy-command fixture to report local_heavy_lane_commands_missing" >&2
  exit 1
fi

version_lane_log="$(mktemp)"
if python3 "$CHECKER" --workflow-file "$UNSAFE_VERSION_LANE_MATRIX_FIXTURE" >"$version_lane_log" 2>&1; then
  cat "$version_lane_log" >&2
  echo "expected version-lane matrix fixture to fail policy checker" >&2
  exit 1
fi
if ! grep -Fq "local_heavy_lane_commands_in_version_lane" "$version_lane_log"; then
  cat "$version_lane_log" >&2
  echo "expected version-lane matrix fixture to report local_heavy_lane_commands_in_version_lane" >&2
  exit 1
fi

rm -f "$safe_report" "$safe_log" "$real_report" "$real_log" "$missing_input_log" "$forced_true_log" "$missing_local_heavy_command_log" "$version_lane_log"

selector_missing_local_heavy_command_log="$(mktemp)"
# Regression: #2388
if python3 "$CHECKER" --workflow-file "$SAFE_FIXTURE" --selector-file "$UNSAFE_SELECTOR_MISSING_LOCAL_HEAVY_COMMAND_FIXTURE" >"$selector_missing_local_heavy_command_log" 2>&1; then
  cat "$selector_missing_local_heavy_command_log" >&2
  echo "expected selector fixture missing local-heavy command to fail policy checker" >&2
  exit 1
fi
if ! grep -Fq "selector_local_heavy_commands_missing" "$selector_missing_local_heavy_command_log"; then
  cat "$selector_missing_local_heavy_command_log" >&2
  echo "expected selector fixture missing local-heavy command to report selector_local_heavy_commands_missing" >&2
  exit 1
fi
if ! grep -Fq "reason_codes=selector_local_heavy_commands_missing" "$selector_missing_local_heavy_command_log"; then
  cat "$selector_missing_local_heavy_command_log" >&2
  echo "expected selector fixture missing local-heavy command to emit deterministic reason code marker" >&2
  exit 1
fi

rm -f "$selector_missing_local_heavy_command_log"
echo "workflow Kolme local-heavy exclusion policy checker tests passed."
