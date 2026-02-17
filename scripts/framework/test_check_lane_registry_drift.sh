#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$ROOT_DIR/scripts/framework/check_lane_registry_drift.sh"
REGISTRY="$ROOT_DIR/scripts/framework/lane_registry.json"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$CHECKER" ]; then
  echo "expected lane registry drift checker to be executable: $CHECKER" >&2
  exit 1
fi

if [ ! -f "$REGISTRY" ]; then
  echo "expected lane registry source to exist: $REGISTRY" >&2
  exit 1
fi

pass_output="$(
  bash "$CHECKER" \
    --repo-root "$ROOT_DIR" \
    --registry-file "$REGISTRY"
)"

printf '%s\n' "$pass_output" | grep -q '^status=ok$'
printf '%s\n' "$pass_output" | grep -q '^final_decision=GO$'
printf '%s\n' "$pass_output" | grep -q '^reason_taxonomy_version=kamn.framework.lane-registry-drift-reason-taxonomy.v1$'
printf '%s\n' "$pass_output" | grep -q '^reason_codes=none$'

MINI_ROOT="$TMP_DIR/mini-repo"
mkdir -p "$MINI_ROOT/scripts/framework/manifests"

MINI_MANIFEST_REL="scripts/framework/manifests/demo_manifest.json"
MINI_MANIFEST="$MINI_ROOT/$MINI_MANIFEST_REL"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$MINI_MANIFEST" <<'JSON'
{
  "schema_version": "kamn.contract-lane.manifest.v1",
  "lane_id": "demo.lane.contract",
  "evidence_key": "demo_lane:v1",
  "reason_key": "demo_reason_codes:GO:v1",
  "phases": {
    "contract": ["bash", "scripts/demo/run_demo_contract_lane_impl.sh"]
  }
}
JSON

MINI_REGISTRY="$TMP_DIR/mini-lane-registry.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$MINI_REGISTRY" <<JSON
{
  "schema_version": "kamn.framework.lane-registry.v1",
  "registry_version": "test",
  "manifest_count": 1,
  "wrapper_count": 0,
  "manifests": [
    {
      "manifest_relpath": "$MINI_MANIFEST_REL",
      "manifest_payload": {
        "schema_version": "kamn.contract-lane.manifest.v1",
        "lane_id": "demo.lane.contract",
        "evidence_key": "demo_lane:v1",
        "reason_key": "demo_reason_codes:GO:v1",
        "phases": {
          "contract": ["bash", "scripts/demo/run_demo_contract_lane_impl.sh"]
        }
      }
    }
  ],
  "wrappers": []
}
JSON

echo '{"schema_version":"tampered"}' > "$MINI_MANIFEST"

set +e
fail_output="$(
  bash "$CHECKER" \
    --repo-root "$MINI_ROOT" \
    --registry-file "$MINI_REGISTRY" 2>&1
)"
fail_status=$?
set -e

if [ "$fail_status" -eq 0 ]; then
  echo "expected drift checker to fail for tampered manifest payload" >&2
  exit 1
fi

printf '%s\n' "$fail_output" | grep -q '^status=fail$'
printf '%s\n' "$fail_output" | grep -q '^final_decision=NO-GO$'
printf '%s\n' "$fail_output" | grep -q '^reason_taxonomy_version=kamn.framework.lane-registry-drift-reason-taxonomy.v1$'
printf '%s\n' "$fail_output" | grep -q 'lane_registry_manifest_drift_detected'

echo "lane registry drift checker tests passed."
