#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATOR="$ROOT_DIR/scripts/kolme/validate_triadic_devnet_smoke.py"
FIXTURE_FILE="$ROOT_DIR/fixtures/kolme_compatibility/devnet_smoke_markers.json"
VALID_MARKERS="$(mktemp)"
MISSING_MARKERS="$(mktemp)"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$VALID_MARKERS" "$MISSING_MARKERS" "$TMP_REPORT"' EXIT

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

if [ ! -x "$VALIDATOR" ]; then
  echo "expected triadic devnet smoke validator to be executable" >&2
  exit 1
fi

if [ ! -f "$FIXTURE_FILE" ]; then
  echo "expected triadic devnet smoke marker fixture to exist" >&2
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
