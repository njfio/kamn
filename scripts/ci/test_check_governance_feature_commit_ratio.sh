#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"

CHECKER="$ROOT_DIR/scripts/ci/check_governance_feature_commit_ratio.py"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

test_harness_require_executable "$CHECKER" "expected governance/feature commit-ratio checker to be executable"

run_checker() {
  python3 "$CHECKER" \
    --commit-subjects-file "$1" \
    --window-size "$2" \
    --max-governance-ratio "$3" \
    --output-json "$4"
}

assert_output_contains() {
  if ! printf '%s\n' "$1" | grep -q "^$2$"; then
    echo "$3" >&2
    printf '%s\n' "$1" >&2
    exit 1
  fi
}

assert_failure() {
  if run_checker "$2" "$3" "$4" "$5" >"$TMP_DIR/$1.out" 2>"$TMP_DIR/$1.err"; then
    echo "$6" >&2
    cat "$TMP_DIR/$1.out" >&2 || true
    cat "$TMP_DIR/$1.err" >&2 || true
    exit 1
  fi
}

PASS_SUBJECTS="$TMP_DIR/pass-subjects.txt"
INTEGRATE_SUBJECTS="$TMP_DIR/integrate-subjects.txt"
FAIL_RATIO_SUBJECTS="$TMP_DIR/fail-ratio-subjects.txt"
UNKNOWN_SUBJECTS="$TMP_DIR/unknown-subjects.txt"
EMPTY_SUBJECTS="$TMP_DIR/empty-subjects.txt"

PASS_OUTPUT_JSON="$TMP_DIR/pass-report.json"
INTEGRATE_OUTPUT_JSON="$TMP_DIR/integrate-report.json"
FAIL_RATIO_OUTPUT_JSON="$TMP_DIR/fail-ratio-report.json"
UNKNOWN_OUTPUT_JSON="$TMP_DIR/unknown-report.json"
EMPTY_OUTPUT_JSON="$TMP_DIR/empty-report.json"

: >"$PASS_SUBJECTS"
for i in $(seq 1 40); do
  printf 'feat(runtime): capability moratorium feature commit %02d\n' "$i" >>"$PASS_SUBJECTS"
done
for i in $(seq 1 10); do
  printf 'docs(ci): capability moratorium governance commit %02d\n' "$i" >>"$PASS_SUBJECTS"
done
for i in $(seq 1 20); do
  printf 'spec(6546): older governance tail commit %02d\n' "$i" >>"$PASS_SUBJECTS"
done

: >"$FAIL_RATIO_SUBJECTS"
for i in $(seq 1 39); do
  printf 'feat(runtime): insufficient capability share commit %02d\n' "$i" >>"$FAIL_RATIO_SUBJECTS"
done
for i in $(seq 1 11); do
  printf 'docs(ci): threshold breach governance commit %02d\n' "$i" >>"$FAIL_RATIO_SUBJECTS"
done

cat >"$INTEGRATE_SUBJECTS" <<'EOF'
integrate(6003): wire relay forwarding into default runtime lane
feat(runtime): keep deterministic selector behavior
EOF

cat >"$UNKNOWN_SUBJECTS" <<'EOF'
wip(runtime): experiment with temporary marker
feat(runtime): keep deterministic selector behavior
EOF

: >"$EMPTY_SUBJECTS"

pass_output="$(run_checker "$PASS_SUBJECTS" 50 0.20 "$PASS_OUTPUT_JSON")"
assert_output_contains "$pass_output" 'status=ok' "expected status=ok at the 80/20 capability moratorium threshold"
assert_output_contains "$pass_output" 'reason_codes_csv=none' "expected reason_codes_csv=none for passing fixture"
assert_output_contains "$pass_output" 'non_merge_commit_total=50' "expected pass fixture to evaluate exactly the latest 50 commit subjects"
assert_output_contains "$pass_output" 'input_non_merge_commit_total=70' "expected pass fixture to report total input subjects before windowing"
assert_output_contains "$pass_output" 'governance_ratio=0.2' "expected pass fixture governance_ratio=0.2 at the moratorium boundary"
assert_output_contains "$pass_output" 'feature_ratio=0.8' "expected pass fixture feature_ratio=0.8 at the moratorium boundary"

