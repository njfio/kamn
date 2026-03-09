#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"

CHECKER="$ROOT_DIR/scripts/ci/check_governance_feature_commit_ratio.py"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

test_harness_require_executable "$CHECKER" "expected governance/feature commit-ratio checker to be executable"

run_checker() {
  local subjects_file="$1"
  local window_size="$2"
  local max_governance_ratio="$3"
  local output_json="$4"
  python3 "$CHECKER" \
    --commit-subjects-file "$subjects_file" \
    --window-size "$window_size" \
    --max-governance-ratio "$max_governance_ratio" \
    --output-json "$output_json"
}

run_range_checker() {
  local repo_root="$1"
  local base_sha="$2"
  local head_sha="$3"
  local window_size="$4"
  local max_governance_ratio="$5"
  local output_json="$6"
  python3 "$CHECKER" \
    --repo-root "$repo_root" \
    --base-sha "$base_sha" \
    --head-sha "$head_sha" \
    --window-size "$window_size" \
    --max-governance-ratio "$max_governance_ratio" \
    --output-json "$output_json"
}

init_history_repo() {
  local repo_dir="$1"
  git init -q "$repo_dir"
  git -C "$repo_dir" config user.name "KAMN Test"
  git -C "$repo_dir" config user.email "kamn-test@example.com"
  printf 'root\n' >"$repo_dir/README.md"
  git -C "$repo_dir" add README.md
  git -C "$repo_dir" commit -q -m "chore(test): seed repo"
}

commit_repo_file() {
  local repo_dir="$1"
  local subject="$2"
  local file_path="$3"
  local file_body="$4"
  mkdir -p "$repo_dir/$(dirname "$file_path")"
  printf '%s\n' "$file_body" >"$repo_dir/$file_path"
  git -C "$repo_dir" add "$file_path"
  git -C "$repo_dir" commit -q -m "$subject"
}

commit_repo_two_files() {
  local repo_dir="$1"
  local subject="$2"
  local first_path="$3"
  local first_body="$4"
  local second_path="$5"
  local second_body="$6"
  mkdir -p "$repo_dir/$(dirname "$first_path")" "$repo_dir/$(dirname "$second_path")"
  printf '%s\n' "$first_body" >"$repo_dir/$first_path"
  printf '%s\n' "$second_body" >"$repo_dir/$second_path"
  git -C "$repo_dir" add "$first_path" "$second_path"
  git -C "$repo_dir" commit -q -m "$subject"
}

assert_output_contains() {
  local output="$1"
  local pattern="$2"
  local message="$3"
  if ! printf '%s\n' "$output" | grep -q "^$pattern$"; then
    echo "$message" >&2
    printf '%s\n' "$output" >&2
    exit 1
  fi
}

assert_failure() {
  local label="$1"
  local subjects_file="$2"
  local window_size="$3"
  local max_governance_ratio="$4"
  local output_json="$5"
  local message="$6"
  if run_checker "$subjects_file" "$window_size" "$max_governance_ratio" "$output_json" >"$TMP_DIR/$label.out" 2>"$TMP_DIR/$label.err"; then
    echo "$message" >&2
    cat "$TMP_DIR/$label.out" >&2 || true
    cat "$TMP_DIR/$label.err" >&2 || true
    exit 1
  fi
}

