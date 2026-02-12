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

reset_fixtures
run_checker >/dev/null

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
expect_failure "README rustdoc link drift should fail"

reset_fixtures
sed -i '/Regression: #2127/d' "$VELOCITY_CADENCE_DOC_FIXTURE"
expect_failure "velocity cadence regression marker drift should fail"

reset_fixtures
python3 - "$VELOCITY_BASELINE_FIXTURE" <<'PY'
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["commit_count"] = 1
path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
expect_failure "velocity guard stagnation policy drift should fail"

# Regression: #1723
reset_fixtures
printf '\nkolme_runtime_commit\n' >>"$ALLOWLIST_FIXTURE"
sed -i 's/^pub mod kolme_runtime_commit;/#[allow(missing_docs)]\npub mod kolme_runtime_commit;/' "$CORE_LIB_FIXTURE"
expect_failure "graduated module allowlist bypass should fail"

echo "kamn-core missing-docs policy checker tests passed."
