#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"

GENERATE_SCRIPT="$ROOT_DIR/scripts/ci/generate_superseded_script_inventory.sh"
CHECK_SCRIPT="$ROOT_DIR/scripts/ci/check_superseded_script_deletion_manifest.sh"
MIGRATION_GROUPS_FILE="$ROOT_DIR/fixtures/ci/kolme_manifest_migration_contract_groups.json"
OWNERSHIP_FILE="$ROOT_DIR/fixtures/ci/superseded_script_lane_ownership.json"
BASELINE_INVENTORY="$ROOT_DIR/fixtures/ci/superseded_script_inventory_baseline.json"
DELETION_MANIFEST="$ROOT_DIR/fixtures/ci/superseded_script_deletion_manifest.json"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

test_harness_require_executable "$GENERATE_SCRIPT" "expected superseded-script inventory generator wrapper to be executable"
test_harness_require_executable "$CHECK_SCRIPT" "expected superseded-script deletion-manifest checker wrapper to be executable"
test_harness_require_file "$MIGRATION_GROUPS_FILE" "expected migration groups fixture to exist"
test_harness_require_file "$OWNERSHIP_FILE" "expected superseded-script ownership mapping fixture to exist"
test_harness_require_file "$BASELINE_INVENTORY" "expected superseded-script inventory baseline fixture to exist"
test_harness_require_file "$DELETION_MANIFEST" "expected superseded-script deletion manifest fixture to exist"

GENERATED_INVENTORY="$TMP_DIR/generated-superseded-script-inventory.json"
bash "$GENERATE_SCRIPT" \
  --migration-groups-file "$MIGRATION_GROUPS_FILE" \
  --lane-ownership-file "$OWNERSHIP_FILE" \
  --output-json "$GENERATED_INVENTORY" >"$TMP_DIR/generate.out"

grep -q '^status=generated$' "$TMP_DIR/generate.out"
grep -q '^reason_codes=none$' "$TMP_DIR/generate.out"
grep -q '^inventory_entry_count=' "$TMP_DIR/generate.out"

python3 - "$BASELINE_INVENTORY" "$GENERATED_INVENTORY" <<'PY'
import json
import sys
from pathlib import Path

baseline = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
generated = json.loads(Path(sys.argv[2]).read_text(encoding="utf-8"))
if baseline != generated:
    raise SystemExit("superseded-script generated baseline drift detected")
PY

PASS_REPORT="$TMP_DIR/superseded-script-pass-report.json"
bash "$CHECK_SCRIPT" \
  --inventory-file "$BASELINE_INVENTORY" \
  --deletion-manifest-file "$DELETION_MANIFEST" \
  --output-json "$PASS_REPORT" >"$TMP_DIR/check-pass.out"

grep -q '^status=ok$' "$TMP_DIR/check-pass.out"
grep -q '^final_decision=GO$' "$TMP_DIR/check-pass.out"
grep -q '^reason_codes=none$' "$TMP_DIR/check-pass.out"
grep -q '^reason_taxonomy_version=kamn.ci.superseded-script-deletion-manifest-reason-taxonomy.v1$' "$TMP_DIR/check-pass.out"

python3 - "$DELETION_MANIFEST" <<'PY'
import json
import sys
from pathlib import Path

manifest = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
deletions = manifest.get("deletions")
if not isinstance(deletions, list):
    raise SystemExit("deletion manifest must include deletions list")

