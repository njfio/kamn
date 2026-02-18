#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)/scripts/lib/common.sh"

CHECKER="$KAMN_ROOT/scripts/cutover/check_cutover_ci_exclusion_policy.py"
WORKFLOW_FILE="$KAMN_ROOT/.github/workflows/ci-fast-gate.yml"
CI_TOOLS_FILE="$KAMN_ROOT/scripts/ci/test_ci_tools.sh"
STRATEGY_DOC="$KAMN_ROOT/docs/ci/strategy.md"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$CHECKER" ]; then
  echo "expected cutover ci exclusion policy checker to be executable" >&2
  exit 1
fi
if [ ! -f "$WORKFLOW_FILE" ]; then
  echo "expected ci-fast-gate workflow for cutover ci exclusion policy checks" >&2
  exit 1
fi
if [ ! -f "$CI_TOOLS_FILE" ]; then
  echo "expected ci tools script for cutover ci exclusion policy checks" >&2
  exit 1
fi
if [ ! -f "$STRATEGY_DOC" ]; then
  echo "expected ci strategy docs for cutover ci exclusion policy checks" >&2
  exit 1
fi

baseline_report="$TMP_DIR/cutover-ci-exclusion-policy-pass.json"
baseline_output="$(
  python3 "$CHECKER" \
    --workflow-file "$WORKFLOW_FILE" \
    --ci-tools-file "$CI_TOOLS_FILE" \
    --strategy-doc "$STRATEGY_DOC" \
    --max-seconds 120 \
    --output-json "$baseline_report"
)"

assert_eq "$(extract_value "$baseline_output" "status")" "pass" "expected baseline cutover ci exclusion policy status=pass"
assert_eq "$(extract_value "$baseline_output" "final_decision")" "GO" "expected baseline cutover ci exclusion policy final_decision=GO"
assert_eq "$(extract_value "$baseline_output" "reason_taxonomy_version")" "kamn.ci.cutover-ci-exclusion-policy-reason-taxonomy.v1" "expected cutover ci exclusion policy reason taxonomy marker"
assert_eq "$(extract_value "$baseline_output" "reason_codes_value")" "none" "expected baseline cutover ci exclusion policy reason_codes_value=none"
assert_eq "$(extract_value "$baseline_output" "cutover_contract_lane_in_ci_fast_gate")" "true" "expected cutover contract lane to remain in ci-fast-gate workflow"
assert_eq "$(extract_value "$baseline_output" "cutover_deep_lane_excluded_from_ci_fast_gate")" "true" "expected cutover deep lane exclusion marker in ci-fast-gate workflow"
assert_eq "$(extract_value "$baseline_output" "cutover_contract_test_in_ci_tools")" "true" "expected cutover contract lane test to remain in ci tools"
assert_eq "$(extract_value "$baseline_output" "cutover_policy_test_in_ci_tools")" "true" "expected cutover ci exclusion policy test to remain in ci tools"
assert_eq "$(extract_value "$baseline_output" "cutover_deep_lane_excluded_from_ci_tools")" "true" "expected cutover deep lane exclusion marker in ci tools"
assert_eq "$(extract_value "$baseline_output" "docs_contract_status")" "verified" "expected ci strategy docs contract marker to remain verified"

python3 - "$baseline_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.ci.cutover-ci-exclusion-policy-report.v1":
    raise SystemExit("expected deterministic cutover ci exclusion policy report schema marker")
if payload.get("status") != "pass":
    raise SystemExit("expected baseline cutover ci exclusion policy report status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected baseline cutover ci exclusion policy report final_decision=GO")
if payload.get("reason_codes") != []:
    raise SystemExit("expected baseline cutover ci exclusion policy report reason_codes=[]")
if payload.get("docs_contract_status") != "verified":
    raise SystemExit("expected baseline cutover ci exclusion policy report docs_contract_status=verified")
PY

workflow_missing_contract="$TMP_DIR/ci-fast-gate.missing-cutover-contract.yml"
cp "$WORKFLOW_FILE" "$workflow_missing_contract"
python3 - "$workflow_missing_contract" <<'PY'
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
path.write_text(text.replace("run: bash scripts/cutover/run_cutover_rollback_contract_lane.sh\n", "", 1), encoding="utf-8")
PY

