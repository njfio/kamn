#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"

CHECKER="$ROOT_DIR/scripts/ci/check_governance_feature_commit_ratio.py"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

test_harness_require_executable "$CHECKER" "expected governance/feature commit-ratio checker to be executable"

PASS_SUBJECTS="$TMP_DIR/pass-subjects.txt"
FAIL_RATIO_SUBJECTS="$TMP_DIR/fail-ratio-subjects.txt"
UNKNOWN_SUBJECTS="$TMP_DIR/unknown-subjects.txt"

PASS_OUTPUT_JSON="$TMP_DIR/pass-report.json"
FAIL_RATIO_OUTPUT_JSON="$TMP_DIR/fail-ratio-report.json"
UNKNOWN_OUTPUT_JSON="$TMP_DIR/unknown-report.json"

cat >"$PASS_SUBJECTS" <<'EOF'
feat(runtime): wire relay forwarding status projection (#6001)
spec(6001): define relay forwarding conformance contract (#6001)
EOF

cat >"$FAIL_RATIO_SUBJECTS" <<'EOF'
spec(6002): define runtime closure contract (#6002)
docs(ci): update governance report markers (#6002)
fix(runtime): enforce relay retry boundary (#6002)
EOF

cat >"$UNKNOWN_SUBJECTS" <<'EOF'
wip(runtime): experiment with temporary marker
feat(runtime): keep deterministic selector behavior
EOF

pass_output="$(
  python3 "$CHECKER" \
    --commit-subjects-file "$PASS_SUBJECTS" \
    --max-governance-ratio 0.50 \
    --output-json "$PASS_OUTPUT_JSON"
)"
if ! printf '%s\n' "$pass_output" | grep -q '^status=ok$'; then
  echo "expected status=ok when governance ratio is at threshold" >&2
  printf '%s\n' "$pass_output" >&2
  exit 1
fi
if ! printf '%s\n' "$pass_output" | grep -q '^reason_codes_csv=none$'; then
  echo "expected reason_codes_csv=none for passing fixture" >&2
  printf '%s\n' "$pass_output" >&2
  exit 1
fi

if python3 "$CHECKER" \
  --commit-subjects-file "$FAIL_RATIO_SUBJECTS" \
  --max-governance-ratio 0.50 \
  --output-json "$FAIL_RATIO_OUTPUT_JSON" \
  >"$TMP_DIR/fail-ratio.out" \
  2>"$TMP_DIR/fail-ratio.err"
then
  echo "expected checker to fail when governance ratio exceeds threshold" >&2
  cat "$TMP_DIR/fail-ratio.out" >&2 || true
  cat "$TMP_DIR/fail-ratio.err" >&2 || true
  exit 1
fi
if ! grep -q '^reason_codes_csv=governance_commit_ratio_threshold_exceeded$' "$TMP_DIR/fail-ratio.out"; then
  echo "expected deterministic threshold reason code for ratio failure" >&2
  cat "$TMP_DIR/fail-ratio.out" >&2 || true
  exit 1
fi

if python3 "$CHECKER" \
  --commit-subjects-file "$UNKNOWN_SUBJECTS" \
  --max-governance-ratio 0.50 \
  --output-json "$UNKNOWN_OUTPUT_JSON" \
  >"$TMP_DIR/unknown.out" \
  2>"$TMP_DIR/unknown.err"
then
  echo "expected checker to fail on unknown commit prefix classification" >&2
  cat "$TMP_DIR/unknown.out" >&2 || true
  cat "$TMP_DIR/unknown.err" >&2 || true
  exit 1
fi
if ! grep -q 'governance_commit_subject_unclassified' "$TMP_DIR/unknown.out"; then
  echo "expected unclassified commit reason code on unknown prefix" >&2
  cat "$TMP_DIR/unknown.out" >&2 || true
  exit 1
fi

python3 - "$PASS_OUTPUT_JSON" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.ci.governance-feature-commit-ratio-report.v1":
    raise SystemExit("expected deterministic schema_version")
if payload.get("reason_codes_csv") != "none":
    raise SystemExit("expected reason_codes_csv=none for pass payload")
if payload.get("non_merge_commit_total") != 2:
    raise SystemExit("expected non_merge_commit_total=2")
if payload.get("governance_commit_count") != 1:
    raise SystemExit("expected governance_commit_count=1")
if payload.get("feature_commit_count") != 1:
    raise SystemExit("expected feature_commit_count=1")
if payload.get("governance_ratio") != 0.5:
    raise SystemExit("expected governance_ratio=0.5")
if payload.get("feature_ratio") != 0.5:
    raise SystemExit("expected feature_ratio=0.5")
if payload.get("max_governance_ratio") != 0.5:
    raise SystemExit("expected max_governance_ratio=0.5")
PY

echo "governance/feature commit-ratio checker tests passed."
