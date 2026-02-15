#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_WORKFLOW="$ROOT_DIR/.github/workflows/ci-fast-gate.yml"
STRATEGY_DOC="$ROOT_DIR/docs/ci/strategy.md"

if ! grep -Fq "Enforce anti-flake merge gate policy" "$FAST_WORKFLOW"; then
  echo "expected anti-flake merge gate policy step in ci-fast-gate.yml" >&2
  exit 1
fi

required_workflow_snippets=(
  "bash scripts/ci/check_anti_flake_policy.sh"
  "--registry-file .ci/flaky-tests.txt"
  "--expected-final-decision GO"
  "--max-active-entries 0"
  "--output-json ci-anti-flake-policy-report.json"
  "Upload anti-flake policy report"
  "ci-anti-flake-policy-\${{ github.run_id }}-\${{ github.run_attempt }}"
)

for snippet in "${required_workflow_snippets[@]}"; do
  if ! grep -Fq -- "$snippet" "$FAST_WORKFLOW"; then
    echo "expected anti-flake merge gate workflow snippet: $snippet" >&2
    exit 1
  fi
done

required_doc_snippets=(
  "check_anti_flake_policy.sh --registry-file .ci/flaky-tests.txt --expected-final-decision GO --max-active-entries 0 --output-json /tmp/anti-flake-policy-report.json"
  "anti_flake_policy_status=pass|fail"
  "anti_flake_policy_final_decision=GO|NO-GO"
  "active_flaky_entries_exceed_max"
  "expected_final_decision_mismatch"
)

for snippet in "${required_doc_snippets[@]}"; do
  if ! grep -Fq -- "$snippet" "$STRATEGY_DOC"; then
    echo "expected anti-flake policy documentation snippet: $snippet" >&2
    exit 1
  fi
done

echo "anti-flake merge gate policy tests passed."
