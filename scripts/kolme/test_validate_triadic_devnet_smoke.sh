#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)/scripts/lib/common.sh"
ROOT_DIR="$KAMN_ROOT"
VALIDATOR="$ROOT_DIR/scripts/kolme/validate_triadic_devnet_smoke.py"
POLICY_CHECKER="$ROOT_DIR/scripts/kolme/check_triadic_devnet_smoke_policy.py"
FIXTURE_FILE="$ROOT_DIR/fixtures/kolme_compatibility/devnet_smoke_markers.json"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
CI_DOC_FILE="$ROOT_DIR/docs/ci/strategy.md"
VALID_MARKERS="$(mktemp)"
MISSING_MARKERS="$(mktemp)"
TMP_REPORT="$(mktemp)"
TMP_POLICY_REPORT="$(mktemp)"
trap 'rm -f "$VALID_MARKERS" "$MISSING_MARKERS" "$TMP_REPORT" "$TMP_POLICY_REPORT"' EXIT

if [ ! -x "$VALIDATOR" ]; then
  echo "expected triadic devnet smoke validator to be executable" >&2
  exit 1
fi

if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected triadic devnet smoke policy checker to be executable" >&2
  exit 1
fi

if [ ! -f "$FIXTURE_FILE" ]; then
  echo "expected triadic devnet smoke marker fixture to exist" >&2
  exit 1
fi

if ! grep -q "check_triadic_devnet_smoke_policy.py" "$DOC_FILE"; then
  echo "expected devnet ops docs to reference triadic devnet smoke policy checker command" >&2
  exit 1
fi

if ! grep -q "check_triadic_devnet_smoke_policy.py" "$CI_DOC_FILE"; then
  echo "expected CI strategy docs to reference triadic devnet smoke policy checker command" >&2
  exit 1
fi

cat >"$VALID_MARKERS" <<'EOF'
marker_startup=ok
marker_tx_progression=ok
marker_block_commit=ok
marker_teardown=ok
status=pass
EOF

go_output="$(
  python3 "$VALIDATOR" \
    --fixture "$FIXTURE_FILE" \
    --marker-file "$VALID_MARKERS" \
    --output-json "$TMP_REPORT"
)"
assert_eq "$(extract_value "$go_output" "status")" "ok" "expected valid marker file to pass"
assert_eq "$(extract_value "$go_output" "final_decision")" "PASS" "expected PASS decision for valid marker file"

policy_output="$(
  python3 "$POLICY_CHECKER" \
    --report-file "$TMP_REPORT" \
    --expected-final-decision GO \
    --require-reason-code triadic_devnet_smoke_policy_passed \
    --output-json "$TMP_POLICY_REPORT"
)"
assert_eq "$(extract_value "$policy_output" "status")" "ok" "expected triadic devnet smoke policy checker to pass for valid report"
assert_eq "$(extract_value "$policy_output" "final_decision")" "GO" "expected triadic devnet smoke policy checker GO decision for valid report"

cat >"$MISSING_MARKERS" <<'EOF'
marker_startup=ok
marker_block_commit=ok
status=pass
EOF

set +e
fail_output="$(
  python3 "$VALIDATOR" \
    --fixture "$FIXTURE_FILE" \
    --marker-file "$MISSING_MARKERS" \
    --output-json "$TMP_REPORT" 2>&1
)"
fail_code=$?
set -e

if [ "$fail_code" -eq 0 ]; then
  echo "expected missing marker input to fail closed" >&2
  exit 1
fi
assert_eq "$(extract_value "$fail_output" "status")" "fail" "expected fail status for missing markers"
assert_eq "$(extract_value "$fail_output" "final_decision")" "FAIL" "expected FAIL decision for missing markers"

if ! printf '%s\n' "$fail_output" | grep -q "marker_tx_progression=ok"; then
  echo "expected missing marker list to include tx progression marker" >&2
  exit 1
fi

set +e
policy_fail_output="$(
  python3 "$POLICY_CHECKER" \
    --report-file "$TMP_REPORT" \
    --expected-final-decision GO \
    --output-json "$TMP_POLICY_REPORT" 2>&1
)"
policy_fail_code=$?
set -e

if [ "$policy_fail_code" -eq 0 ]; then
  echo "expected triadic devnet smoke policy checker to fail for report with missing markers" >&2
  exit 1
fi

if ! printf '%s\n' "$policy_fail_output" | grep -q "report_missing_markers_non_empty"; then
  echo "expected deterministic report_missing_markers_non_empty reason code for policy failure" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text())
if payload.get("schema_version") != "kamn.kolme.triadic-devnet-smoke-validation-report.v1":
    raise SystemExit("unexpected triadic devnet smoke validator report schema")
if payload.get("final_decision") != "FAIL":
    raise SystemExit("expected persisted FAIL decision for missing marker case")
PY

# Regression: #785
if ! printf '%s\n' "$fail_output" | grep -q "missing_markers"; then
  echo "expected marker drift regression output contract" >&2
  exit 1
fi

echo "triadic devnet smoke validator tests passed."