integrate_output="$(run_checker "$INTEGRATE_SUBJECTS" 2 0.50 "$INTEGRATE_OUTPUT_JSON")"
assert_output_contains "$integrate_output" 'status=ok' "expected status=ok when integrate commit subjects are classified as feature"
assert_output_contains "$integrate_output" 'unknown_commit_count=0' "expected integrate fixture to avoid unknown classification count"

assert_failure fail-ratio "$FAIL_RATIO_SUBJECTS" 50 0.20 "$FAIL_RATIO_OUTPUT_JSON" "expected checker to fail when the latest 50 commits breach the 80/20 capability moratorium"
if ! grep -q '^reason_codes_csv=governance_commit_ratio_threshold_exceeded$' "$TMP_DIR/fail-ratio.out"; then
  echo "expected deterministic threshold reason code for ratio failure" >&2
  cat "$TMP_DIR/fail-ratio.out" >&2 || true
  exit 1
fi

assert_failure unknown "$UNKNOWN_SUBJECTS" 2 0.50 "$UNKNOWN_OUTPUT_JSON" "expected checker to fail on unknown commit prefix classification"
if ! grep -q 'governance_commit_subject_unclassified' "$TMP_DIR/unknown.out"; then
  echo "expected unclassified commit reason code on unknown prefix" >&2
  cat "$TMP_DIR/unknown.out" >&2 || true
  exit 1
fi

assert_failure empty "$EMPTY_SUBJECTS" 50 0.20 "$EMPTY_OUTPUT_JSON" "expected checker to fail when the commit subject input window is empty"
if ! grep -q '^reason_codes_csv=governance_commit_subjects_empty$' "$TMP_DIR/empty.out"; then
  echo "expected deterministic empty-input reason code" >&2
  cat "$TMP_DIR/empty.out" >&2 || true
  exit 1
fi

python3 - "$PASS_OUTPUT_JSON" "$INTEGRATE_OUTPUT_JSON" <<'PY'
import json
import pathlib
import sys

pass_payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
integrate_payload = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))

if pass_payload.get("schema_version") != "kamn.ci.governance-feature-commit-ratio-report.v1":
    raise SystemExit("expected deterministic schema_version")
if pass_payload.get("reason_codes_csv") != "none":
    raise SystemExit("expected reason_codes_csv=none for pass payload")
if pass_payload.get("non_merge_commit_total") != 50 or pass_payload.get("input_non_merge_commit_total") != 70:
    raise SystemExit("expected pass payload to preserve evaluated and input window counts")
if pass_payload.get("governance_commit_count") != 10 or pass_payload.get("feature_commit_count") != 40:
    raise SystemExit("expected pass payload to preserve 40/10 feature-governance counts")
if pass_payload.get("governance_ratio") != 0.2 or pass_payload.get("feature_ratio") != 0.8:
    raise SystemExit("expected pass payload to preserve 80/20 capability ratios")
if pass_payload.get("max_governance_ratio") != 0.2 or pass_payload.get("window_size") != 50:
    raise SystemExit("expected pass payload to record moratorium thresholds")
if integrate_payload.get("status") != "ok" or integrate_payload.get("unknown_commit_count") != 0:
    raise SystemExit("expected integrate fixture to stay classified as feature work")
if integrate_payload.get("feature_commit_count") != 2 or integrate_payload.get("window_size") != 2:
    raise SystemExit("expected integrate fixture to preserve 2-commit evaluated window")
if "integrate" not in integrate_payload.get("feature_commit_types_csv", "").split(","):
    raise SystemExit("expected feature_commit_types_csv to include integrate")
PY

echo "governance/feature commit-ratio checker tests passed."
