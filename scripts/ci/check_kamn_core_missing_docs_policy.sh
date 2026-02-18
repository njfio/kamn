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
GRADUATION_BATCH_REPORT_PATH="${KAMN_MISSING_DOCS_GRADUATION_BATCH_REPORT_PATH:-$ROOT_DIR/docs/planning/issues/missing-docs-first-batch-graduation-report.md}"
POLICY_REASON_TAXONOMY_VERSION="kamn.ci.kamn-core-missing-docs-policy-reason-taxonomy.v1"
POLICY_REASON_CODES_CSV="graduated_module_exemption_regression,rustdoc_navigation_parity_drift"

require_file() {
  local file="$1"
  local name="$2"
  if [ ! -f "$file" ]; then
    echo "missing-docs policy contract failed: missing ${name} at '${file}'." >&2
    exit 1
  fi
}

fail_with_reason() {
  local reason_code="$1"
  local message="$2"
  echo "reason_taxonomy_version=$POLICY_REASON_TAXONOMY_VERSION" >&2
  echo "reason_codes_csv=$POLICY_REASON_CODES_CSV" >&2
  echo "reason_code=$reason_code" >&2
  echo "$message" >&2
  exit 1
}

emit_missing_docs_evidence_markers() {
  local throughput_report="$1"
  local baseline_file="$2"
  local velocity_policy_report="$3"
  local velocity_markers_file="$4"
  python3 - "$throughput_report" "$baseline_file" "$velocity_policy_report" "$velocity_markers_file" <<'PY'
import json
import sys
from pathlib import Path

throughput_path = Path(sys.argv[1])
baseline_path = Path(sys.argv[2])
velocity_policy_path = Path(sys.argv[3])
velocity_markers_path = Path(sys.argv[4])

throughput_payload = json.loads(throughput_path.read_text(encoding="utf-8"))
baseline_payload = json.loads(baseline_path.read_text(encoding="utf-8"))

current_allowlisted = int(throughput_payload.get("allowlisted_module_count", 0))
current_graduated = int(throughput_payload.get("graduated_module_count", 0))
baseline_allowlisted = int(baseline_payload.get("allowlisted_module_count", 0))
baseline_graduated = int(baseline_payload.get("graduated_module_count", 0))

allowlisted_delta = current_allowlisted - baseline_allowlisted
graduated_delta = current_graduated - baseline_graduated

marker_map: dict[str, str] = {}
for raw_line in velocity_markers_path.read_text(encoding="utf-8").splitlines():
    if "=" not in raw_line:
        continue
    key, value = raw_line.split("=", 1)
    key = key.strip()
    value = value.strip()
    if key and key not in marker_map:
        marker_map[key] = value

policy_payload: dict[str, object] = {}
if velocity_policy_path.is_file():
    loaded_policy = json.loads(velocity_policy_path.read_text(encoding="utf-8"))
    if isinstance(loaded_policy, dict):
        policy_payload = loaded_policy
        if "allowlisted_module_delta" in policy_payload:
            allowlisted_delta = int(policy_payload["allowlisted_module_delta"])  # type: ignore[arg-type]
        if "graduated_module_delta" in policy_payload:
            graduated_delta = int(policy_payload["graduated_module_delta"])  # type: ignore[arg-type]

status = str(policy_payload.get("status") or marker_map.get("status", "unknown"))
final_decision = str(
    policy_payload.get("final_decision") or marker_map.get("final_decision", "unknown")
)
reason_taxonomy_version = str(
    policy_payload.get("reason_taxonomy_version")
    or marker_map.get("reason_taxonomy_version", "unknown")
)
reason_codes_csv = str(
    policy_payload.get("reason_codes_csv") or marker_map.get("reason_codes_csv", "unknown")
)
reason_codes_value = str(
    policy_payload.get("reason_codes_value")
    or policy_payload.get("reason_key")
    or marker_map.get("reason_codes_value")
    or marker_map.get("reason_key", "unknown")
)

print(f"missing_docs_allowlisted_module_count={current_allowlisted}")
print(f"missing_docs_graduated_module_count={current_graduated}")
print(f"missing_docs_allowlisted_module_delta={allowlisted_delta}")
print(f"missing_docs_graduated_module_delta={graduated_delta}")
print(f"missing_docs_velocity_status={status}")
print(f"missing_docs_velocity_final_decision={final_decision}")
print(f"missing_docs_velocity_reason_taxonomy_version={reason_taxonomy_version}")
print(f"missing_docs_velocity_reason_codes_csv={reason_codes_csv}")
print(f"missing_docs_velocity_reason_codes_value={reason_codes_value}")
PY
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
require_file "$GRADUATION_BATCH_REPORT_PATH" "missing-docs graduation batch report doc"

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
  fail_with_reason \
    "graduated_module_exemption_regression" \
    "missing-docs policy contract failed: graduated modules cannot be re-added to #[allow(missing_docs)] allowlist.
$graduated_allowlist_overlap"
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
  fail_with_reason \
    "rustdoc_navigation_parity_drift" \
    "missing-docs policy contract failed: README must link rustdoc publishing guide."
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

if ! grep -Fq "docs/planning/issues/missing-docs-first-batch-graduation-report.md" "$PLAN_DOC_PATH"; then
  echo "missing-docs policy contract failed: engineering hardening plan must link missing-docs graduation batch report doc." >&2
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

if ! grep -Fq "Missing-Docs First Batch Graduation Report" "$GRADUATION_BATCH_REPORT_PATH"; then
  echo "missing-docs policy contract failed: graduation batch report doc must include title marker." >&2
  exit 1
fi

if ! grep -Fq "schema_version: kamn.ci.kamn-core-missing-docs-graduation-batch-report.v1" "$GRADUATION_BATCH_REPORT_PATH"; then
  echo "missing-docs policy contract failed: graduation batch report doc must include schema marker." >&2
  exit 1
fi

if ! grep -Fq "batch_id: first-three-modules-v1" "$GRADUATION_BATCH_REPORT_PATH"; then
  echo "missing-docs policy contract failed: graduation batch report doc must include batch id marker." >&2
  exit 1
fi

for batch_module in bootstrap key_recovery kolme_runtime_commit; do
  if ! grep -Fq "$batch_module" "$GRADUATION_BATCH_REPORT_PATH"; then
    echo "missing-docs policy contract failed: graduation batch report doc must include module marker '$batch_module'." >&2
    exit 1
  fi
done

if ! grep -Fq "Regression: #2126" "$GRADUATION_BATCH_REPORT_PATH"; then
  echo "missing-docs policy contract failed: graduation batch report doc must include regression marker." >&2
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
tmp_velocity_stdout="$(mktemp)"
tmp_velocity_stderr="$(mktemp)"
set +e
python3 "$VELOCITY_GUARD_SCRIPT_PATH" check \
  --report-file "$tmp_throughput_report" \
  --baseline-file "$VELOCITY_BASELINE_PATH" \
  --threshold-file "$VELOCITY_THRESHOLD_PATH" \
  --output-json "$tmp_velocity_policy_report" >"$tmp_velocity_stdout" 2>"$tmp_velocity_stderr"
velocity_exit="$?"
set -e
cat "$tmp_velocity_stdout" "$tmp_velocity_stderr" >"$tmp_velocity_stdout.merged"

if [ "$velocity_exit" -ne 0 ]; then
  emit_missing_docs_evidence_markers \
    "$tmp_throughput_report" \
    "$VELOCITY_BASELINE_PATH" \
    "$tmp_velocity_policy_report" \
    "$tmp_velocity_stdout.merged" >&2
  cat "$tmp_velocity_stderr" >&2
  echo "missing-docs policy contract failed: velocity guard policy check failed." >&2
  rm -f \
    "$tmp_throughput_report" \
    "$tmp_velocity_policy_report" \
    "$tmp_velocity_stdout" \
    "$tmp_velocity_stderr" \
    "$tmp_velocity_stdout.merged"
  exit 1
fi

emit_missing_docs_evidence_markers \
  "$tmp_throughput_report" \
  "$VELOCITY_BASELINE_PATH" \
  "$tmp_velocity_policy_report" \
  "$tmp_velocity_stdout.merged"

rm -f \
  "$tmp_throughput_report" \
  "$tmp_velocity_policy_report" \
  "$tmp_velocity_stdout" \
  "$tmp_velocity_stderr" \
  "$tmp_velocity_stdout.merged"

echo "kamn-core missing-docs policy contract passed."
