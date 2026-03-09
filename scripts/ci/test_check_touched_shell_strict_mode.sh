#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"

CHECKER="$ROOT_DIR/scripts/ci/check_touched_shell_strict_mode.sh"
PY_CHECKER="$ROOT_DIR/scripts/ci/check_touched_shell_strict_mode.py"
EXCEPTION_FILE="$ROOT_DIR/fixtures/ci/touched_shell_strict_mode_exceptions.txt"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT
TEST_EXCEPTION_FILE="$TMP_DIR/exceptions.txt"

test_harness_require_executable "$CHECKER" "expected touched-shell strict-mode checker wrapper to be executable"
test_harness_require_executable "$PY_CHECKER" "expected touched-shell strict-mode python checker to be executable"
test_harness_require_file "$EXCEPTION_FILE" "expected touched-shell strict-mode exception fixture to exist"

cat >"$TEST_EXCEPTION_FILE" <<'EOF'
scripts/lib/common.sh
EOF

REPO_DIR="$TMP_DIR/repo"
mkdir -p "$REPO_DIR/scripts/demo" "$REPO_DIR/scripts/lib"
(
  cd "$REPO_DIR"
  git init -q
  git config user.name "KAMN Test"
  git config user.email "kamn-test@example.com"
  cat >scripts/demo/pass.sh <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

echo "ok"
EOF
  cat >scripts/lib/common.sh <<'EOF'
#!/usr/bin/env bash

echo "library"
EOF
  chmod +x scripts/demo/pass.sh scripts/lib/common.sh
  git add scripts/demo/pass.sh scripts/lib/common.sh
  git commit -q -m "base"
  git branch -M main
)

cat >"$REPO_DIR/scripts/demo/pass.sh" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

echo "still ok"
EOF

bash "$CHECKER" \
  --repo-root "$REPO_DIR" \
  --base-ref main \
  --exception-file "$TEST_EXCEPTION_FILE" \
  --output-json "$TMP_DIR/pass.json" >"$TMP_DIR/pass.out"
grep -q '^status=pass$' "$TMP_DIR/pass.out"
grep -q '^reason_codes=none$' "$TMP_DIR/pass.out"

cat >"$REPO_DIR/scripts/demo/pass.sh" <<'EOF'
#!/usr/bin/env bash

echo "missing strict mode"
EOF

if bash "$CHECKER" \
  --repo-root "$REPO_DIR" \
  --base-ref main \
  --exception-file "$TEST_EXCEPTION_FILE" \
  --output-json "$TMP_DIR/fail.json" >"$TMP_DIR/fail.out" 2>&1; then
  echo "expected checker to fail for touched scripts missing strict mode" >&2
  exit 1
fi
grep -q '^status=fail$' "$TMP_DIR/fail.out"
grep -q '^reason_codes=touched_shell_strict_mode_missing_strict_mode$' "$TMP_DIR/fail.out"
grep -q '^offending_shell_scripts=scripts/demo/pass.sh$' "$TMP_DIR/fail.out"

git -C "$REPO_DIR" checkout -- scripts/demo/pass.sh scripts/lib/common.sh
cat >"$REPO_DIR/scripts/lib/common.sh" <<'EOF'
#!/usr/bin/env bash

echo "still a sourced library"
EOF

bash "$CHECKER" \
  --repo-root "$REPO_DIR" \
  --base-ref main \
  --exception-file "$TEST_EXCEPTION_FILE" \
  --output-json "$TMP_DIR/exempt.json" >"$TMP_DIR/exempt.out"
grep -q '^status=pass$' "$TMP_DIR/exempt.out"
grep -q '^exempted_shell_scripts=scripts/lib/common.sh$' "$TMP_DIR/exempt.out"

cat >"$TMP_DIR/bad-exceptions.txt" <<'EOF'
not-a-shell-path.txt
EOF

if bash "$CHECKER" \
  --repo-root "$REPO_DIR" \
  --base-ref main \
  --exception-file "$TMP_DIR/bad-exceptions.txt" \
  --output-json "$TMP_DIR/bad-exceptions.json" >"$TMP_DIR/bad-exceptions.out" 2>&1; then
  echo "expected checker to fail for invalid exception metadata" >&2
  exit 1
fi
grep -q '^status=fail$' "$TMP_DIR/bad-exceptions.out"
grep -q '^reason_codes=touched_shell_strict_mode_exception_file_invalid$' "$TMP_DIR/bad-exceptions.out"

if bash "$CHECKER" \
  --repo-root "$REPO_DIR" \
  --base-ref does-not-exist \
  --exception-file "$TEST_EXCEPTION_FILE" \
  --output-json "$TMP_DIR/bad-base.json" >"$TMP_DIR/bad-base.out" 2>&1; then
  echo "expected checker to fail when git base cannot be resolved" >&2
  exit 1
fi
grep -q '^status=fail$' "$TMP_DIR/bad-base.out"
grep -q '^reason_codes=touched_shell_strict_mode_git_base_unavailable$' "$TMP_DIR/bad-base.out"

echo "touched-shell strict-mode checker tests passed."
