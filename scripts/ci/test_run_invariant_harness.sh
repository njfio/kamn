#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/ci/run_invariant_harness.sh"

fast_output="$(bash "$SCRIPT" --mode fast --dry-run)"
if [ "$(printf '%s\n' "$fast_output" | wc -l | tr -d ' ')" -ne 1 ]; then
  echo "expected one command in fast mode" >&2
  exit 1
fi
if ! printf '%s\n' "$fast_output" | grep -q 'KAMN_INVARIANT_SEED=13'; then
  echo "expected fast mode to use seed 13" >&2
  exit 1
fi

deep_output="$(bash "$SCRIPT" --mode deep --dry-run)"
if [ "$(printf '%s\n' "$deep_output" | wc -l | tr -d ' ')" -ne 3 ]; then
  echo "expected three commands in deep mode" >&2
  exit 1
fi
for seed in 13 97 401; do
  if ! printf '%s\n' "$deep_output" | grep -q "KAMN_INVARIANT_SEED=$seed"; then
    echo "expected deep mode to include seed $seed" >&2
    exit 1
  fi
done

set +e
bash "$SCRIPT" --mode invalid --dry-run >/dev/null 2>&1
invalid_status=$?
set -e
if [ "$invalid_status" -eq 0 ]; then
  echo "expected invalid mode to fail" >&2
  exit 1
fi

echo "run_invariant_harness tests passed."
