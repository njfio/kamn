#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/runtime/run_local_heavy_redaction_validation_lane.sh"
CHECKER="$ROOT_DIR/scripts/runtime/check_local_heavy_redaction_validation_policy.sh"
STRATEGY_DOC="$ROOT_DIR/docs/ci/strategy.md"
OPS_DOC="$ROOT_DIR/docs/ops/configuration.md"

for required_exec in "$RUNNER" "$CHECKER"; do
  if [ ! -x "$required_exec" ]; then
    echo "expected redaction validation policy script to be executable: $required_exec" >&2
    exit 1
  fi
done
for required_file in "$STRATEGY_DOC" "$OPS_DOC"; do
  if [ ! -f "$required_file" ]; then
    echo "expected redaction validation policy source to exist: $required_file" >&2
    exit 1
  fi
done

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT
lane_report="$TMP_DIR/local-heavy-redaction-validation-baseline.json"
policy_report="$TMP_DIR/local-heavy-redaction-validation-policy-report.json"

lane_output="$({
  bash "$RUNNER" \
    --profile baseline \
    --mode dry-run \
    --ci-fast-gate PASS \
    --max-seconds 120 \
    --output-json "$lane_report"
} 2>&1)"
if ! printf '%s\n' "$lane_output" | grep -q '^status=pass$'; then
  echo "expected redaction validation baseline status=pass marker" >&2
  exit 1
fi

policy_output="$({
  bash "$CHECKER" \
    --report-file "$lane_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --strategy-doc "$STRATEGY_DOC" \
    --ops-doc "$OPS_DOC" \
    --output-json "$policy_report"
} 2>&1)"
for marker in \
  "status=pass" \
  "final_decision=GO" \
  "redaction_policy_status=verified" \
  "redaction_policy_docs_marker_parity_status=verified" \
  "promotion_decision_reason_mapping_status=verified" \
  "reason_codes_value=none"; do
  if ! printf '%s\n' "$policy_output" | grep -q "^${marker}$"; then
    echo "expected redaction policy marker ${marker}" >&2
    exit 1
  fi
done

python3 - "$policy_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.local-heavy-redaction-validation-policy-report.v1":
    raise SystemExit("unexpected redaction policy report schema")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected redaction policy final_decision=GO")
if payload.get("reason_taxonomy_version") != "kamn.runtime.local-heavy-redaction-validation-policy-reason-taxonomy.v1":
    raise SystemExit("expected redaction policy reason taxonomy marker")
PY

tampered_report="$TMP_DIR/local-heavy-redaction-validation-baseline.tampered.json"
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
    --output-json "$TMP_DIR/local-heavy-redaction-validation-policy-report.tampered.json"
} 2>&1)"
tampered_code=$?
set -e
if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered redaction report to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q 'redaction_policy_profile_contract_mismatch'; then
  echo "expected deterministic fail-closed reason for tampered redaction marker" >&2
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
    "local_heavy_redaction_validation_required_profiles_csv=baseline,injected-leak",
    "local_heavy_redaction_validation_required_profiles_csv=baseline,drifted",
    1,
)
if text == updated:
    raise SystemExit("failed to drift redaction ops marker fixture")
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
    --output-json "$TMP_DIR/local-heavy-redaction-validation-policy-report.ops-drift.json"
} 2>&1)"
ops_drift_code=$?
set -e
if [ "$ops_drift_code" -eq 0 ]; then
  echo "expected ops drift fixture to fail redaction policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$ops_drift_output" | grep -q 'redaction_policy_docs_marker_parity_mismatch'; then
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
    "local_heavy_redaction_validation_policy_reason_taxonomy_version=kamn.runtime.local-heavy-redaction-validation-policy-reason-taxonomy.v1",
    "local_heavy_redaction_validation_policy_reason_taxonomy_version=drifted",
    1,
)
if text == updated:
    raise SystemExit("failed to drift redaction strategy marker fixture")
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
    --output-json "$TMP_DIR/local-heavy-redaction-validation-policy-report.strategy-drift.json"
} 2>&1)"
strategy_drift_code=$?
set -e
if [ "$strategy_drift_code" -eq 0 ]; then
  echo "expected strategy drift fixture to fail redaction policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$strategy_drift_output" | grep -q 'redaction_policy_docs_marker_parity_mismatch'; then
  echo "expected deterministic strategy parity drift reason marker" >&2
  exit 1
fi

echo "redaction local-heavy policy checker tests passed."
