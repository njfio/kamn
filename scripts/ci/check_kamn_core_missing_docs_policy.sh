#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CORE_LIB_PATH="${KAMN_CORE_LIB_PATH:-$ROOT_DIR/crates/kamn-core/src/lib.rs}"
ALLOWLIST_PATH="${KAMN_CORE_MISSING_DOCS_ALLOWLIST_PATH:-$ROOT_DIR/fixtures/ci/kamn_core_missing_docs_allowlist.txt}"
GRADUATED_MODULES_PATH="${KAMN_CORE_MISSING_DOCS_GRADUATED_MODULES_PATH:-$ROOT_DIR/fixtures/ci/kamn_core_missing_docs_graduated_modules.txt}"
README_PATH="${KAMN_README_PATH:-$ROOT_DIR/README.md}"
PLAN_DOC_PATH="${KAMN_ENGINEERING_HARDENING_DOC_PATH:-$ROOT_DIR/docs/planning/engineering-hardening-wave.md}"
ARCH_DOC_PATH="${KAMN_CORE_MODULE_MAP_DOC_PATH:-$ROOT_DIR/docs/architecture/kamn-core-module-map.md}"
RUSTDOC_GUIDE_PATH="${KAMN_RUSTDOC_PUBLISHING_DOC_PATH:-$ROOT_DIR/docs/developer/rustdoc-publishing.md}"
THROUGHPUT_CONTRACT_SCRIPT_PATH="${KAMN_MISSING_DOCS_THROUGHPUT_CONTRACT_SCRIPT_PATH:-$ROOT_DIR/scripts/ci/missing_docs_throughput_report_contract.py}"
VELOCITY_GUARD_SCRIPT_PATH="${KAMN_MISSING_DOCS_VELOCITY_GUARD_SCRIPT_PATH:-$ROOT_DIR/scripts/ci/missing_docs_velocity_guard.py}"
VELOCITY_BASELINE_PATH="${KAMN_MISSING_DOCS_VELOCITY_BASELINE_PATH:-$ROOT_DIR/fixtures/ci/kamn_core_missing_docs_velocity_baseline.json}"
VELOCITY_THRESHOLD_PATH="${KAMN_MISSING_DOCS_VELOCITY_THRESHOLD_PATH:-$ROOT_DIR/.ci/kamn-core-missing-docs-velocity-thresholds.json}"
VELOCITY_CADENCE_DOC_PATH="${KAMN_MISSING_DOCS_VELOCITY_CADENCE_DOC_PATH:-$ROOT_DIR/docs/planning/issues/missing-docs-velocity-cadence.md}"

require_file() {
  local file="$1"
  local name="$2"
  if [ ! -f "$file" ]; then
    echo "missing-docs policy contract failed: missing ${name} at '${file}'." >&2
    exit 1
  fi
}

require_file "$CORE_LIB_PATH" "kamn-core lib"
require_file "$ALLOWLIST_PATH" "missing-docs allowlist fixture"
require_file "$GRADUATED_MODULES_PATH" "missing-docs graduated-modules fixture"
require_file "$README_PATH" "README"
require_file "$PLAN_DOC_PATH" "engineering hardening plan"
require_file "$ARCH_DOC_PATH" "kamn-core module map"
require_file "$RUSTDOC_GUIDE_PATH" "rustdoc publishing guide"
require_file "$THROUGHPUT_CONTRACT_SCRIPT_PATH" "missing-docs throughput report contract script"
require_file "$VELOCITY_GUARD_SCRIPT_PATH" "missing-docs velocity guard script"
require_file "$VELOCITY_BASELINE_PATH" "missing-docs velocity baseline fixture"
require_file "$VELOCITY_THRESHOLD_PATH" "missing-docs velocity threshold config"
require_file "$VELOCITY_CADENCE_DOC_PATH" "missing-docs velocity cadence doc"

if ! grep -Fq "#![warn(missing_docs)]" "$CORE_LIB_PATH"; then
  echo "missing-docs policy contract failed: kamn-core must declare #![warn(missing_docs)]." >&2
  exit 1
fi

if grep -Eq '^#!\[allow\(missing_docs\)\]' "$CORE_LIB_PATH"; then
  echo "missing-docs policy contract failed: crate-wide #![allow(missing_docs)] is not permitted." >&2
  exit 1
fi

