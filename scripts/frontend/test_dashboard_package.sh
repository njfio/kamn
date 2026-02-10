#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

NODE_BIN="${KAMN_DASHBOARD_NODE_BIN:-node}"
FALLBACK_NODE_CMD="${KAMN_DASHBOARD_FALLBACK_NODE_CMD:-npx -y node@22 --}"
TEST_TARGET_GLOB="${KAMN_DASHBOARD_TEST_TARGET_GLOB:-./packages/kamn-dashboard/tests/*.test.ts}"

supports_strip_types() {
  local -a cmd=("$@")
  "${cmd[@]}" --experimental-strip-types -e "" >/dev/null 2>&1
}

run_dashboard_tests() {
  local -a cmd=("$@")
  # shellcheck disable=SC2086
  "${cmd[@]}" --experimental-strip-types --test $TEST_TARGET_GLOB
}

if supports_strip_types "$NODE_BIN"; then
  run_dashboard_tests "$NODE_BIN"
  exit 0
fi

IFS=' ' read -r -a fallback_parts <<<"$FALLBACK_NODE_CMD"
if [ "${#fallback_parts[@]}" -eq 0 ]; then
  echo "dashboard fallback Node command is empty" >&2
  exit 1
fi

if supports_strip_types "${fallback_parts[@]}"; then
  run_dashboard_tests "${fallback_parts[@]}"
  exit 0
fi

echo "dashboard node runtime does not support --experimental-strip-types and fallback failed" >&2
echo "set KAMN_DASHBOARD_FALLBACK_NODE_CMD to a Node 22+ command" >&2
exit 1