assert_activation_scope_ok() {
  local output="$1"
  local activation_scope_status="$2"
  local status_message="$3"
  local scope_message="$4"
  assert_output_contains "$output" 'status=ok' "$status_message"
  assert_output_contains "$output" "activation_scope_status=$activation_scope_status" "$scope_message"
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

HISTORY_REPO="$TMP_DIR/history-repo"
RANGE_OUTPUT_JSON="$TMP_DIR/range-report.json"
init_history_repo "$HISTORY_REPO"
BASE_SHA="$(git -C "$HISTORY_REPO" rev-parse HEAD)"

commit_repo_file "$HISTORY_REPO" "feat(ci): governance-only surface with feature-looking prefix" "scripts/ci/policy.sh" "echo governance"
GOVERNANCE_ONLY_SHA="$(git -C "$HISTORY_REPO" rev-parse HEAD)"

if run_range_checker "$HISTORY_REPO" "$BASE_SHA" "$GOVERNANCE_ONLY_SHA" 50 0.20 "$RANGE_OUTPUT_JSON" >"$TMP_DIR/range-governance.out" 2>"$TMP_DIR/range-governance.err"; then
  echo "expected governance-only path history to fail even when the commit prefix looks like feature work" >&2
  cat "$TMP_DIR/range-governance.out" >&2 || true
  cat "$TMP_DIR/range-governance.err" >&2 || true
  exit 1
fi
if ! grep -q '^governance_commit_count=1$' "$TMP_DIR/range-governance.out"; then
  echo "expected governance-only history to classify as governance work" >&2
  cat "$TMP_DIR/range-governance.out" >&2 || true
  exit 1
fi

commit_repo_file "$HISTORY_REPO" "docs(runtime): capability surface with governance-looking prefix" "crates/kamn-core/src/capability.rs" "pub fn capability_marker() {}"
CAPABILITY_SHA="$(git -C "$HISTORY_REPO" rev-parse HEAD)"
capability_output="$(run_range_checker "$HISTORY_REPO" "$GOVERNANCE_ONLY_SHA" "$CAPABILITY_SHA" 50 0.20 "$RANGE_OUTPUT_JSON")"
assert_output_contains "$capability_output" 'status=ok' "expected capability-surface history to pass even when the commit prefix looks like governance work"
assert_output_contains "$capability_output" 'feature_commit_count=1' "expected capability-surface history to classify as capability work"

commit_repo_two_files "$HISTORY_REPO" "chore(ci): mixed governance and capability surfaces" \
  "crates/kamn-core/src/mixed.rs" "pub fn mixed_marker() {}" \
  "scripts/ci/mixed.sh" "echo more governance"
MIXED_SHA="$(git -C "$HISTORY_REPO" rev-parse HEAD)"
mixed_output="$(run_range_checker "$HISTORY_REPO" "$CAPABILITY_SHA" "$MIXED_SHA" 50 0.20 "$RANGE_OUTPUT_JSON")"
assert_output_contains "$mixed_output" 'status=ok' "expected mixed-surface history to classify as capability work"
assert_output_contains "$mixed_output" 'feature_commit_count=1' "expected mixed-surface history to count as capability work"

ACTIVATION_REPO="$TMP_DIR/activation-repo"
ACTIVATION_OUTPUT_JSON="$TMP_DIR/activation-report.json"
init_history_repo "$ACTIVATION_REPO"
PREACTIVATION_BASE_SHA="$(git -C "$ACTIVATION_REPO" rev-parse HEAD)"

commit_repo_file "$ACTIVATION_REPO" "docs(ci): preactivation governance prep" "scripts/ci/preactivation.sh" "echo preactivation"
PREACTIVATION_SHA="$(git -C "$ACTIVATION_REPO" rev-parse HEAD)"

commit_repo_file "$ACTIVATION_REPO" "feat(ci): rollout activation policy" "scripts/ci/activation.sh" "echo activation"
ACTIVATION_SHA="$(git -C "$ACTIVATION_REPO" rev-parse HEAD)"

activation_base_output="$(run_range_checker "$ACTIVATION_REPO" "$ACTIVATION_SHA" "$ACTIVATION_SHA" 50 0.20 "$ACTIVATION_OUTPUT_JSON")"
assert_activation_scope_ok \
  "$activation_base_output" \
  'head_at_activation_base' \
  "expected activation-base head to produce a non-violating result" \
  "expected activation-base head to emit explicit activation-scope status"

preactivation_output="$(run_range_checker "$ACTIVATION_REPO" "$ACTIVATION_SHA" "$PREACTIVATION_SHA" 50 0.20 "$ACTIVATION_OUTPUT_JSON")"
assert_activation_scope_ok \
  "$preactivation_output" \
  'head_precedes_activation_base' \
  "expected preactivation head to produce a non-violating historical result" \
  "expected preactivation head to emit explicit historical activation-scope status"

commit_repo_file "$ACTIVATION_REPO" "docs(ci): post-activation governance drift" "scripts/ci/post_activation.sh" "echo post activation"
POSTACTIVATION_GOVERNANCE_SHA="$(git -C "$ACTIVATION_REPO" rev-parse HEAD)"

if run_range_checker "$ACTIVATION_REPO" "$ACTIVATION_SHA" "$POSTACTIVATION_GOVERNANCE_SHA" 50 0.20 "$ACTIVATION_OUTPUT_JSON" >"$TMP_DIR/postactivation-governance.out" 2>"$TMP_DIR/postactivation-governance.err"; then
  echo "expected post-activation governance-only history to keep failing the ratio gate" >&2
  cat "$TMP_DIR/postactivation-governance.out" >&2 || true
  cat "$TMP_DIR/postactivation-governance.err" >&2 || true
  exit 1
fi
if ! grep -q '^activation_scope_status=post_activation_window$' "$TMP_DIR/postactivation-governance.out"; then
  echo "expected post-activation governance-only history to report post_activation_window scope" >&2
  cat "$TMP_DIR/postactivation-governance.out" >&2 || true
  exit 1
fi
if ! grep -q '^reason_codes_csv=governance_commit_ratio_threshold_exceeded$' "$TMP_DIR/postactivation-governance.out"; then
  echo "expected post-activation governance-only history to preserve deterministic threshold reason code" >&2
  cat "$TMP_DIR/postactivation-governance.out" >&2 || true
  exit 1
fi

echo "governance/feature commit-ratio checker tests passed."
