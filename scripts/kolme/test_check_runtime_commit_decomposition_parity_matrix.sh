#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$ROOT_DIR/scripts/kolme/check_runtime_commit_decomposition_parity_matrix.py"
MATRIX_FILE="$ROOT_DIR/fixtures/kolme_compatibility/runtime_commit_decomposition_parity_matrix.json"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$CHECKER" ]; then
  echo "expected runtime commit decomposition parity checker to be executable" >&2
  exit 1
fi

if [ ! -f "$MATRIX_FILE" ]; then
  echo "expected runtime commit decomposition parity matrix fixture to exist" >&2
  exit 1
fi

python3 "$CHECKER" check \
  --matrix-file "$MATRIX_FILE" \
  --output-json "$TMP_DIR/parity-policy.json" >"$TMP_DIR/pass.out"

grep -q '^status=pass$' "$TMP_DIR/pass.out"
grep -q '^final_decision=GO$' "$TMP_DIR/pass.out"
grep -q '"schema_version": "kamn.kolme.runtime-commit-decomposition-parity-policy.v1"' \
  "$TMP_DIR/parity-policy.json"

MUTATED_MATRIX="$TMP_DIR/mutated-matrix.json"
cp "$MATRIX_FILE" "$MUTATED_MATRIX"
python3 - "$MUTATED_MATRIX" <<'PY'
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["scenarios"][0]["parity_status"] = "diverged"
path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

if python3 "$CHECKER" check \
  --matrix-file "$MUTATED_MATRIX" \
  --output-json "$TMP_DIR/fail-policy.json" >"$TMP_DIR/fail.out" 2>&1; then
  echo "expected parity checker to fail on parity_status drift" >&2
  exit 1
fi

grep -q '^status=fail$' "$TMP_DIR/fail.out"
grep -q 'parity_status must be preserved' "$TMP_DIR/fail.out"

echo "runtime commit decomposition parity matrix checker tests passed."
