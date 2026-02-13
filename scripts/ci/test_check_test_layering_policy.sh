#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$ROOT_DIR/scripts/ci/check_test_layering_policy.py"
POLICY_DOC="$ROOT_DIR/docs/planning/test_layering_policy.md"
STRATEGY_DOC="$ROOT_DIR/docs/ci/strategy.md"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$CHECKER" ]; then
  echo "expected test-layering policy checker to be executable" >&2
  exit 1
fi

python3 "$CHECKER" \
  --policy-doc "$POLICY_DOC" \
  --strategy-doc "$STRATEGY_DOC" \
  --output-json "$TMP_DIR/policy-check.json" \
  >/dev/null

BROKEN_POLICY_DOC="$TMP_DIR/test_layering_policy_broken.md"
cp "$POLICY_DOC" "$BROKEN_POLICY_DOC"
python3 - "$BROKEN_POLICY_DOC" <<'PY'
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
path.write_text(text.replace("unit_hotspots_required=true", "unit_hotspots_required=false"), encoding="utf-8")
PY

if python3 "$CHECKER" \
  --policy-doc "$BROKEN_POLICY_DOC" \
  --strategy-doc "$STRATEGY_DOC" \
  --output-json "$TMP_DIR/broken-policy-check.json" \
  >"$TMP_DIR/broken.out" \
  2>"$TMP_DIR/broken.err"; then
  echo "expected checker to fail when required layering marker drifts" >&2
  cat "$TMP_DIR/broken.out" >&2 || true
  cat "$TMP_DIR/broken.err" >&2 || true
  exit 1
fi

if ! grep -q "layering_marker_missing" "$TMP_DIR/broken.err"; then
  echo "expected layering_marker_missing reason in checker stderr output" >&2
  cat "$TMP_DIR/broken.err" >&2 || true
  exit 1
fi

echo "test-layering policy checker tests passed."
