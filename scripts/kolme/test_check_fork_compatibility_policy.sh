#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/kolme/generate_fork_compatibility_evidence.py"
CHECKER="$ROOT_DIR/scripts/kolme/check_fork_compatibility_policy.py"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

extract_value() {
  local output="$1"
  local key="$2"
  printf '%s\n' "$output" | awk -F= -v key="$key" '$1 == key { print $2; exit }'
}

assert_eq() {
  local actual="$1"
  local expected="$2"
  local message="$3"
  if [ "$actual" != "$expected" ]; then
    echo "$message: expected '$expected', got '$actual'" >&2
    exit 1
  fi
}

if [ ! -x "$GENERATOR" ]; then
  echo "expected fork compatibility evidence generator to be executable" >&2
  exit 1
fi

if [ ! -x "$CHECKER" ]; then
  echo "expected fork compatibility policy checker to be executable" >&2
  exit 1
fi

go_report="$TMP_DIR/fork-go-report.json"
python3 "$GENERATOR" \
  --upstream-release-tag "v0.15.2" \
  --fork-release-tag "v0.15.2" \
  --fork-repo "njfio/kolme_fork" \
  --fork-ref "refs/heads/main" \
  --ci-fast-gate PASS \
  --output-json "$go_report" \
  >/dev/null

go_policy_output="$(
  python3 "$CHECKER" \
    --report-file "$go_report" \
    --expected-upstream-release-tag "v0.15.2" \
    --expected-fork-release-tag "v0.15.2" \
    --expected-fork-repo "njfio/kolme_fork" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/policy-go-report.json"
)"
assert_eq "$(extract_value "$go_policy_output" "status")" "ok" "expected policy checker to accept go report"
assert_eq "$(extract_value "$go_policy_output" "final_decision")" "GO" "expected GO policy decision for go report"

drift_report="$TMP_DIR/fork-drift-report.json"
set +e
python3 "$GENERATOR" \
  --upstream-release-tag "v0.15.2" \
  --fork-release-tag "v0.14.9" \
  --fork-repo "njfio/kolme_fork" \
  --fork-ref "refs/heads/main" \
  --ci-fast-gate PASS \
  --output-json "$drift_report" \
  >/dev/null
generator_code=$?
set -e
if [ "$generator_code" -eq 0 ]; then
  echo "expected drifted fork tuple generation to fail closed" >&2
  exit 1
fi

drift_policy_output="$(
  python3 "$CHECKER" \
    --report-file "$drift_report" \
    --expected-upstream-release-tag "v0.15.2" \
    --expected-fork-release-tag "v0.14.9" \
    --expected-fork-repo "njfio/kolme_fork" \
    --expected-final-decision NO-GO \
    --require-reason-code fork_release_tag_mismatch \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/policy-drift-report.json"
)"
assert_eq "$(extract_value "$drift_policy_output" "status")" "ok" "expected policy checker to accept expected no-go report"
assert_eq "$(extract_value "$drift_policy_output" "final_decision")" "GO" "expected GO policy decision when expected no-go reason code is present"

malformed_report="$TMP_DIR/malformed-report.json"
cat <<'JSON' >"$malformed_report"
{
  "schema_version": "kamn.kolme.fork-compatibility-report.v0",
  "upstream_release_tag": "v0.15.2",
  "fork_release_tag": "v0.15.2",
  "fork_repo": "njfio/kolme_fork",
  "fork_ref": "refs/heads/main",
  "reason_codes": [],
  "final_decision": "GO"
}
JSON

set +e
malformed_output="$(
  python3 "$CHECKER" \
    --report-file "$malformed_report" \
    --expected-upstream-release-tag "v0.15.2" \
    --expected-fork-release-tag "v0.15.2" \
    --expected-fork-repo "njfio/kolme_fork" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/policy-malformed-report.json" 2>&1
)"
malformed_code=$?
set -e
if [ "$malformed_code" -eq 0 ]; then
  echo "expected malformed report to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$malformed_output" | grep -q "report_schema_invalid"; then
  echo "expected malformed report to emit report_schema_invalid reason code" >&2
  exit 1
fi

python3 - "$TMP_DIR/policy-malformed-report.json" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text())
if report.get("schema_version") != "kamn.kolme.fork-compatibility-policy-report.v1":
    raise SystemExit("unexpected fork compatibility policy report schema")
if report.get("final_decision") != "NO-GO":
    raise SystemExit("expected malformed report policy decision to be NO-GO")
PY

# Regression: #1402
if ! printf '%s\n' "$malformed_output" | grep -q "report_schema_invalid"; then
  echo "expected malformed schema regression guard to remain fail-closed" >&2
  exit 1
fi

echo "Kolme fork compatibility policy checker tests passed."