set +e
missing_contract_output="$(
  python3 "$CHECKER" \
    --workflow-file "$workflow_missing_contract" \
    --ci-tools-file "$CI_TOOLS_FILE" \
    --strategy-doc "$STRATEGY_DOC" \
    --max-seconds 120 2>&1
)"
missing_contract_code=$?
set -e
if [ "$missing_contract_code" -eq 0 ]; then
  echo "expected cutover ci exclusion policy checker to fail when contract lane is missing in ci-fast-gate" >&2
  exit 1
fi
if ! printf '%s\n' "$missing_contract_output" | grep -q 'cutover_contract_lane_missing_in_ci_fast_gate'; then
  echo "expected deterministic missing cutover contract-lane reason marker" >&2
  exit 1
fi

workflow_deep_leak="$TMP_DIR/ci-fast-gate.deep-lane-leak.yml"
cp "$WORKFLOW_FILE" "$workflow_deep_leak"
printf '\n      - name: Drift injection cutover deep lane\n        run: bash scripts/cutover/run_cutover_rollback_deep_lane.sh\n' >> "$workflow_deep_leak"

set +e
deep_leak_output="$(
  python3 "$CHECKER" \
    --workflow-file "$workflow_deep_leak" \
    --ci-tools-file "$CI_TOOLS_FILE" \
    --strategy-doc "$STRATEGY_DOC" \
    --max-seconds 120 2>&1
)"
deep_leak_code=$?
set -e
if [ "$deep_leak_code" -eq 0 ]; then
  echo "expected cutover ci exclusion policy checker to fail on deep-lane leakage into ci-fast-gate" >&2
  exit 1
fi
if ! printf '%s\n' "$deep_leak_output" | grep -q 'cutover_rollback_deep_lane_leaked_into_ci_fast_gate'; then
  echo "expected deterministic cutover deep-lane leakage reason marker" >&2
  exit 1
fi

ci_tools_missing_contract_test="$TMP_DIR/test_ci_tools.missing-cutover-contract-test.sh"
cp "$CI_TOOLS_FILE" "$ci_tools_missing_contract_test"
python3 - "$ci_tools_missing_contract_test" <<'PY'
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
path.write_text(text.replace('bash "$ROOT_DIR/scripts/cutover/test_run_cutover_rollback_contract_lane.sh"\n', "", 1), encoding="utf-8")
PY

set +e
missing_ci_tools_contract_output="$(
  python3 "$CHECKER" \
    --workflow-file "$WORKFLOW_FILE" \
    --ci-tools-file "$ci_tools_missing_contract_test" \
    --strategy-doc "$STRATEGY_DOC" \
    --max-seconds 120 2>&1
)"
missing_ci_tools_contract_code=$?
set -e
if [ "$missing_ci_tools_contract_code" -eq 0 ]; then
  echo "expected cutover ci exclusion policy checker to fail when ci tools contract test is missing" >&2
  exit 1
fi
if ! printf '%s\n' "$missing_ci_tools_contract_output" | grep -q 'cutover_contract_test_missing_in_ci_tools'; then
  echo "expected deterministic missing cutover contract-test reason marker" >&2
  exit 1
fi

strategy_docs_drift="$TMP_DIR/ci-strategy.cutover-docs-drift.md"
cp "$STRATEGY_DOC" "$strategy_docs_drift"
python3 - "$strategy_docs_drift" <<'PY'
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
path.write_text(
    text.replace(
        "cutover_ci_exclusion_policy_reason_taxonomy_version=kamn.ci.cutover-ci-exclusion-policy-reason-taxonomy.v1",
        "cutover_ci_exclusion_policy_reason_taxonomy_version=<drifted>",
        1,
    ),
    encoding="utf-8",
)
PY

set +e
docs_drift_output="$(
  python3 "$CHECKER" \
    --workflow-file "$WORKFLOW_FILE" \
    --ci-tools-file "$CI_TOOLS_FILE" \
    --strategy-doc "$strategy_docs_drift" \
    --max-seconds 120 2>&1
)"
docs_drift_code=$?
set -e
if [ "$docs_drift_code" -eq 0 ]; then
  echo "expected cutover ci exclusion policy checker to fail on strategy docs marker drift" >&2
  exit 1
fi
if ! printf '%s\n' "$docs_drift_output" | grep -q 'ci_strategy_cutover_exclusion_markers_missing'; then
  echo "expected deterministic strategy docs drift reason marker for cutover ci exclusion policy" >&2
  exit 1
fi

echo "cutover ci exclusion policy checks passed."
