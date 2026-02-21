#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATE_SCRIPT="$ROOT_DIR/scripts/ci/generate_performance_smoke_report.sh"
CHECK_SCRIPT="$ROOT_DIR/scripts/ci/check_performance_thresholds.sh"
FIXTURE_MATRIX="$ROOT_DIR/fixtures/ci/performance_hot_path_fixture_matrix.json"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

RUNTIME_REPORT="$TMP_DIR/runtime-smoke.json"

bash "$GENERATE_SCRIPT" \
  --lane smoke \
  --workload runtime \
  --fixture-file "$FIXTURE_MATRIX" \
  --output-json "$RUNTIME_REPORT" >"$TMP_DIR/generate.out"

# Unit: baseline checker pass path emits deterministic taxonomy markers.
bash "$CHECK_SCRIPT" \
  --lane smoke \
  --report-json "$RUNTIME_REPORT" >"$TMP_DIR/check-pass.out"
grep -q '^status=pass$' "$TMP_DIR/check-pass.out"
grep -q '^final_decision=GO$' "$TMP_DIR/check-pass.out"
grep -q '^performance_ci_smoke_reason_taxonomy_version=kamn.ci.performance-ci-smoke-threshold-reason-taxonomy.v1$' "$TMP_DIR/check-pass.out"
grep -q '^performance_ci_smoke_reason_codes_value=none$' "$TMP_DIR/check-pass.out"
grep -q '^performance_ci_smoke_selector_status=verified$' "$TMP_DIR/check-pass.out"
grep -q '^performance_ci_smoke_workflow_status=verified$' "$TMP_DIR/check-pass.out"

# Functional: threshold breach fails closed with deterministic reason code.
BREACHED_REPORT="$TMP_DIR/runtime-smoke-breached.json"
cp "$RUNTIME_REPORT" "$BREACHED_REPORT"
python3 - "$BREACHED_REPORT" <<'PY'
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["latency_p50_ms"] = 999
path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

if bash "$CHECK_SCRIPT" \
  --lane smoke \
  --report-json "$BREACHED_REPORT" >"$TMP_DIR/check-breached.out" 2>&1; then
  echo "expected checker to fail for breached thresholds" >&2
  exit 1
fi
grep -q '^status=fail$' "$TMP_DIR/check-breached.out"
grep -q '^performance_ci_smoke_reason_codes_value=.*performance_ci_smoke_latency_p50_threshold_exceeded' "$TMP_DIR/check-breached.out"

# Regression: report marker drift fails closed with report contract reason.
MISSING_PROVENANCE_REPORT="$TMP_DIR/runtime-smoke-missing-provenance.json"
cp "$RUNTIME_REPORT" "$MISSING_PROVENANCE_REPORT"
python3 - "$MISSING_PROVENANCE_REPORT" <<'PY'
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload.pop("baseline_provenance_artifact_version", None)
payload.pop("drift_threshold_seed_id", None)
path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

if bash "$CHECK_SCRIPT" \
  --lane smoke \
  --report-json "$MISSING_PROVENANCE_REPORT" >"$TMP_DIR/check-missing-provenance.out" 2>&1; then
  echo "expected checker to fail when baseline provenance/seed markers are missing" >&2
  exit 1
fi
grep -q '^performance_ci_smoke_reason_codes_value=.*performance_ci_smoke_report_contract_violation' "$TMP_DIR/check-missing-provenance.out"

# Integration: selector/workflow drift are detected deterministically.
LEAKED_CI_TOOLS="$TMP_DIR/test_ci_tools_selector_drift.sh"
cp "$ROOT_DIR/scripts/ci/test_ci_tools.sh" "$LEAKED_CI_TOOLS"
python3 - "$LEAKED_CI_TOOLS" <<'PY'
import sys
from pathlib import Path

path = Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
text = text.replace(
    'cargo test -p kamn-core --test performance_ci_smoke_governance_contract -- --nocapture\n',
    '',
)
path.write_text(text, encoding="utf-8")
PY

if bash "$CHECK_SCRIPT" \
  --lane smoke \
  --report-json "$RUNTIME_REPORT" \
  --ci-tools-file "$LEAKED_CI_TOOLS" >"$TMP_DIR/check-selector-drift.out" 2>&1; then
  echo "expected checker to fail when selector required entry is missing" >&2
  exit 1
fi
grep -q '^performance_ci_smoke_reason_codes_value=.*performance_ci_smoke_selector_missing_checker_entry' "$TMP_DIR/check-selector-drift.out"

LEAKED_WORKFLOW="$TMP_DIR/ci-fast-gate-selector-drift.yml"
cp "$ROOT_DIR/.github/workflows/ci-fast-gate.yml" "$LEAKED_WORKFLOW"
cat >> "$LEAKED_WORKFLOW" <<'YAML'
      - name: leaked performance deep checker
        run: bash scripts/ci/check_performance_thresholds.sh --lane deep --report-json performance-smoke-report.json --profile-file .ci/performance-targets.env
YAML

if bash "$CHECK_SCRIPT" \
  --lane smoke \
  --report-json "$RUNTIME_REPORT" \
  --workflow-file "$LEAKED_WORKFLOW" >"$TMP_DIR/check-workflow-drift.out" 2>&1; then
  echo "expected checker to fail when workflow deep entry is leaked" >&2
  exit 1
fi
grep -q '^performance_ci_smoke_reason_codes_value=.*performance_ci_smoke_workflow_forbidden_entry_present' "$TMP_DIR/check-workflow-drift.out"

echo "performance threshold checker tests passed."
