#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_unified_api_observability_local_heavy_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_unified_api_observability_local_heavy_live_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT
EXPECTED_REASON_TAXONOMY_VERSION="kamn.runtime.unified-api-observability-local-heavy-policy-reason-taxonomy.v1"
EXPECTED_REASON_CODES_CSV="ci_fast_gate_failed,unified_api_observability_local_heavy_policy_artifact_paths_invalid,unified_api_observability_local_heavy_policy_evidence_artifact_missing,unified_api_observability_local_heavy_policy_evidence_convergence_mismatch,unified_api_observability_local_heavy_policy_evidence_links_incomplete,unified_api_observability_local_heavy_policy_ci_fast_gate_mismatch,unified_api_observability_local_heavy_policy_command_budget_exceeded,unified_api_observability_local_heavy_policy_command_count_invalid,unified_api_observability_local_heavy_policy_command_max_seconds_invalid,unified_api_observability_local_heavy_policy_compatibility_matrix_status_mismatch,unified_api_observability_local_heavy_policy_compatibility_policy_schema_mismatch,unified_api_observability_local_heavy_policy_compatibility_policy_status_mismatch,unified_api_observability_local_heavy_policy_compatibility_report_schema_mismatch,unified_api_observability_local_heavy_policy_dry_run_command_count_mismatch,unified_api_observability_local_heavy_policy_dry_run_command_status_mismatch,unified_api_observability_local_heavy_policy_dry_run_eligibility_mismatch,unified_api_observability_local_heavy_policy_dry_run_reason_code_mismatch,unified_api_observability_local_heavy_policy_dry_run_soak_iterations_executed_mismatch,unified_api_observability_local_heavy_policy_dry_run_soak_status_mismatch,unified_api_observability_local_heavy_policy_elapsed_seconds_invalid,unified_api_observability_local_heavy_policy_fast_gate_exclusion_reason_mismatch,unified_api_observability_local_heavy_policy_fast_gate_exclusion_status_mismatch,unified_api_observability_local_heavy_policy_final_decision_invalid,unified_api_observability_local_heavy_policy_final_decision_mismatch,unified_api_observability_local_heavy_policy_lane_mode_invalid,unified_api_observability_local_heavy_policy_max_seconds_invalid,unified_api_observability_local_heavy_policy_observability_policy_schema_mismatch,unified_api_observability_local_heavy_policy_observability_policy_status_mismatch,unified_api_observability_local_heavy_policy_observability_report_schema_mismatch,unified_api_observability_local_heavy_policy_observability_soak_status_mismatch,unified_api_observability_local_heavy_policy_run_mode_command_count_mismatch,unified_api_observability_local_heavy_policy_run_mode_command_status_mismatch,unified_api_observability_local_heavy_policy_run_mode_exclusion_mismatch,unified_api_observability_local_heavy_policy_run_mode_reason_code_mismatch,unified_api_observability_local_heavy_policy_run_mode_soak_iterations_executed_invalid,unified_api_observability_local_heavy_policy_run_mode_soak_iterations_mismatch,unified_api_observability_local_heavy_policy_run_mode_soak_iterations_requested_invalid,unified_api_observability_local_heavy_policy_run_mode_soak_status_mismatch,unified_api_observability_local_heavy_policy_runtime_budget_exceeded,unified_api_observability_local_heavy_policy_runtime_budget_status_mismatch,unified_api_observability_local_heavy_policy_schema_mismatch,unified_api_observability_local_heavy_policy_soak_iterations_executed_invalid,unified_api_observability_local_heavy_policy_soak_iterations_requested_invalid,unified_api_observability_local_heavy_policy_status_mismatch"

if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected unified API-observability local-heavy validation script to be executable" >&2
  exit 1
fi
if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected unified API-observability local-heavy policy checker script to be executable" >&2
  exit 1
fi

report_file="$TMP_DIR/unified-api-observability-local-heavy-summary.json"
bash "$VALIDATION_SCRIPT" \
  --mode dry-run \
  --ci-fast-gate PASS \
  --output-json "$report_file" >/dev/null

policy_report="$TMP_DIR/unified-api-observability-local-heavy-policy.json"
policy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$report_file" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$policy_report"
)"
for marker in \
  '^status=ok$' \
  '^final_decision=GO$' \
  '^unified_api_observability_local_heavy_policy_status=verified$' \
  "^reason_taxonomy_version=$EXPECTED_REASON_TAXONOMY_VERSION$" \
  "^reason_codes_csv=$EXPECTED_REASON_CODES_CSV$" \
  '^reason_codes=none$' \
  '^reason_codes_value=none$'; do
  if ! printf '%s\n' "$policy_output" | grep -q "$marker"; then
    echo "expected unified API-observability local-heavy policy marker: $marker" >&2
    exit 1
  fi
done

