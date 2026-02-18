#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"

CHECK_SCRIPT="$ROOT_DIR/scripts/ci/check_stale_script_references.sh"
DELETION_MANIFEST="$ROOT_DIR/fixtures/ci/superseded_script_deletion_manifest.json"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

test_harness_require_executable "$CHECK_SCRIPT" "expected stale-script reference checker wrapper to be executable"
test_harness_require_file "$DELETION_MANIFEST" "expected superseded-script deletion manifest fixture to exist"

PASS_REPORT="$TMP_DIR/stale-script-reference-pass-report.json"
bash "$CHECK_SCRIPT" \
  --repo-root "$ROOT_DIR" \
  --deletion-manifest-file "$DELETION_MANIFEST" \
  --output-json "$PASS_REPORT" >"$TMP_DIR/pass.out"

grep -q '^status=ok$' "$TMP_DIR/pass.out"
grep -q '^final_decision=GO$' "$TMP_DIR/pass.out"
grep -q '^reason_codes=none$' "$TMP_DIR/pass.out"
grep -q '^reason_taxonomy_version=kamn.ci.stale-script-reference-detector-reason-taxonomy.v1$' "$TMP_DIR/pass.out"

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
  --repo-root "$ROOT_DIR" \
  --deletion-manifest-file "$INVALID_SCHEMA_MANIFEST" \
  --output-json "$TMP_DIR/invalid-schema-report.json" >"$TMP_DIR/invalid-schema.out" 2>&1; then
  echo "expected stale-script reference checker to fail for invalid deletion-manifest schema" >&2
  exit 1
fi
grep -q 'stale_script_reference_deletion_manifest_schema_invalid' "$TMP_DIR/invalid-schema.out"

INVALID_ENTRY_MANIFEST="$TMP_DIR/invalid-entry-deletion-manifest.json"
cp "$DELETION_MANIFEST" "$INVALID_ENTRY_MANIFEST"
python3 - "$INVALID_ENTRY_MANIFEST" <<'PY'
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["deletions"] = [{"reason_code": "superseded_by_manifest_lane_runner"}]
path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

if bash "$CHECK_SCRIPT" \
  --repo-root "$ROOT_DIR" \
  --deletion-manifest-file "$INVALID_ENTRY_MANIFEST" \
  --output-json "$TMP_DIR/invalid-entry-report.json" >"$TMP_DIR/invalid-entry.out" 2>&1; then
  echo "expected stale-script reference checker to fail for invalid deletion-manifest entries" >&2
  exit 1
fi
grep -q 'stale_script_reference_manifest_entry_invalid' "$TMP_DIR/invalid-entry.out"

MUTATED_ROOT="$TMP_DIR/mutated-root"
mkdir -p "$MUTATED_ROOT/docs/foundation" "$MUTATED_ROOT/.github/workflows" "$MUTATED_ROOT/scripts/framework/manifests"
cat >"$MUTATED_ROOT/docs/foundation/release-gonogo-checklist.md" <<'EOF'
Run legacy entrypoint:
bash scripts/runtime/deprecated_entrypoint.sh
EOF
cat >"$MUTATED_ROOT/.github/workflows/ci-fast-gate.yml" <<'EOF'
name: ci-fast-gate
jobs:
  stale:
    steps:
      - run: bash scripts/runtime/deprecated_entrypoint.sh
EOF
cat >"$MUTATED_ROOT/scripts/framework/manifests/runtime_legacy_lane.json" <<'EOF'
{
  "schema_version": "kamn.contract-lane.manifest.v1",
  "lane_id": "runtime.legacy.contract",
  "phase": "contract",
  "phases": {
    "contract": [
      "bash",
      "scripts/runtime/deprecated_entrypoint.sh"
    ]
  },
  "wrapper_name": "run_legacy_contract_lane.sh"
}
EOF
cat >"$MUTATED_ROOT/superseded_script_deletion_manifest.json" <<'EOF'
{
  "schema_version": "kamn.ci.superseded-script-deletion-manifest.v1",
  "deletion_wave_id": "test-wave",
  "deletions": [
    {
      "script_path": "scripts/runtime/deprecated_entrypoint.sh",
      "reason_code": "superseded_by_manifest_lane_runner"
    }
  ]
}
EOF

if bash "$CHECK_SCRIPT" \
  --repo-root "$MUTATED_ROOT" \
  --deletion-manifest-file "$MUTATED_ROOT/superseded_script_deletion_manifest.json" \
  --output-json "$TMP_DIR/stale-reference-report.json" >"$TMP_DIR/stale-reference.out" 2>&1; then
  echo "expected stale-script reference checker to fail when deleted entrypoints are still referenced" >&2
  exit 1
fi
grep -q '^reason_codes=.*stale_script_reference_detected' "$TMP_DIR/stale-reference.out"
grep -q '^stale_reference_count=3$' "$TMP_DIR/stale-reference.out"

echo "stale-script reference checker tests passed."
