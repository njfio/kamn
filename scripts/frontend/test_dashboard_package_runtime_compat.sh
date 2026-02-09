#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT_UNDER_TEST="$ROOT_DIR/scripts/frontend/test_dashboard_package.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$SCRIPT_UNDER_TEST" ]; then
  echo "expected dashboard package test script to be executable" >&2
  exit 1
fi

create_fake_node() {
  local path="$1"
  local mode="$2"
  local marker_file="$3"
  local marker_value="$4"

  cat >"$path" <<'FAKE_NODE'
#!/usr/bin/env bash
set -euo pipefail

MODE="$1"
MARKER_FILE="$2"
MARKER_VALUE="$3"
shift 3

if [[ "${1:-}" == "--experimental-strip-types" && "${2:-}" == "-e" ]]; then
  if [[ "$MODE" == "support" ]]; then
    exit 0
  fi
  echo "node: bad option: --experimental-strip-types" >&2
  exit 9
fi

if [[ "${1:-}" == "--experimental-strip-types" && "${2:-}" == "--test" ]]; then
  if [[ "$MODE" == "support" ]]; then
    if [[ -n "$MARKER_FILE" ]]; then
      printf '%s\n' "$MARKER_VALUE" >>"$MARKER_FILE"
    fi
    exit 0
  fi
  echo "node: bad option: --experimental-strip-types" >&2
  exit 9
fi

exit 0
FAKE_NODE

  cat <<EOF_WRAPPER >"$path.wrapper"
#!/usr/bin/env bash
set -euo pipefail
exec "$path" "$mode" "$marker_file" "$marker_value" "\$@"
EOF_WRAPPER

  chmod +x "$path" "$path.wrapper"
}

assert_contains() {
  local file="$1"
  local expected="$2"
  local message="$3"
  if ! grep -Fq "$expected" "$file"; then
    echo "$message" >&2
    exit 1
  fi
}

marker_file="$TMP_DIR/markers.log"
primary_binary="$TMP_DIR/primary-node"
fallback_binary="$TMP_DIR/fallback-node"

create_fake_node "$primary_binary" "support" "$marker_file" "primary"
create_fake_node "$fallback_binary" "reject" "$marker_file" "fallback"

KAMN_DASHBOARD_NODE_BIN="$primary_binary.wrapper" \
KAMN_DASHBOARD_FALLBACK_NODE_CMD="$fallback_binary.wrapper" \
KAMN_DASHBOARD_TEST_TARGET_GLOB="./packages/kamn-dashboard/tests/dashboard.test.ts" \
  bash "$SCRIPT_UNDER_TEST"

assert_contains "$marker_file" "primary" "expected dashboard package script to use primary node when strip-types is supported"
if grep -Fq "fallback" "$marker_file"; then
  echo "expected dashboard package script to avoid fallback node when primary supports strip-types" >&2
  exit 1
fi

: >"$marker_file"
create_fake_node "$primary_binary" "reject" "$marker_file" "primary"
create_fake_node "$fallback_binary" "support" "$marker_file" "fallback"

KAMN_DASHBOARD_NODE_BIN="$primary_binary.wrapper" \
KAMN_DASHBOARD_FALLBACK_NODE_CMD="$fallback_binary.wrapper" \
KAMN_DASHBOARD_TEST_TARGET_GLOB="./packages/kamn-dashboard/tests/dashboard.test.ts" \
  bash "$SCRIPT_UNDER_TEST"

assert_contains "$marker_file" "fallback" "expected dashboard package script to fall back when primary node lacks strip-types support"
if grep -Fq "primary" "$marker_file"; then
  echo "expected dashboard package script to avoid primary node execution when strip-types probing fails" >&2
  exit 1
fi

: >"$marker_file"
create_fake_node "$primary_binary" "reject" "$marker_file" "primary"
create_fake_node "$fallback_binary" "reject" "$marker_file" "fallback"

set +e
failure_output="$(
  KAMN_DASHBOARD_NODE_BIN="$primary_binary.wrapper" \
  KAMN_DASHBOARD_FALLBACK_NODE_CMD="$fallback_binary.wrapper" \
  KAMN_DASHBOARD_TEST_TARGET_GLOB="./packages/kamn-dashboard/tests/dashboard.test.ts" \
    bash "$SCRIPT_UNDER_TEST" 2>&1
)"
failure_code=$?
set -e

if [ "$failure_code" -eq 0 ]; then
  echo "expected dashboard package script to fail when both primary and fallback nodes reject strip-types" >&2
  exit 1
fi

if ! printf '%s\n' "$failure_output" | grep -q "dashboard node runtime does not support --experimental-strip-types"; then
  echo "expected explicit runtime compatibility failure message when both node commands reject strip-types" >&2
  exit 1
fi

# Regression: #866
if ! printf '%s\n' "$failure_output" | grep -q "set KAMN_DASHBOARD_FALLBACK_NODE_CMD"; then
  echo "expected remediation guidance in dashboard runtime compatibility failure path" >&2
  exit 1
fi

echo "dashboard package runtime compatibility tests passed."