python3 - "$policy_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.unified-api-observability-local-heavy-live-policy-report.v1":
    raise SystemExit("unexpected unified API-observability local-heavy policy schema")
if payload.get("status") != "pass":
    raise SystemExit("expected policy status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected policy final_decision=GO")
if payload.get("unified_api_observability_local_heavy_policy_status") != "verified":
    raise SystemExit("expected unified_api_observability_local_heavy_policy_status=verified")
if payload.get("reason_taxonomy_version") != "kamn.runtime.unified-api-observability-local-heavy-policy-reason-taxonomy.v1":
    raise SystemExit("expected deterministic reason taxonomy marker in unified API-observability local-heavy policy report")
if payload.get("reason_codes_csv") != "ci_fast_gate_failed,unified_api_observability_local_heavy_policy_artifact_paths_invalid,unified_api_observability_local_heavy_policy_evidence_artifact_missing,unified_api_observability_local_heavy_policy_evidence_convergence_mismatch,unified_api_observability_local_heavy_policy_evidence_links_incomplete,unified_api_observability_local_heavy_policy_ci_fast_gate_mismatch,unified_api_observability_local_heavy_policy_command_budget_exceeded,unified_api_observability_local_heavy_policy_command_count_invalid,unified_api_observability_local_heavy_policy_command_max_seconds_invalid,unified_api_observability_local_heavy_policy_compatibility_matrix_status_mismatch,unified_api_observability_local_heavy_policy_compatibility_policy_schema_mismatch,unified_api_observability_local_heavy_policy_compatibility_policy_status_mismatch,unified_api_observability_local_heavy_policy_compatibility_report_schema_mismatch,unified_api_observability_local_heavy_policy_dry_run_command_count_mismatch,unified_api_observability_local_heavy_policy_dry_run_command_status_mismatch,unified_api_observability_local_heavy_policy_dry_run_eligibility_mismatch,unified_api_observability_local_heavy_policy_dry_run_reason_code_mismatch,unified_api_observability_local_heavy_policy_dry_run_soak_iterations_executed_mismatch,unified_api_observability_local_heavy_policy_dry_run_soak_status_mismatch,unified_api_observability_local_heavy_policy_elapsed_seconds_invalid,unified_api_observability_local_heavy_policy_fast_gate_exclusion_reason_mismatch,unified_api_observability_local_heavy_policy_fast_gate_exclusion_status_mismatch,unified_api_observability_local_heavy_policy_final_decision_invalid,unified_api_observability_local_heavy_policy_final_decision_mismatch,unified_api_observability_local_heavy_policy_lane_mode_invalid,unified_api_observability_local_heavy_policy_max_seconds_invalid,unified_api_observability_local_heavy_policy_observability_policy_schema_mismatch,unified_api_observability_local_heavy_policy_observability_policy_status_mismatch,unified_api_observability_local_heavy_policy_observability_report_schema_mismatch,unified_api_observability_local_heavy_policy_observability_soak_status_mismatch,unified_api_observability_local_heavy_policy_run_mode_command_count_mismatch,unified_api_observability_local_heavy_policy_run_mode_command_status_mismatch,unified_api_observability_local_heavy_policy_run_mode_exclusion_mismatch,unified_api_observability_local_heavy_policy_run_mode_reason_code_mismatch,unified_api_observability_local_heavy_policy_run_mode_soak_iterations_executed_invalid,unified_api_observability_local_heavy_policy_run_mode_soak_iterations_mismatch,unified_api_observability_local_heavy_policy_run_mode_soak_iterations_requested_invalid,unified_api_observability_local_heavy_policy_run_mode_soak_status_mismatch,unified_api_observability_local_heavy_policy_runtime_budget_exceeded,unified_api_observability_local_heavy_policy_runtime_budget_status_mismatch,unified_api_observability_local_heavy_policy_schema_mismatch,unified_api_observability_local_heavy_policy_soak_iterations_executed_invalid,unified_api_observability_local_heavy_policy_soak_iterations_requested_invalid,unified_api_observability_local_heavy_policy_status_mismatch":
    raise SystemExit("expected deterministic reason taxonomy csv marker in unified API-observability local-heavy policy report")
if payload.get("reason_codes_value") != "none":
    raise SystemExit("expected reason_codes_value=none in unified API-observability local-heavy policy report")
PY

policy_report_repeat="$TMP_DIR/unified-api-observability-local-heavy-policy.repeat.json"
bash "$POLICY_CHECKER" \
  --report-file "$report_file" \
  --expected-final-decision GO \
  --ci-fast-gate PASS \
  --output-json "$policy_report_repeat" >/dev/null

python3 - "$policy_report" "$policy_report_repeat" <<'PY'
import json
import pathlib
import sys

first = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
second = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
first.pop("generated_at_epoch", None)
second.pop("generated_at_epoch", None)
if first != second:
    raise SystemExit("expected deterministic policy evidence payload across repeated runs")
