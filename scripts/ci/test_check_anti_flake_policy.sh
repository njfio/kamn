#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/ci/check_anti_flake_policy.sh"

if [ ! -x "$SCRIPT" ]; then
  echo "expected anti-flake policy checker script to be executable" >&2
  exit 1
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

future="$(date -u -d '+14 days' +%Y-%m-%d)"

cat > "$TMP_DIR/empty-registry.txt" <<'EOF'
# owner|test-id|issue|expiry|notes
EOF

empty_report="$TMP_DIR/empty-report.json"
empty_output="$(
  bash "$SCRIPT" \
    --registry-file "$TMP_DIR/empty-registry.txt" \
    --expected-final-decision GO \
    --max-active-entries 0 \
    --output-json "$empty_report"
)"
for marker in \
  '^anti_flake_policy_status=pass$' \
  '^anti_flake_policy_final_decision=GO$' \
  '^anti_flake_policy_reason_taxonomy_version=kamn.ci.anti-flake-policy-reason-taxonomy.v1$' \
  '^anti_flake_policy_reason_codes=no_active_flaky_entries$' \
  '^anti_flake_policy_reason_codes_csv=no_active_flaky_entries$' \
  '^anti_flake_policy_reason_codes_value=no_active_flaky_entries$' \
  '^anti_flake_policy_reason_class=stable$' \
  '^ci_smoke_local_heavy_boundary_status=verified$'; do
  if ! printf '%s\n' "$empty_output" | grep -q "$marker"; then
    echo "expected anti-flake empty-registry marker: $marker" >&2
    exit 1
  fi
done

python3 - "$empty_report" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if report.get("schema_version") != "kamn.ci.anti-flake-policy-report.v1":
    raise SystemExit("unexpected anti-flake policy schema")
if report.get("reason_taxonomy_version") != "kamn.ci.anti-flake-policy-reason-taxonomy.v1":
    raise SystemExit("unexpected anti-flake policy reason taxonomy version")
if report.get("status") != "pass":
    raise SystemExit("expected anti-flake policy pass status for empty registry")
if report.get("final_decision") != "GO":
    raise SystemExit("expected anti-flake policy GO decision for empty registry")
if report.get("reason_codes_csv") != "no_active_flaky_entries":
    raise SystemExit("expected anti-flake policy reason_codes_csv marker for empty registry")
if report.get("reason_codes_value") != "no_active_flaky_entries":
    raise SystemExit("expected anti-flake policy reason_codes_value marker for empty registry")
if report.get("reason_class") != "stable":
    raise SystemExit("expected anti-flake policy reason_class=stable for empty registry")
if report.get("ci_smoke_local_heavy_boundary_status") != "verified":
    raise SystemExit("expected anti-flake policy boundary status verified for empty registry")
if report.get("active_entries") != 0:
    raise SystemExit("expected anti-flake active_entries=0")
PY

cat > "$TMP_DIR/active-registry.txt" <<EOF
# owner|test-id|issue|expiry|notes
qa|crate::tests::flaky_a|#70|$future|temporary quarantine
EOF

active_report="$TMP_DIR/active-report.json"
set +e
active_output="$(
  bash "$SCRIPT" \
    --registry-file "$TMP_DIR/active-registry.txt" \
    --expected-final-decision NO-GO \
    --max-active-entries 0 \
    --output-json "$active_report" 2>&1
)"
active_status=$?
set -e
if [ "$active_status" -eq 0 ]; then
  echo "expected anti-flake policy to fail when active entries exceed max" >&2
  exit 1
fi
if ! printf '%s\n' "$active_output" | grep -q '^anti_flake_policy_reason_codes=active_flaky_entries_exceed_max$'; then
  echo "expected anti-flake exceed-max reason marker" >&2
  exit 1
fi

allowed_report="$TMP_DIR/allowed-report.json"
allowed_output="$(
  bash "$SCRIPT" \
    --registry-file "$TMP_DIR/active-registry.txt" \
    --expected-final-decision GO \
    --max-active-entries 1 \
    --output-json "$allowed_report"
)"
if ! printf '%s\n' "$allowed_output" | grep -q '^anti_flake_policy_status=pass$'; then
  echo "expected anti-flake policy to pass when entries are within max threshold" >&2
  exit 1
fi
if ! printf '%s\n' "$allowed_output" | grep -q '^anti_flake_policy_reason_codes=active_flaky_entries_within_budget$'; then
  echo "expected anti-flake within-budget reason marker" >&2
  exit 1
fi
if ! printf '%s\n' "$allowed_output" | grep -q '^anti_flake_policy_reason_class=budgeted$'; then
  echo "expected anti-flake within-budget reason class marker" >&2
  exit 1
fi

cat > "$TMP_DIR/invalid-registry.txt" <<EOF
qa|crate::tests::flaky_a|issue-70|$future|temporary quarantine
EOF

