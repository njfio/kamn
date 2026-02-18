#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/ci/check_kamn_core_missing_docs_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

CORE_LIB_FIXTURE="$TMP_DIR/lib.rs"
ALLOWLIST_FIXTURE="$TMP_DIR/allowlist.txt"
GRADUATED_MODULES_FIXTURE="$TMP_DIR/graduated-modules.txt"
README_FIXTURE="$TMP_DIR/README.md"
PLAN_DOC_FIXTURE="$TMP_DIR/engineering-hardening-wave.md"
ARCH_DOC_FIXTURE="$TMP_DIR/kamn-core-module-map.md"
RUSTDOC_GUIDE_FIXTURE="$TMP_DIR/rustdoc-publishing.md"
VELOCITY_BASELINE_FIXTURE="$TMP_DIR/missing-docs-velocity-baseline.json"
VELOCITY_THRESHOLD_FIXTURE="$TMP_DIR/missing-docs-velocity-thresholds.json"
VELOCITY_CADENCE_DOC_FIXTURE="$TMP_DIR/missing-docs-velocity-cadence.md"
GRADUATION_BATCH_REPORT_FIXTURE="$TMP_DIR/missing-docs-first-batch-graduation-report.md"

run_checker() {
  KAMN_CORE_LIB_PATH="$CORE_LIB_FIXTURE" \
  KAMN_CORE_MISSING_DOCS_ALLOWLIST_PATH="$ALLOWLIST_FIXTURE" \
  KAMN_CORE_MISSING_DOCS_GRADUATED_MODULES_PATH="$GRADUATED_MODULES_FIXTURE" \
  KAMN_README_PATH="$README_FIXTURE" \
  KAMN_ENGINEERING_HARDENING_DOC_PATH="$PLAN_DOC_FIXTURE" \
  KAMN_CORE_MODULE_MAP_DOC_PATH="$ARCH_DOC_FIXTURE" \
  KAMN_RUSTDOC_PUBLISHING_DOC_PATH="$RUSTDOC_GUIDE_FIXTURE" \
  KAMN_MISSING_DOCS_VELOCITY_BASELINE_PATH="$VELOCITY_BASELINE_FIXTURE" \
  KAMN_MISSING_DOCS_VELOCITY_THRESHOLD_PATH="$VELOCITY_THRESHOLD_FIXTURE" \
  KAMN_MISSING_DOCS_VELOCITY_CADENCE_DOC_PATH="$VELOCITY_CADENCE_DOC_FIXTURE" \
  KAMN_MISSING_DOCS_GRADUATION_BATCH_REPORT_PATH="$GRADUATION_BATCH_REPORT_FIXTURE" \
    bash "$SCRIPT"
}

reset_fixtures() {
  cp "$ROOT_DIR/crates/kamn-core/src/lib.rs" "$CORE_LIB_FIXTURE"
  cp "$ROOT_DIR/fixtures/ci/kamn_core_missing_docs_allowlist.txt" "$ALLOWLIST_FIXTURE"
  cp "$ROOT_DIR/fixtures/ci/kamn_core_missing_docs_graduated_modules.txt" "$GRADUATED_MODULES_FIXTURE"
  cp "$ROOT_DIR/README.md" "$README_FIXTURE"
  cp "$ROOT_DIR/docs/planning/engineering-hardening-wave.md" "$PLAN_DOC_FIXTURE"
  cp "$ROOT_DIR/docs/architecture/kamn-core-module-map.md" "$ARCH_DOC_FIXTURE"
  cp "$ROOT_DIR/docs/developer/rustdoc-publishing.md" "$RUSTDOC_GUIDE_FIXTURE"
  cp "$ROOT_DIR/fixtures/ci/kamn_core_missing_docs_velocity_baseline.json" "$VELOCITY_BASELINE_FIXTURE"
  cp "$ROOT_DIR/.ci/kamn-core-missing-docs-velocity-thresholds.json" "$VELOCITY_THRESHOLD_FIXTURE"
  cp "$ROOT_DIR/docs/planning/issues/missing-docs-velocity-cadence.md" "$VELOCITY_CADENCE_DOC_FIXTURE"
  cp "$ROOT_DIR/docs/planning/issues/missing-docs-first-batch-graduation-report.md" "$GRADUATION_BATCH_REPORT_FIXTURE"
}