actual_allowlisted_modules="$(
  awk '
    /^[[:space:]]*#\[allow\(missing_docs\)\]/ {
      allow = 1
      next
    }
    /^[[:space:]]*pub mod / {
      module_name = $3
      sub(/;/, "", module_name)
      if (allow == 1) {
        print module_name
      }
      allow = 0
      next
    }
    /^[[:space:]]*$/ { next }
    { allow = 0 }
  ' "$CORE_LIB_PATH" | sort
)"

expected_allowlisted_modules="$(
  awk '
    /^[[:space:]]*#/ { next }
    /^[[:space:]]*$/ { next }
    { print $1 }
  ' "$ALLOWLIST_PATH" | sort
)"

expected_graduated_modules="$(
  awk '
    /^[[:space:]]*#/ { next }
    /^[[:space:]]*$/ { next }
    { print $1 }
  ' "$GRADUATED_MODULES_PATH" | sort
)"

if ! diff -u \
  <(printf '%s\n' "$expected_allowlisted_modules") \
  <(printf '%s\n' "$actual_allowlisted_modules") >/dev/null; then
  echo "missing-docs policy contract failed: allowlisted module set drifted from fixture." >&2
  diff -u \
    <(printf '%s\n' "$expected_allowlisted_modules") \
    <(printf '%s\n' "$actual_allowlisted_modules") >&2
  exit 1
fi

graduated_allowlist_overlap="$(
  comm -12 \
    <(printf '%s\n' "$expected_graduated_modules") \
    <(printf '%s\n' "$actual_allowlisted_modules")
)"
if [ -n "$graduated_allowlist_overlap" ]; then
  echo "missing-docs policy contract failed: graduated modules cannot be re-added to #[allow(missing_docs)] allowlist." >&2
  echo "$graduated_allowlist_overlap" >&2
  exit 1
fi

if ! grep -Fq "check_kamn_core_missing_docs_policy.sh" "$README_PATH"; then
  echo "missing-docs policy contract failed: README must document check_kamn_core_missing_docs_policy.sh." >&2
  exit 1
fi

if ! grep -Fq "docs/planning/engineering-hardening-wave.md" "$README_PATH"; then
  echo "missing-docs policy contract failed: README must link engineering-hardening-wave.md." >&2
  exit 1
fi

if ! grep -Fq "docs/architecture/kamn-core-module-map.md" "$README_PATH"; then
  echo "missing-docs policy contract failed: README must link kamn-core module map." >&2
  exit 1
fi

if ! grep -Fq "docs/developer/rustdoc-publishing.md" "$README_PATH"; then
  echo "missing-docs policy contract failed: README must link rustdoc publishing guide." >&2
  exit 1
fi

if ! grep -Fq "check_kamn_core_missing_docs_policy.sh" "$PLAN_DOC_PATH"; then
  echo "missing-docs policy contract failed: engineering hardening plan must include policy checker command." >&2
  exit 1
fi

if ! grep -Fq "missing_docs_throughput_report_contract.py" "$PLAN_DOC_PATH"; then
  echo "missing-docs policy contract failed: engineering hardening plan must include throughput report contract command." >&2
  exit 1
fi

if ! grep -Fq "target_modules_per_100_commits" "$PLAN_DOC_PATH"; then
  echo "missing-docs policy contract failed: engineering hardening plan must include throughput target marker." >&2
  exit 1
fi

if ! grep -Fq "kamn.ci.kamn-core-missing-docs-throughput-report.v1" "$PLAN_DOC_PATH"; then
  echo "missing-docs policy contract failed: engineering hardening plan must include throughput schema marker." >&2
  exit 1
fi

if ! grep -Fq "missing_docs_velocity_guard.py" "$PLAN_DOC_PATH"; then
  echo "missing-docs policy contract failed: engineering hardening plan must include velocity guard command marker." >&2
  exit 1
fi

if ! grep -Fq "kamn.ci.kamn-core-missing-docs-velocity-policy.v1" "$PLAN_DOC_PATH"; then
  echo "missing-docs policy contract failed: engineering hardening plan must include velocity policy schema marker." >&2
  exit 1
fi

if ! grep -Fq "docs/planning/issues/missing-docs-velocity-cadence.md" "$PLAN_DOC_PATH"; then
  echo "missing-docs policy contract failed: engineering hardening plan must link missing-docs velocity cadence doc." >&2
  exit 1
fi

if ! grep -Fq "#![warn(missing_docs)]" "$PLAN_DOC_PATH"; then
  echo "missing-docs policy contract failed: engineering hardening plan must include #![warn(missing_docs)] policy marker." >&2
  exit 1
