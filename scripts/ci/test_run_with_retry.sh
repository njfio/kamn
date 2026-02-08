#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/ci/run_with_retry.sh"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

# Case 1: immediate success
out_file="$TMP_DIR/out_success.txt"
GITHUB_OUTPUT="$out_file" "$SCRIPT" --label immediate --max-attempts 2 -- bash -lc 'exit 0' >/dev/null

grep -q '^retry_attempts<<EOF$' "$out_file"
grep -q '^1$' "$out_file"
grep -q '^retry_used<<EOF$' "$out_file"
grep -q '^false$' "$out_file"
grep -q '^retry_final_status<<EOF$' "$out_file"
grep -q '^passed$' "$out_file"

# Case 2: fail once then pass
counter_file="$TMP_DIR/counter"
cat > "$TMP_DIR/flaky_once.sh" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
f="$1"
if [ ! -f "$f" ]; then
  echo 1 > "$f"
  exit 1
fi
exit 0
SH
chmod +x "$TMP_DIR/flaky_once.sh"

out_file2="$TMP_DIR/out_retry.txt"
GITHUB_OUTPUT="$out_file2" "$SCRIPT" --label flaky-once --max-attempts 2 -- "$TMP_DIR/flaky_once.sh" "$counter_file" >/dev/null

grep -q '^retry_attempts<<EOF$' "$out_file2"
grep -q '^2$' "$out_file2"
grep -q '^retry_used<<EOF$' "$out_file2"
grep -q '^true$' "$out_file2"
grep -q '^retry_final_status<<EOF$' "$out_file2"
grep -q '^passed$' "$out_file2"

# Case 3: always fail
if "$SCRIPT" --label always-fail --max-attempts 2 -- bash -lc 'exit 1' >/dev/null 2>&1; then
  echo "Expected failure for always-fail case" >&2
  exit 1
fi

echo "run_with_retry tests passed."
