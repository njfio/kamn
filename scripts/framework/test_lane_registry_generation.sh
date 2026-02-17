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

echo "lane registry generation tests passed."
