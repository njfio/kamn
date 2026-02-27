#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"

CHECKER="$ROOT_DIR/scripts/ci/check_review_document_freeze.py"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

test_harness_require_executable "$CHECKER" "expected review-document freeze checker to be executable"

MANIFEST_OK="$TMP_DIR/review-document-freeze.manifest"
MANIFEST_INVALID="$TMP_DIR/review-document-freeze-invalid.manifest"
CHANGED_PASS="$TMP_DIR/changed-pass.txt"
CHANGED_BLOCKED="$TMP_DIR/changed-blocked.txt"
PASS_JSON="$TMP_DIR/pass-report.json"
BLOCKED_JSON="$TMP_DIR/blocked-report.json"
INVALID_JSON="$TMP_DIR/invalid-report.json"
MISSING_JSON="$TMP_DIR/missing-report.json"

cat >"$MANIFEST_OK" <<'EOF'
review_document_freeze_manifest_schema_version=kamn.review.review-document-freeze-manifest.v1
review_document_freeze_entries_csv=gaps-and-issues-r51.md,gaps-and-issues-r52.md
EOF

cat >"$MANIFEST_INVALID" <<'EOF'
review_document_freeze_manifest_schema_version=kamn.review.review-document-freeze-manifest.v1
review_document_freeze_entries_csv=
EOF

cat >"$CHANGED_PASS" <<'EOF'
docs/review/gaps-and-issues-r57.md
docs/ci/strategy.md
EOF

cat >"$CHANGED_BLOCKED" <<'EOF'
docs/review/gaps-and-issues-r52.md
scripts/ci/test_check_review_document_freeze.sh
EOF

pass_output="$(
  python3 "$CHECKER" \
    --changed-files-file "$CHANGED_PASS" \
    --freeze-manifest "$MANIFEST_OK" \
    --output-json "$PASS_JSON"
)"
if ! printf '%s\n' "$pass_output" | grep -q '^status=ok$'; then
  echo "expected status=ok when changed files do not hit frozen entries" >&2
  printf '%s\n' "$pass_output" >&2
  exit 1
fi
if ! printf '%s\n' "$pass_output" | grep -q '^reason_codes_csv=none$'; then
  echo "expected reason_codes_csv=none for pass fixture" >&2
  printf '%s\n' "$pass_output" >&2
  exit 1
fi

if python3 "$CHECKER" \
  --changed-files-file "$CHANGED_BLOCKED" \
  --freeze-manifest "$MANIFEST_OK" \
  --output-json "$BLOCKED_JSON" \
  >"$TMP_DIR/blocked.out" \
  2>"$TMP_DIR/blocked.err"
then
  echo "expected checker to fail when frozen review document is changed" >&2
  cat "$TMP_DIR/blocked.out" >&2 || true
  cat "$TMP_DIR/blocked.err" >&2 || true
  exit 1
fi
if ! grep -q 'review_document_freeze_violation_detected' "$TMP_DIR/blocked.out"; then
  echo "expected frozen-doc violation reason code" >&2
  cat "$TMP_DIR/blocked.out" >&2 || true
  exit 1
fi

if python3 "$CHECKER" \
  --changed-files-file "$CHANGED_PASS" \
  --freeze-manifest "$MANIFEST_INVALID" \
  --output-json "$INVALID_JSON" \
  >"$TMP_DIR/invalid.out" \
  2>"$TMP_DIR/invalid.err"
then
  echo "expected checker to fail on invalid freeze manifest" >&2
  cat "$TMP_DIR/invalid.out" >&2 || true
  cat "$TMP_DIR/invalid.err" >&2 || true
  exit 1
fi
if ! grep -q 'review_document_freeze_manifest_invalid' "$TMP_DIR/invalid.out"; then
  echo "expected invalid-manifest reason code" >&2
  cat "$TMP_DIR/invalid.out" >&2 || true
  exit 1
fi

if python3 "$CHECKER" \
  --changed-files-file "$CHANGED_PASS" \
  --freeze-manifest "$TMP_DIR/missing.manifest" \
  --output-json "$MISSING_JSON" \
  >"$TMP_DIR/missing.out" \
  2>"$TMP_DIR/missing.err"
then
  echo "expected checker to fail when manifest file is missing" >&2
  cat "$TMP_DIR/missing.out" >&2 || true
  cat "$TMP_DIR/missing.err" >&2 || true
  exit 1
fi
if ! grep -q 'review_document_freeze_manifest_missing' "$TMP_DIR/missing.out"; then
  echo "expected missing-manifest reason code" >&2
  cat "$TMP_DIR/missing.out" >&2 || true
  exit 1
fi

python3 - "$PASS_JSON" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.ci.review-document-freeze-gate-report.v1":
    raise SystemExit("expected deterministic schema_version marker")
if payload.get("reason_taxonomy_version") != "kamn.ci.review-document-freeze-gate-reason-taxonomy.v1":
    raise SystemExit("expected deterministic reason taxonomy marker")
if payload.get("reason_codes_csv") != "none":
    raise SystemExit("expected reason_codes_csv=none in pass payload")
if payload.get("frozen_entry_count") != 2:
    raise SystemExit("expected frozen_entry_count=2")
if payload.get("blocked_changed_files") != []:
    raise SystemExit("expected no blocked_changed_files for pass payload")
PY

echo "review-document freeze checker tests passed."
