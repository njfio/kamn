#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/runtime/run_dependency_local_heavy_deep_scan_lane.sh"
CHECKER="$ROOT_DIR/scripts/runtime/check_dependency_local_heavy_deep_scan_policy.sh"
STRATEGY_DOC="$ROOT_DIR/docs/ci/strategy.md"
OPS_DOC="$ROOT_DIR/docs/ops/configuration.md"
CI_TOOLS_FILE="$ROOT_DIR/scripts/ci/test_ci_tools.sh"
WORKFLOW_FILE="$ROOT_DIR/.github/workflows/ci-fast-gate.yml"

for required_exec in "$RUNNER" "$CHECKER"; do
  if [ ! -x "$required_exec" ]; then
    echo "expected deep-scan policy script to be executable: $required_exec" >&2
    exit 1
  fi
done
for required_file in "$STRATEGY_DOC" "$OPS_DOC" "$CI_TOOLS_FILE" "$WORKFLOW_FILE"; do
  if [ ! -f "$required_file" ]; then
    echo "expected deep-scan policy source to exist: $required_file" >&2
    exit 1
  fi
done

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT
lane_report="$TMP_DIR/dependency-local-heavy-deep-scan-baseline.json"
policy_report="$TMP_DIR/dependency-local-heavy-deep-scan-policy-report.json"

lane_output="$({
  bash "$RUNNER" \
    --profile baseline \
    --mode dry-run \
    --ci-fast-gate PASS \
    --max-seconds 180 \
    --output-json "$lane_report"
} 2>&1)"
if ! printf '%s\n' "$lane_output" | grep -q '^status=pass$'; then
  echo "expected deep-scan baseline status=pass marker" >&2
  exit 1
fi

policy_output="$({
  bash "$CHECKER" \
    --report-file "$lane_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --strategy-doc "$STRATEGY_DOC" \
    --ops-doc "$OPS_DOC" \
    --ci-tools-file "$CI_TOOLS_FILE" \
    --workflow-file "$WORKFLOW_FILE" \
    --output-json "$policy_report"
} 2>&1)"
for marker in \
  "status=pass" \
  "final_decision=GO" \
  "dependency_local_heavy_deep_scan_policy_status=verified" \
  "dependency_local_heavy_deep_scan_policy_docs_marker_parity_status=verified" \
  "dependency_local_heavy_deep_scan_policy_ci_dry_run_selector_status=verified" \
  "dependency_local_heavy_deep_scan_policy_ci_dry_run_workflow_status=verified" \
  "promotion_decision_reason_mapping_status=verified" \
  "reason_codes_value=none"; do
  if ! printf '%s\n' "$policy_output" | grep -q "^${marker}$"; then
    echo "expected deep-scan policy marker ${marker}" >&2
    exit 1
  fi
done

python3 - "$policy_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.dependency-local-heavy-deep-scan-policy-report.v1":
    raise SystemExit("unexpected deep-scan policy report schema")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected deep-scan policy final_decision=GO")
if payload.get("reason_taxonomy_version") != "kamn.runtime.dependency-local-heavy-deep-scan-policy-reason-taxonomy.v1":
    raise SystemExit("expected deep-scan policy reason taxonomy marker")
PY

tampered_report="$TMP_DIR/dependency-local-heavy-deep-scan-baseline.tampered.json"
cp "$lane_report" "$tampered_report"
python3 - "$tampered_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["profile_status"] = "failed"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_output="$({
  bash "$CHECKER" \
    --report-file "$tampered_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --strategy-doc "$STRATEGY_DOC" \
    --ops-doc "$OPS_DOC" \
    --ci-tools-file "$CI_TOOLS_FILE" \
    --workflow-file "$WORKFLOW_FILE" \
    --output-json "$TMP_DIR/dependency-local-heavy-deep-scan-policy-report.tampered.json"
} 2>&1)"
tampered_code=$?
set -e
if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered deep-scan report to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q 'dependency_local_heavy_deep_scan_policy_profile_contract_mismatch'; then
  echo "expected deterministic fail-closed reason for tampered deep-scan marker" >&2
  exit 1
fi

drifted_ops="$TMP_DIR/ops.drifted.md"
cp "$OPS_DOC" "$drifted_ops"
python3 - "$drifted_ops" <<'PY'
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
updated = text.replace(
    "dependency_local_heavy_deep_scan_required_profiles_csv=baseline,injected-risk",
    "dependency_local_heavy_deep_scan_required_profiles_csv=baseline,drifted",
    1,
)
if text == updated:
    raise SystemExit("failed to drift deep-scan ops marker fixture")
path.write_text(updated, encoding="utf-8")
PY

