#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/framework/generate_lane_artifacts.py"
REGISTRY="$ROOT_DIR/scripts/framework/lane_registry.json"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$GENERATOR" ]; then
  echo "expected lane artifact generator to be executable: $GENERATOR" >&2
  exit 1
fi

if [ ! -f "$REGISTRY" ]; then
  echo "expected lane registry source to exist: $REGISTRY" >&2
  exit 1
fi

check_output="$(
  python3 "$GENERATOR" \
    --registry-file "$REGISTRY" \
    --repo-root "$ROOT_DIR" \
    --mode check
)"

printf '%s\n' "$check_output" | grep -q '^status=ok$'
printf '%s\n' "$check_output" | grep -q '^validation_mode=check$'
printf '%s\n' "$check_output" | grep -q '^manifest_entries='
printf '%s\n' "$check_output" | grep -q '^wrapper_entries='
printf '%s\n' "$check_output" | grep -q '^registry_schema_version=kamn.framework.lane-registry.v1$'

render_dir="$TMP_DIR/rendered"
render_output="$(
  python3 "$GENERATOR" \
    --registry-file "$REGISTRY" \
    --repo-root "$ROOT_DIR" \
    --mode render \
    --output-root "$render_dir"
)"

printf '%s\n' "$render_output" | grep -q '^status=ok$'
printf '%s\n' "$render_output" | grep -q '^validation_mode=render$'

render_manifest="$render_dir/scripts/framework/manifests/bridge_bridge_adapter_conformance_contract_lane.json"
if [ ! -f "$render_manifest" ]; then
  echo "expected rendered bridge adapter conformance manifest at $render_manifest" >&2
  exit 1
fi

render_wrapper="$render_dir/scripts/bridge/run_bridge_adapter_conformance_contract_lane.sh"
if [ ! -L "$render_wrapper" ]; then
  echo "expected rendered bridge adapter conformance wrapper symlink at $render_wrapper" >&2
  exit 1
fi

invalid_schema_registry="$TMP_DIR/invalid-schema-registry.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$invalid_schema_registry" <<'JSON'
{
  "schema_version": "kamn.framework.lane-registry.invalid.v1",
  "manifests": [],
  "wrappers": []
}
JSON

set +e
invalid_schema_output="$(
  python3 "$GENERATOR" \
    --registry-file "$invalid_schema_registry" \
    --repo-root "$ROOT_DIR" \
    --mode check 2>&1
)"
invalid_schema_code=$?
set -e

if [ "$invalid_schema_code" -eq 0 ]; then
  echo "expected lane artifact generator to fail on registry schema mismatch" >&2
  exit 1
fi

printf '%s\n' "$invalid_schema_output" | grep -q '^status=fail$'
printf '%s\n' "$invalid_schema_output" | grep -q '^error=registry schema_version mismatch$'

invalid_wrapper_registry="$TMP_DIR/invalid-wrapper-registry.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$invalid_wrapper_registry" <<'JSON'
{
  "schema_version": "kamn.framework.lane-registry.v1",
  "manifests": [],
  "wrappers": [
    {
      "wrapper_relpath": "scripts/demo/run_demo_contract_lane.sh",
      "wrapper_name": "run_other_contract_lane.sh",
      "link_target": "../framework/run_non_kolme_contract_lane_dispatch.sh"
    }
  ]
}
JSON

set +e
invalid_wrapper_output="$(
  python3 "$GENERATOR" \
    --registry-file "$invalid_wrapper_registry" \
    --repo-root "$ROOT_DIR" \
    --mode check 2>&1
)"
invalid_wrapper_code=$?
set -e

if [ "$invalid_wrapper_code" -eq 0 ]; then
  echo "expected lane artifact generator to fail on wrapper shape mismatch" >&2
  exit 1
fi

printf '%s\n' "$invalid_wrapper_output" | grep -q '^status=fail$'
printf '%s\n' "$invalid_wrapper_output" | grep -q 'wrapper_relpath basename mismatch'

echo "lane registry generation tests passed."
