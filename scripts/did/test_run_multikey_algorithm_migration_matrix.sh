#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/did/run_multikey_algorithm_migration_matrix.py"
FIXTURE="$ROOT_DIR/fixtures/did_core_conformance/multikey_algorithm_migration_vectors.json"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$SCRIPT" ]; then
  echo "expected multikey algorithm migration matrix runner to be executable" >&2
  exit 1
fi

if [ ! -f "$FIXTURE" ]; then
  echo "expected multikey algorithm migration fixture to exist" >&2
  exit 1
fi

output_json="$TMP_DIR/did-multikey-algorithm-migration-matrix.json"
matrix_output="$(
  python3 "$SCRIPT" \
    --fixture "$FIXTURE" \
    --output-json "$output_json"
)"

if ! printf '%s\n' "$matrix_output" | grep -q "^final_decision=GO$"; then
  echo "expected multikey algorithm migration matrix runner to emit GO final decision" >&2
  exit 1
fi

python3 - "$output_json" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.did.multikey-algorithm-migration-matrix-report.v1":
    raise SystemExit("unexpected multikey algorithm migration matrix schema")
summary = payload.get("summary", {})
if summary.get("mismatch_vectors") != 0:
    raise SystemExit("expected zero mismatch vectors for multikey algorithm migration fixture")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected GO final decision for multikey algorithm migration matrix runner")
PY

echo "multikey algorithm migration matrix tests passed."
