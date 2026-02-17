#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/ci/check_pr_ci_declaration.sh"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

# Non-PR event should skip and pass
GITHUB_EVENT_NAME=push GITHUB_EVENT_PATH="$TMP_DIR/missing.json" "$SCRIPT" >/dev/null

bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$TMP_DIR/pass.json" <<'JSON'
{
  "pull_request": {
    "body": "## CI Impact Declaration\n- [ ] No CI scope impact.\n- [x] CI scope impact present (explain below).\n\nWorkflow(s) touched: ci-fast-gate\nExpected runtime delta: +30s\nExpected runner-minute delta: +1\nRollback plan if CI cost/runtime regresses: revert workflow change"
  }
}
JSON

GITHUB_EVENT_NAME=pull_request GITHUB_EVENT_PATH="$TMP_DIR/pass.json" CI_DECLARATION_FORCE_SENSITIVE=true "$SCRIPT" >/dev/null

bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$TMP_DIR/fail_not_marked.json" <<'JSON'
{
  "pull_request": {
    "body": "## CI Impact Declaration\n- [x] No CI scope impact.\n- [ ] CI scope impact present (explain below)."
  }
}
JSON

if GITHUB_EVENT_NAME=pull_request GITHUB_EVENT_PATH="$TMP_DIR/fail_not_marked.json" CI_DECLARATION_FORCE_SENSITIVE=true "$SCRIPT" >/dev/null 2>&1; then
  echo "Expected failure when CI impact is not marked present" >&2
  exit 1
fi

bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$TMP_DIR/fail_both.json" <<'JSON'
{
  "pull_request": {
    "body": "## CI Impact Declaration\n- [x] No CI scope impact.\n- [x] CI scope impact present (explain below).\n\nWorkflow(s) touched: ci-fast-gate\nExpected runtime delta: +1m\nExpected runner-minute delta: +2\nRollback plan if CI cost/runtime regresses: revert"
  }
}
JSON

if GITHUB_EVENT_NAME=pull_request GITHUB_EVENT_PATH="$TMP_DIR/fail_both.json" CI_DECLARATION_FORCE_SENSITIVE=true "$SCRIPT" >/dev/null 2>&1; then
  echo "Expected failure when both CI impact options are checked" >&2
  exit 1
fi

bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$TMP_DIR/fail_missing_field.json" <<'JSON'
{
  "pull_request": {
    "body": "## CI Impact Declaration\n- [ ] No CI scope impact.\n- [x] CI scope impact present (explain below).\n\nWorkflow(s) touched: ci-fast-gate\nExpected runtime delta: +1m\nExpected runner-minute delta: \nRollback plan if CI cost/runtime regresses: revert"
  }
}
JSON

if GITHUB_EVENT_NAME=pull_request GITHUB_EVENT_PATH="$TMP_DIR/fail_missing_field.json" CI_DECLARATION_FORCE_SENSITIVE=true "$SCRIPT" >/dev/null 2>&1; then
  echo "Expected failure when a required CI impact field is blank" >&2
  exit 1
fi

echo "check_pr_ci_declaration tests passed."
