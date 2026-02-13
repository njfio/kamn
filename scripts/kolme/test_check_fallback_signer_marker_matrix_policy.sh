#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/kolme/check_fallback_signer_marker_matrix_policy.py"
MATRIX_FIXTURE="$ROOT_DIR/fixtures/kolme_compatibility/fallback_signer_marker_matrix.json"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$SCRIPT" ]; then
  echo "expected fallback signer marker matrix policy checker to be executable" >&2
  exit 1
fi

if [ ! -f "$MATRIX_FIXTURE" ]; then
  echo "expected fallback signer marker matrix fixture to exist" >&2
  exit 1
fi

if ! grep -q "check_fallback_signer_marker_matrix_policy.py" "$DOC_FILE"; then
  echo "expected Kolme devnet ops docs to reference fallback signer marker matrix policy checker" >&2
  exit 1
fi

if ! grep -q "fallback_signer_marker_matrix.json" "$DOC_FILE"; then
  echo "expected Kolme devnet ops docs to reference fallback signer marker matrix fixture" >&2
  exit 1
fi

python3 "$SCRIPT" \
  --matrix-file "$MATRIX_FIXTURE"

INVALID_CLASSIFICATION_MATRIX="$TMP_DIR/invalid-classification.json"
cp "$MATRIX_FIXTURE" "$INVALID_CLASSIFICATION_MATRIX"
python3 - "$INVALID_CLASSIFICATION_MATRIX" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["markers"][0]["classification"] = "legacy"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

if python3 "$SCRIPT" --matrix-file "$INVALID_CLASSIFICATION_MATRIX" >"$TMP_DIR/invalid-classification.out" 2>"$TMP_DIR/invalid-classification.err"; then
  echo "expected fallback signer marker matrix policy checker to fail for invalid classification" >&2
  cat "$TMP_DIR/invalid-classification.out" >&2 || true
  cat "$TMP_DIR/invalid-classification.err" >&2 || true
  exit 1
fi

MISSING_REQUIRED_MARKER_MATRIX="$TMP_DIR/missing-required-marker.json"
cp "$MATRIX_FIXTURE" "$MISSING_REQUIRED_MARKER_MATRIX"
python3 - "$MISSING_REQUIRED_MARKER_MATRIX" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["markers"] = [
    marker
    for marker in payload.get("markers", [])
    if marker.get("marker_id") != "runtime_commit_fallback_private_key_command_marker_detected"
]
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

if python3 "$SCRIPT" --matrix-file "$MISSING_REQUIRED_MARKER_MATRIX" >"$TMP_DIR/missing-required.out" 2>"$TMP_DIR/missing-required.err"; then
  echo "expected fallback signer marker matrix policy checker to fail when required marker is missing" >&2
  cat "$TMP_DIR/missing-required.out" >&2 || true
  cat "$TMP_DIR/missing-required.err" >&2 || true
  exit 1
fi

echo "fallback signer marker matrix policy checker tests passed."