PY

tampered_report="$TMP_DIR/unified-api-observability-local-heavy-summary.tampered.json"
cp "$report_file" "$tampered_report"
python3 - "$tampered_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["compatibility_matrix_status"] = "missing"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/unified-api-observability-local-heavy-policy.tampered.json" 2>&1
)"
tampered_code=$?
set -e
if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered unified API-observability local-heavy report to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q 'unified_api_observability_local_heavy_policy_compatibility_matrix_status_mismatch'; then
  echo "expected deterministic compatibility matrix drift reason code for unified API-observability local-heavy policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q "^reason_taxonomy_version=$EXPECTED_REASON_TAXONOMY_VERSION$"; then
  echo "expected deterministic reason taxonomy marker for tampered unified API-observability local-heavy policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q '^reason_codes_value=unified_api_observability_local_heavy_policy_compatibility_matrix_status_mismatch$'; then
  echo "expected deterministic normalized reason_codes_value marker for tampered unified API-observability local-heavy policy validation" >&2
  exit 1
fi

set +e
fast_gate_fail_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$report_file" \
    --expected-final-decision GO \
    --ci-fast-gate FAIL 2>&1
)"
fast_gate_fail_code=$?
set -e
if [ "$fast_gate_fail_code" -eq 0 ]; then
  echo "expected unified API-observability local-heavy policy checker to fail closed when ci-fast-gate=FAIL" >&2
  exit 1
fi
if ! printf '%s\n' "$fast_gate_fail_output" | grep -q 'ci_fast_gate_failed'; then
  echo "expected deterministic ci_fast_gate_failed marker for unified API-observability local-heavy policy checker" >&2
  exit 1
fi

incomplete_evidence_report="$TMP_DIR/unified-api-observability-local-heavy-summary.incomplete-evidence.json"
cp "$report_file" "$incomplete_evidence_report"
python3 - "$incomplete_evidence_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["lane_mode"] = "run"
payload["ci_fast_gate_eligibility"] = "excluded_local_heavy"
payload["run_mode_command_status"] = "executed"
payload["run_mode_command_count"] = 4
payload["execution_reason_code"] = "unified_api_observability_local_heavy_executed"
payload["local_heavy_soak_lane_status"] = "verified"
payload["soak_iterations_requested"] = 1
payload["soak_iterations_executed"] = 1
payload["artifact_paths"] = {
    "compatibility_report": "/tmp/unified-observability-compatibility-report.json"
}
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
incomplete_evidence_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$incomplete_evidence_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/unified-api-observability-local-heavy-policy.incomplete-evidence.json" 2>&1
)"
incomplete_evidence_code=$?
set -e
if [ "$incomplete_evidence_code" -eq 0 ]; then
  echo "expected telemetry policy checker to fail closed for incomplete run-mode evidence links" >&2
  exit 1
fi
if ! printf '%s\n' "$incomplete_evidence_output" | grep -q 'unified_api_observability_local_heavy_policy_evidence_links_incomplete'; then
  echo "expected deterministic incomplete evidence-links reason code for telemetry policy checker" >&2
  exit 1
fi

missing_evidence_report="$TMP_DIR/unified-api-observability-local-heavy-summary.missing-evidence-artifacts.json"
cp "$report_file" "$missing_evidence_report"
python3 - "$missing_evidence_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["lane_mode"] = "run"
payload["ci_fast_gate_eligibility"] = "excluded_local_heavy"
payload["run_mode_command_status"] = "executed"
payload["run_mode_command_count"] = 4
payload["execution_reason_code"] = "unified_api_observability_local_heavy_executed"
payload["local_heavy_soak_lane_status"] = "verified"
payload["soak_iterations_requested"] = 1
payload["soak_iterations_executed"] = 1
payload["artifact_paths"] = {
    "compatibility_report": "/tmp/unified-observability-compatibility-report.json",
    "compatibility_policy_report": "/tmp/unified-observability-compatibility-policy-report.json",
    "observability_report": "/tmp/unified-observability-observability-report.json",
    "observability_policy_report": "/tmp/unified-observability-observability-policy-report.json",
}
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
missing_evidence_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$missing_evidence_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/unified-api-observability-local-heavy-policy.missing-evidence-artifacts.json" 2>&1
)"
missing_evidence_code=$?
set -e
if [ "$missing_evidence_code" -eq 0 ]; then
  echo "expected telemetry policy checker to fail closed for non-existent evidence artifact links" >&2
  exit 1
fi
if ! printf '%s\n' "$missing_evidence_output" | grep -q 'unified_api_observability_local_heavy_policy_evidence_artifact_missing'; then
  echo "expected deterministic missing evidence artifact reason code for telemetry policy checker" >&2
  exit 1
fi

echo "unified API-observability local-heavy policy checker tests passed."