expect_failure() {
  local label="$1"
  if run_checker >"$TMP_DIR/checker.out" 2>"$TMP_DIR/checker.err"; then
    echo "$label: expected failure but checker succeeded." >&2
    cat "$TMP_DIR/checker.out" >&2 || true
    cat "$TMP_DIR/checker.err" >&2 || true
    exit 1
  fi
}

assert_exemption_regression_markers() {
  if ! grep -q '^reason_taxonomy_version=kamn.ci.kamn-core-missing-docs-policy-reason-taxonomy.v1$' "$TMP_DIR/checker.err"; then
    echo "graduated module allowlist bypass should emit deterministic reason taxonomy marker" >&2
    exit 1
  fi
  if ! grep -q '^reason_codes_csv=graduated_module_exemption_regression,rustdoc_navigation_parity_drift$' "$TMP_DIR/checker.err"; then
    echo "graduated module allowlist bypass should emit deterministic reason code set marker" >&2
    exit 1
  fi
  if ! grep -q '^reason_code=graduated_module_exemption_regression$' "$TMP_DIR/checker.err"; then
    echo "graduated module allowlist bypass should emit deterministic exemption-regression reason code marker" >&2
    exit 1
  fi
}

reset_fixtures
run_checker >"$TMP_DIR/pass.out"
if ! grep -Eq '^missing_docs_allowlisted_module_count=[0-9]+$' "$TMP_DIR/pass.out"; then
  echo "pass path should emit missing_docs_allowlisted_module_count marker" >&2
  exit 1
fi
if ! grep -Eq '^missing_docs_graduated_module_count=[0-9]+$' "$TMP_DIR/pass.out"; then
  echo "pass path should emit missing_docs_graduated_module_count marker" >&2
  exit 1
fi
if ! grep -Eq '^missing_docs_allowlisted_module_delta=-?[0-9]+$' "$TMP_DIR/pass.out"; then
  echo "pass path should emit missing_docs_allowlisted_module_delta marker" >&2
  exit 1
fi
if ! grep -Eq '^missing_docs_graduated_module_delta=-?[0-9]+$' "$TMP_DIR/pass.out"; then
  echo "pass path should emit missing_docs_graduated_module_delta marker" >&2
  exit 1
fi
if ! grep -q '^missing_docs_velocity_reason_taxonomy_version=kamn.ci.kamn-core-missing-docs-velocity-reason-taxonomy.v1$' "$TMP_DIR/pass.out"; then
  echo "pass path should emit missing_docs_velocity_reason_taxonomy_version marker" >&2
  exit 1
fi
if ! grep -q '^missing_docs_velocity_reason_codes_csv=allowlist_fully_graduated,baseline_window_not_elapsed,ci_local_docs_velocity_window_boundary_exceeded,multiple_policy_violations,stagnation_window_exceeded,velocity_target_met,velocity_threshold_config_invalid,velocity_window_under_threshold,window_not_elapsed$' "$TMP_DIR/pass.out"; then
  echo "pass path should emit missing_docs_velocity_reason_codes_csv marker" >&2
  exit 1
fi
if ! grep -Eq '^missing_docs_velocity_reason_codes_value=[a-z0-9_]+$' "$TMP_DIR/pass.out"; then
  echo "pass path should emit missing_docs_velocity_reason_codes_value marker" >&2
  exit 1
fi

# Regression: #896
reset_fixtures
sed -i '/#!\[warn(missing_docs)\]/d' "$CORE_LIB_FIXTURE"
expect_failure "missing warn policy should fail"

reset_fixtures
printf '\nsynthetic_module\n' >>"$ALLOWLIST_FIXTURE"
expect_failure "allowlist drift should fail"

reset_fixtures
sed -i '/check_kamn_core_missing_docs_policy.sh/d' "$README_FIXTURE"
expect_failure "README drift should fail"

reset_fixtures
sed -i '/#!\[warn(missing_docs)\]/d' "$PLAN_DOC_FIXTURE"
expect_failure "plan doc marker drift should fail"

reset_fixtures
sed -i '/missing_docs_throughput_report_contract.py/d' "$PLAN_DOC_FIXTURE"
expect_failure "plan doc throughput command drift should fail"

reset_fixtures
sed -i '/missing_docs_velocity_guard.py/d' "$PLAN_DOC_FIXTURE"
expect_failure "plan doc velocity guard command drift should fail"

reset_fixtures
sed -i '/## Runtime Flow (Condensed)/d' "$ARCH_DOC_FIXTURE"
expect_failure "architecture map runtime flow marker drift should fail"

