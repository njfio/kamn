#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"

CHECKER="$ROOT_DIR/scripts/ci/check_touched_shell_strict_mode.sh"
PY_CHECKER="$ROOT_DIR/scripts/ci/check_touched_shell_strict_mode.py"
EXCEPTION_FILE="$ROOT_DIR/fixtures/ci/touched_shell_strict_mode_exceptions.txt"

test_harness_require_executable "$CHECKER" "expected touched-shell strict-mode checker wrapper to be executable"
test_harness_require_executable "$PY_CHECKER" "expected touched-shell strict-mode python checker to be executable"
test_harness_require_file "$EXCEPTION_FILE" "expected touched-shell strict-mode exception fixture to exist"
