#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_WORKFLOW="$ROOT_DIR/.github/workflows/ci-fast-gate.yml"
MAKEFILE="$ROOT_DIR/Makefile"

fail() {
  echo "$1" >&2
  exit 1
}

extract_make_target() {
  local target="$1"
  awk '
    $0 == target ":" { in_target=1; print; next }
    in_target && /^[^[:space:]].*:/ { exit }
    in_target { print }
  ' target="$target" "$MAKEFILE"
}

workflow_has_pr_trigger() {
  local file="$1"
  awk '
    /^on:/ { in_on=1; next }
    in_on && /^[^[:space:]]/ { in_on=0 }
    in_on && $1 ~ /^pull_request:/ { found=1 }
    END { exit found ? 0 : 1 }
  ' "$file"
}

if [ -f "$FAST_WORKFLOW" ] && workflow_has_pr_trigger "$FAST_WORKFLOW"; then
  fail "ci-fast-gate GitHub workflow must not run on pull_request; run local gates with make pre-push"
fi

pre_push_target="$(extract_make_target "pre-push")"
if [ -z "$pre_push_target" ]; then
  fail "expected Makefile pre-push target for local gate execution"
fi

for marker in \
  "PRE_PUSH_PYTHON3_CANDIDATES" \
  "PRE_PUSH_WORKSPACE_TARGET_DIR" \
  "PRE_PUSH_WORKSPACE_TIMEOUT_SECONDS" \
  "python@3.12/libexec/bin/python3"; do
  if ! grep -Fq "$marker" "$MAKEFILE"; then
    fail "expected Makefile to include local pre-push configuration marker: $marker"
  fi
done

for marker in \
  "PRE_PUSH_PYTHON3" \
  "import cryptography" \
  "tomllib" \
  "\$(MAKE) check" \
  "\$(MAKE) ci-tools" \
  'CARGO_TARGET_DIR="$(PRE_PUSH_WORKSPACE_TARGET_DIR)"' \
  'timeout "$(PRE_PUSH_WORKSPACE_TIMEOUT_SECONDS)"' \
  "cargo test --workspace --locked --all-features --no-fail-fast" \
  "bash scripts/ci/check_touched_rust_size_policy.sh" \
  "bash scripts/ci/run_critical_path_coverage_gate.sh" \
  "bash scripts/ci/run_critical_path_mutation_gate.sh"; do
  if ! grep -Fq "$marker" <<<"$pre_push_target"; then
    fail "expected make pre-push to include local gate marker: $marker"
  fi
done

check_target="$(extract_make_target "check")"
if [ -z "$check_target" ]; then
  fail "expected Makefile check target for local static gate execution"
fi

for marker in \
  "cargo fmt --check" \
  "cargo clippy --workspace --all-targets --all-features -- -D warnings"; do
  if ! grep -Fq "$marker" <<<"$check_target"; then
    fail "expected make check to include static gate marker: $marker"
  fi
done

echo "local pre-push gate policy tests passed."