actual_paths = {
    entry.get("script_path")
    for entry in deletions
    if isinstance(entry, dict) and isinstance(entry.get("script_path"), str)
}
expected_non_kolme_paths = {
    "scripts/canary/run_launch_canary_contract_lane.sh",
    "scripts/canary/run_post_cutover_slo_contract_lane.sh",
    "scripts/ci/run_fast_gate_budget_delta_contract_lane.sh",
    "scripts/ci/run_ignored_test_and_script_budget_trend_contract_lane.sh",
    "scripts/ci/run_kamn_core_rustdoc_artifact_contract_lane.sh",
    "scripts/ci/run_kolme_test_harness_loc_soft_budget_contract_lane.sh",
    "scripts/ci/run_test_harness_loc_soft_budget_contract_lane.sh",
    "scripts/deploy/run_deployment_slo_rollback_contract_lane.sh",
    "scripts/deploy/run_dr_evidence_contract_lane.sh",
    "scripts/deploy/run_gonogo_evidence_contract_lane.sh",
    "scripts/deploy/run_staging_rehearsal_contract_lane.sh",
    "scripts/governance/run_governance_lifecycle_rollback_contract_lane.sh",
    "scripts/governance/run_governance_simulation_contract_lane.sh",
    "scripts/governance/run_quorum_attestation_replay_contract_lane.sh",
    "scripts/governance/run_stake_slash_risk_contract_lane.sh",
}
missing = sorted(expected_non_kolme_paths - actual_paths)
if missing:
    raise SystemExit(
        "deletion manifest missing canary/ci/deploy/governance wave entries: "
        + ", ".join(missing)
    )
PY

INVALID_SCHEMA_MANIFEST="$TMP_DIR/invalid-schema-deletion-manifest.json"
cp "$DELETION_MANIFEST" "$INVALID_SCHEMA_MANIFEST"
python3 - "$INVALID_SCHEMA_MANIFEST" <<'PY'
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["schema_version"] = "kamn.ci.superseded-script-deletion-manifest.v0"
path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

if bash "$CHECK_SCRIPT" \
  --inventory-file "$BASELINE_INVENTORY" \
  --deletion-manifest-file "$INVALID_SCHEMA_MANIFEST" \
  --output-json "$TMP_DIR/invalid-schema-report.json" >"$TMP_DIR/invalid-schema.out" 2>&1; then
  echo "expected superseded-script checker to fail for invalid deletion-manifest schema" >&2
  exit 1
fi
grep -q 'superseded_deletion_manifest_schema_invalid' "$TMP_DIR/invalid-schema.out"

UNKNOWN_SCRIPT_MANIFEST="$TMP_DIR/unknown-script-deletion-manifest.json"
cp "$DELETION_MANIFEST" "$UNKNOWN_SCRIPT_MANIFEST"
python3 - "$UNKNOWN_SCRIPT_MANIFEST" <<'PY'
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["deletions"] = [
    {
        "script_path": "scripts/ci/not-a-real-superseded-script.sh",
        "reason_code": "superseded_by_manifest_lane_runner",
    }
]
path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

if bash "$CHECK_SCRIPT" \
  --inventory-file "$BASELINE_INVENTORY" \
  --deletion-manifest-file "$UNKNOWN_SCRIPT_MANIFEST" \
  --output-json "$TMP_DIR/unknown-script-report.json" >"$TMP_DIR/unknown-script.out" 2>&1; then
  echo "expected superseded-script checker to fail for unknown script manifest entry" >&2
  exit 1
fi
grep -q 'superseded_deletion_manifest_references_unknown_script' "$TMP_DIR/unknown-script.out"

MISSING_EVIDENCE_INVENTORY="$TMP_DIR/missing-evidence-inventory.json"
cp "$BASELINE_INVENTORY" "$MISSING_EVIDENCE_INVENTORY"
python3 - "$MISSING_EVIDENCE_INVENTORY" <<'PY'
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
if payload.get("superseded_scripts"):
    payload["superseded_scripts"][0]["replacement_evidence"] = {}
path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

if bash "$CHECK_SCRIPT" \
  --inventory-file "$MISSING_EVIDENCE_INVENTORY" \
  --deletion-manifest-file "$DELETION_MANIFEST" \
  --output-json "$TMP_DIR/missing-evidence-report.json" >"$TMP_DIR/missing-evidence.out" 2>&1; then
  echo "expected superseded-script checker to fail for missing replacement evidence" >&2
  exit 1
fi
grep -q 'superseded_inventory_replacement_evidence_missing' "$TMP_DIR/missing-evidence.out"

echo "superseded-script deletion manifest checker tests passed."