invalid_report="$TMP_DIR/invalid-report.json"
set +e
invalid_output="$(
  bash "$SCRIPT" \
    --registry-file "$TMP_DIR/invalid-registry.txt" \
    --expected-final-decision NO-GO \
    --max-active-entries 0 \
    --output-json "$invalid_report" 2>&1
)"
invalid_status=$?
set -e
if [ "$invalid_status" -eq 0 ]; then
  echo "expected anti-flake policy to fail when registry validation fails" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_output" | grep -q '^anti_flake_policy_reason_codes=registry_validation_failed$'; then
  echo "expected anti-flake registry-validation reason marker" >&2
  exit 1
fi

set +e
mismatch_output="$(
  bash "$SCRIPT" \
    --registry-file "$TMP_DIR/empty-registry.txt" \
    --expected-final-decision NO-GO \
    --max-active-entries 0 \
    --output-json "$TMP_DIR/mismatch-report.json" 2>&1
)"
mismatch_status=$?
set -e
if [ "$mismatch_status" -eq 0 ]; then
  echo "expected anti-flake policy expected-final-decision mismatch failure" >&2
  exit 1
fi
if ! printf '%s\n' "$mismatch_output" | grep -q '^anti_flake_policy_reason_codes=expected_final_decision_mismatch$'; then
  echo "expected anti-flake expected-final-decision mismatch marker" >&2
  exit 1
fi

cat > "$TMP_DIR/fast-workflow-rerun-drift.yml" <<'EOF'
name: CI fast gate
jobs:
  fast:
    steps:
      - run: bash scripts/ci/run_with_retry.sh --max-attempts 1 -- cargo test
EOF

cat > "$TMP_DIR/deep-workflow-rerun-ok.yml" <<'EOF'
name: CI deep validate
jobs:
  deep:
    steps:
      - run: bash scripts/ci/run_with_retry.sh --max-attempts 2 -- cargo test
      - run: bash scripts/ci/run_with_retry.sh --max-attempts 1 -- cargo test
EOF

set +e
rerun_drift_output="$(
  bash "$SCRIPT" \
    --registry-file "$TMP_DIR/empty-registry.txt" \
    --expected-final-decision NO-GO \
    --max-active-entries 0 \
    --fast-workflow-file "$TMP_DIR/fast-workflow-rerun-drift.yml" \
    --deep-workflow-file "$TMP_DIR/deep-workflow-rerun-ok.yml" \
    --output-json "$TMP_DIR/rerun-drift-report.json" 2>&1
)"
rerun_drift_status=$?
set -e
if [ "$rerun_drift_status" -eq 0 ]; then
  echo "expected anti-flake policy to fail when rerun-policy bounded retry marker drifts" >&2
  exit 1
fi
if ! printf '%s\n' "$rerun_drift_output" | grep -q '^anti_flake_policy_reason_codes=rerun_policy_bounded_retry_missing$'; then
  echo "expected anti-flake rerun-policy bounded-retry reason marker" >&2
  exit 1
fi
if ! printf '%s\n' "$rerun_drift_output" | grep -q '^anti_flake_policy_reason_class=violation$'; then
  echo "expected anti-flake rerun-policy violation reason class marker" >&2
  exit 1
fi

cat > "$TMP_DIR/boundary-drift-fast-workflow.yml" <<'EOF'
name: CI fast gate
jobs:
  fast:
    steps:
      - name: Run Kolme local-heavy contract lane
        if: steps.scope.outputs.run_kolme_local_heavy_contract_tests == 'true' && steps.scope.outputs.kolme_local_heavy_selector_opt_in == 'true'
        run: echo "local heavy"
      - name: Generate performance smoke report
        run: echo "smoke"
EOF

set +e
boundary_drift_output="$(
  bash "$SCRIPT" \
    --registry-file "$TMP_DIR/empty-registry.txt" \
    --expected-final-decision NO-GO \
    --max-active-entries 0 \
    --fast-workflow-file "$TMP_DIR/boundary-drift-fast-workflow.yml" \
    --deep-workflow-file "$TMP_DIR/deep-workflow-rerun-ok.yml" \
    --output-json "$TMP_DIR/boundary-drift-report.json" 2>&1
)"
boundary_drift_status=$?
set -e
if [ "$boundary_drift_status" -eq 0 ]; then
  echo "expected anti-flake policy to fail when CI smoke/local-heavy boundary markers drift" >&2
  exit 1
fi
if ! printf '%s\n' "$boundary_drift_output" | grep -q '^anti_flake_policy_reason_codes=ci_smoke_threshold_check_step_missing$'; then
  echo "expected anti-flake boundary drift reason marker" >&2
  exit 1
fi
if ! printf '%s\n' "$boundary_drift_output" | grep -q '^ci_smoke_local_heavy_boundary_status=violation$'; then
  echo "expected anti-flake boundary violation marker" >&2
  exit 1
fi
if ! printf '%s\n' "$boundary_drift_output" | grep -q '^anti_flake_policy_reason_class=violation$'; then
  echo "expected anti-flake boundary drift reason class marker" >&2
  exit 1
fi

echo "check_anti_flake_policy tests passed."
