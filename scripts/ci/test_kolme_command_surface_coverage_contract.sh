#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"
FAST_WORKFLOW="$ROOT_DIR/.github/workflows/ci-fast-gate.yml"
CI_TOOLS_SCRIPT="$ROOT_DIR/scripts/ci/test_ci_tools.sh"

test_harness_require_file "$FAST_WORKFLOW" "expected fast-gate workflow to exist"

test_harness_require_file "$CI_TOOLS_SCRIPT" "expected aggregate CI tools script to exist"

mapfile -t kolme_tests < <(find "$ROOT_DIR/scripts/kolme" -maxdepth 1 -type f -name 'test_*.sh' | sort)
if [ "${#kolme_tests[@]}" -eq 0 ]; then
  echo "expected Kolme command-surface test scripts under scripts/kolme" >&2
  exit 1
fi

covered_in_both=0
covered_in_fast_only=0
covered_in_ci_tools_only=0
missing_both=0
missing_scripts=()

for script_path in "${kolme_tests[@]}"; do
  relative_path="${script_path#"$ROOT_DIR/"}"
  in_fast=false
  in_ci_tools=false

  if grep -Fq "$relative_path" "$FAST_WORKFLOW"; then
    in_fast=true
  fi
  if grep -Fq "$relative_path" "$CI_TOOLS_SCRIPT"; then
    in_ci_tools=true
  fi

  if [ "$in_fast" = true ] && [ "$in_ci_tools" = true ]; then
    covered_in_both=$((covered_in_both + 1))
  elif [ "$in_fast" = true ]; then
    covered_in_fast_only=$((covered_in_fast_only + 1))
  elif [ "$in_ci_tools" = true ]; then
    covered_in_ci_tools_only=$((covered_in_ci_tools_only + 1))
  else
    missing_both=$((missing_both + 1))
    missing_scripts+=("$relative_path")
  fi
done

echo "kolme_test_total=${#kolme_tests[@]}"
echo "covered_in_both=$covered_in_both"
echo "covered_in_fast_only=$covered_in_fast_only"
echo "covered_in_ci_tools_only=$covered_in_ci_tools_only"
echo "missing_both=$missing_both"

if [ "$missing_both" -ne 0 ]; then
  echo "expected every scripts/kolme/test_*.sh script to be covered by fast-gate or aggregate ci-tools" >&2
  for missing_script in "${missing_scripts[@]}"; do
    echo "missing_both_script=$missing_script" >&2
  done
  exit 1
fi

echo "kolme command-surface coverage contract tests passed."
