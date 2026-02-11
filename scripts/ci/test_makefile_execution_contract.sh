#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
MAKEFILE="$ROOT_DIR/Makefile"

if [ ! -f "$MAKEFILE" ]; then
  echo "Makefile execution contract failed: Makefile is missing." >&2
  exit 1
fi

assert_make_dry_run_contains() {
  local target="$1"
  local expected_snippet="$2"
  local output

  if ! output="$(make -n "$target" 2>&1)"; then
    echo "Makefile execution contract failed: make -n $target failed." >&2
    echo "$output" >&2
    exit 1
  fi

  if ! grep -Fq -- "$expected_snippet" <<<"$output"; then
    echo "Makefile execution contract failed: target '$target' does not resolve expected command snippet '$expected_snippet'." >&2
    echo "Observed dry-run output:" >&2
    echo "$output" >&2
    exit 1
  fi
}

assert_make_dry_run_contains "check" "cargo fmt --check"
assert_make_dry_run_contains "check" "cargo clippy --workspace --all-targets --all-features -- -D warnings"
assert_make_dry_run_contains "test" "cargo test"
assert_make_dry_run_contains "demo" "bash scripts/sdk/run_localhost_signed_demo.sh"
assert_make_dry_run_contains "demo-localhost-transport" "bash scripts/sdk/run_localhost_signed_demo.sh"

echo "Makefile execution contract tests passed."