fi

if ! grep -Fq "docs/architecture/kamn-core-module-map.md" "$PLAN_DOC_PATH"; then
  echo "missing-docs policy contract failed: engineering hardening plan must include module map doc path." >&2
  exit 1
fi

if ! grep -Fq "docs/developer/rustdoc-publishing.md" "$PLAN_DOC_PATH"; then
  echo "missing-docs policy contract failed: engineering hardening plan must include rustdoc publishing doc path." >&2
  exit 1
fi

if ! grep -Fq "KAMN Core Module Map" "$ARCH_DOC_PATH"; then
  echo "missing-docs policy contract failed: module map must declare title marker." >&2
  exit 1
fi

if ! grep -Fq "## Runtime Flow (Condensed)" "$ARCH_DOC_PATH"; then
  echo "missing-docs policy contract failed: module map must document runtime flow." >&2
  exit 1
fi

if ! grep -Fq "crates/kamn-core/src/lib.rs" "$ARCH_DOC_PATH"; then
  echo "missing-docs policy contract failed: module map must document contributor entrypoint." >&2
  exit 1
fi

if ! grep -Fq "cargo doc -p kamn-core --no-deps" "$RUSTDOC_GUIDE_PATH"; then
  echo "missing-docs policy contract failed: rustdoc guide must include bounded cargo doc command." >&2
  exit 1
fi

if ! grep -Fq "RUSTDOCFLAGS=\"-D warnings\" cargo doc -p kamn-core --no-deps" "$RUSTDOC_GUIDE_PATH"; then
  echo "missing-docs policy contract failed: rustdoc guide must include warning-fail command." >&2
  exit 1
fi

if ! grep -Fq "target/doc" "$RUSTDOC_GUIDE_PATH"; then
  echo "missing-docs policy contract failed: rustdoc guide must include publication artifact path." >&2
  exit 1
fi

if ! grep -Fq "missing_docs_velocity_guard.py" "$VELOCITY_CADENCE_DOC_PATH"; then
  echo "missing-docs policy contract failed: velocity cadence doc must include velocity guard command." >&2
  exit 1
fi

if ! grep -Fq ".ci/kamn-core-missing-docs-velocity-thresholds.json" "$VELOCITY_CADENCE_DOC_PATH"; then
  echo "missing-docs policy contract failed: velocity cadence doc must include threshold config path." >&2
  exit 1
fi

if ! grep -Fq "fixtures/ci/kamn_core_missing_docs_velocity_baseline.json" "$VELOCITY_CADENCE_DOC_PATH"; then
  echo "missing-docs policy contract failed: velocity cadence doc must include baseline fixture path." >&2
  exit 1
fi

if ! grep -Fq "Regression: #2127" "$VELOCITY_CADENCE_DOC_PATH"; then
  echo "missing-docs policy contract failed: velocity cadence doc must include regression marker." >&2
  exit 1
fi

tmp_throughput_report="$(mktemp)"
if ! python3 "$THROUGHPUT_CONTRACT_SCRIPT_PATH" generate \
  --core-lib "$CORE_LIB_PATH" \
  --allowlist "$ALLOWLIST_PATH" \
  --graduated-modules "$GRADUATED_MODULES_PATH" \
  --output-json "$tmp_throughput_report" >/dev/null; then
  echo "missing-docs policy contract failed: throughput report generation command failed." >&2
  rm -f "$tmp_throughput_report"
  exit 1
fi

if ! python3 "$THROUGHPUT_CONTRACT_SCRIPT_PATH" check \
  --report-file "$tmp_throughput_report" >/dev/null; then
  echo "missing-docs policy contract failed: throughput report policy check failed." >&2
  rm -f "$tmp_throughput_report"
  exit 1
fi

tmp_velocity_policy_report="$(mktemp)"
if ! python3 "$VELOCITY_GUARD_SCRIPT_PATH" check \
  --report-file "$tmp_throughput_report" \
  --baseline-file "$VELOCITY_BASELINE_PATH" \
  --threshold-file "$VELOCITY_THRESHOLD_PATH" \
  --output-json "$tmp_velocity_policy_report" >/dev/null; then
  echo "missing-docs policy contract failed: velocity guard policy check failed." >&2
  rm -f "$tmp_throughput_report" "$tmp_velocity_policy_report"
  exit 1
fi
rm -f "$tmp_throughput_report" "$tmp_velocity_policy_report"

echo "kamn-core missing-docs policy contract passed."