set +e
ops_drift_output="$({
  bash "$CHECKER" \
    --report-file "$lane_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --strategy-doc "$STRATEGY_DOC" \
    --ops-doc "$drifted_ops" \
    --ci-tools-file "$CI_TOOLS_FILE" \
    --workflow-file "$WORKFLOW_FILE" \
    --output-json "$TMP_DIR/dependency-local-heavy-deep-scan-policy-report.ops-drift.json"
} 2>&1)"
ops_drift_code=$?
set -e
if [ "$ops_drift_code" -eq 0 ]; then
  echo "expected ops drift fixture to fail deep-scan policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$ops_drift_output" | grep -q 'dependency_local_heavy_deep_scan_policy_docs_marker_parity_mismatch'; then
  echo "expected deterministic ops parity drift reason marker" >&2
  exit 1
fi

drifted_strategy="$TMP_DIR/strategy.drifted.md"
cp "$STRATEGY_DOC" "$drifted_strategy"
python3 - "$drifted_strategy" <<'PY'
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
updated = text.replace(
    "dependency_local_heavy_deep_scan_policy_reason_taxonomy_version=kamn.runtime.dependency-local-heavy-deep-scan-policy-reason-taxonomy.v1",
    "dependency_local_heavy_deep_scan_policy_reason_taxonomy_version=drifted",
    1,
)
if text == updated:
    raise SystemExit("failed to drift deep-scan strategy marker fixture")
path.write_text(updated, encoding="utf-8")
PY

set +e
strategy_drift_output="$({
  bash "$CHECKER" \
    --report-file "$lane_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --strategy-doc "$drifted_strategy" \
    --ops-doc "$OPS_DOC" \
    --ci-tools-file "$CI_TOOLS_FILE" \
    --workflow-file "$WORKFLOW_FILE" \
    --output-json "$TMP_DIR/dependency-local-heavy-deep-scan-policy-report.strategy-drift.json"
} 2>&1)"
strategy_drift_code=$?
set -e
if [ "$strategy_drift_code" -eq 0 ]; then
  echo "expected strategy drift fixture to fail deep-scan policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$strategy_drift_output" | grep -q 'dependency_local_heavy_deep_scan_policy_docs_marker_parity_mismatch'; then
  echo "expected deterministic strategy parity drift reason marker" >&2
  exit 1
fi

selector_drift_ci_tools="$TMP_DIR/test_ci_tools.selector_drift.sh"
cp "$CI_TOOLS_FILE" "$selector_drift_ci_tools"
python3 - "$selector_drift_ci_tools" <<'PY'
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
needle = '  bash "$ROOT_DIR/scripts/runtime/test_check_dependency_local_heavy_deep_scan_policy.sh"\n'
updated = text.replace(needle, "", 1)
if text == updated:
    raise SystemExit("failed to drift required ci-tools entry fixture")
path.write_text(updated, encoding="utf-8")
PY

set +e
selector_drift_output="$({
  bash "$CHECKER" \
    --report-file "$lane_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --strategy-doc "$STRATEGY_DOC" \
    --ops-doc "$OPS_DOC" \
    --ci-tools-file "$selector_drift_ci_tools" \
    --workflow-file "$WORKFLOW_FILE" \
    --output-json "$TMP_DIR/dependency-local-heavy-deep-scan-policy-report.selector-drift.json"
} 2>&1)"
selector_drift_code=$?
set -e
if [ "$selector_drift_code" -eq 0 ]; then
  echo "expected selector drift fixture to fail deep-scan policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$selector_drift_output" | grep -q 'dependency_local_heavy_deep_scan_policy_ci_dry_run_selector_drift'; then
  echo "expected deterministic selector drift reason marker" >&2
  exit 1
fi

workflow_drift_file="$TMP_DIR/ci-fast-gate.workflow-drift.yml"
cp "$WORKFLOW_FILE" "$workflow_drift_file"
cat >> "$workflow_drift_file" <<'EOF_APPEND'
      - name: Forbidden deep-scan run-mode fixture
        run: bash scripts/runtime/run_dependency_local_heavy_deep_scan_lane.sh --profile baseline --mode run
EOF_APPEND

set +e
workflow_drift_output="$({
  bash "$CHECKER" \
    --report-file "$lane_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --strategy-doc "$STRATEGY_DOC" \
    --ops-doc "$OPS_DOC" \
    --ci-tools-file "$CI_TOOLS_FILE" \
    --workflow-file "$workflow_drift_file" \
    --output-json "$TMP_DIR/dependency-local-heavy-deep-scan-policy-report.workflow-drift.json"
} 2>&1)"
workflow_drift_code=$?
set -e
if [ "$workflow_drift_code" -eq 0 ]; then
  echo "expected workflow drift fixture to fail deep-scan policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$workflow_drift_output" | grep -q 'dependency_local_heavy_deep_scan_policy_ci_dry_run_workflow_drift'; then
  echo "expected deterministic workflow drift reason marker" >&2
  exit 1
fi

echo "dependency local-heavy deep-scan policy checker tests passed."