reset_fixtures
sed -i '/cargo doc -p kamn-core --no-deps/d' "$RUSTDOC_GUIDE_FIXTURE"
expect_failure "rustdoc publishing command drift should fail"

reset_fixtures
sed -i '/docs\/developer\/rustdoc-publishing.md/d' "$README_FIXTURE"
if run_checker >"$TMP_DIR/rustdoc-link-drift.out" 2>"$TMP_DIR/rustdoc-link-drift.err"; then
  echo "README rustdoc link drift should fail: expected failure but checker succeeded." >&2
  exit 1
fi
if ! grep -q '^reason_taxonomy_version=kamn.ci.kamn-core-missing-docs-policy-reason-taxonomy.v1$' "$TMP_DIR/rustdoc-link-drift.err"; then
  echo "README rustdoc link drift should emit deterministic reason taxonomy marker" >&2
  exit 1
fi
if ! grep -q '^reason_codes_csv=graduated_module_exemption_regression,rustdoc_navigation_parity_drift$' "$TMP_DIR/rustdoc-link-drift.err"; then
  echo "README rustdoc link drift should emit deterministic reason code set marker" >&2
  exit 1
fi
if ! grep -q '^reason_code=rustdoc_navigation_parity_drift$' "$TMP_DIR/rustdoc-link-drift.err"; then
  echo "README rustdoc link drift should emit deterministic reason code marker" >&2
  exit 1
fi

reset_fixtures
sed -i '/Regression: #2127/d' "$VELOCITY_CADENCE_DOC_FIXTURE"
expect_failure "velocity cadence regression marker drift should fail"

reset_fixtures
sed -i '/batch_id: first-three-modules-v1/d' "$GRADUATION_BATCH_REPORT_FIXTURE"
expect_failure "graduation batch report marker drift should fail"

for first_batch_module in bootstrap key_recovery kolme_runtime_commit; do
  reset_fixtures
  sed -i "/^${first_batch_module}\$/d" "$GRADUATED_MODULES_FIXTURE"
  expect_failure "first-batch graduated-module fixture drift (${first_batch_module}) should fail"
  if ! grep -q "first graduation batch module '${first_batch_module}'" "$TMP_DIR/checker.err"; then
    echo "first-batch graduated-module fixture drift should mention missing module '${first_batch_module}'" >&2
    exit 1
  fi
done

reset_fixtures
printf '\nagent_key_hierarchy\n' >>"$ALLOWLIST_FIXTURE"
sed -i 's/^pub mod agent_key_hierarchy;/#[allow(missing_docs)]\npub mod agent_key_hierarchy;/' "$CORE_LIB_FIXTURE"
sed -i '/^agent_key_hierarchy$/d' "$GRADUATED_MODULES_FIXTURE"
python3 - "$VELOCITY_BASELINE_FIXTURE" <<'PY'
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["commit_count"] = 1
payload["graduated_module_count"] = 61
payload["allowlisted_module_count"] = 1
path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
expect_failure "velocity guard stagnation policy drift should fail"
if ! grep -q '^missing_docs_velocity_reason_taxonomy_version=kamn.ci.kamn-core-missing-docs-velocity-reason-taxonomy.v1$' "$TMP_DIR/checker.err"; then
  echo "stagnation failure should emit missing_docs velocity taxonomy marker" >&2
  exit 1
fi
if ! grep -Eq '^missing_docs_velocity_reason_codes_value=(stagnation_window_exceeded|multiple_policy_violations|velocity_window_under_threshold)$' "$TMP_DIR/checker.err"; then
  echo "stagnation failure should emit missing_docs velocity reason code marker for a deterministic failure reason" >&2
  exit 1
fi
if ! grep -Eq '^missing_docs_allowlisted_module_delta=-?[0-9]+$' "$TMP_DIR/checker.err"; then
  echo "stagnation failure should emit missing_docs_allowlisted_module_delta marker" >&2
  exit 1
fi

# Regression: #1723
for first_batch_module in bootstrap key_recovery kolme_runtime_commit; do
  reset_fixtures
  printf '\n%s\n' "$first_batch_module" >>"$ALLOWLIST_FIXTURE"
  sed -i "s/^pub mod ${first_batch_module};/#[allow(missing_docs)]\\npub mod ${first_batch_module};/" "$CORE_LIB_FIXTURE"
  expect_failure "graduated module allowlist bypass (${first_batch_module}) should fail"
  assert_exemption_regression_markers
done

echo "kamn-core missing-docs policy checker tests passed."
