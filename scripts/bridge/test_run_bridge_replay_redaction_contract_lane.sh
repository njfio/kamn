#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_SCRIPT="$ROOT_DIR/scripts/bridge/run_bridge_replay_redaction_contract_lane.sh"
FAST_IMPL_SCRIPT="$ROOT_DIR/scripts/bridge/run_bridge_replay_redaction_contract_lane_impl.sh"
DEEP_SCRIPT="$ROOT_DIR/scripts/bridge/run_bridge_replay_redaction_deep_lane.sh"
DEEP_IMPL_SCRIPT="$ROOT_DIR/scripts/bridge/run_bridge_replay_redaction_deep_lane_impl.sh"
REPLAY_SCRIPT="$ROOT_DIR/scripts/bridge/run_bridge_replay_matrix.sh"
REPLAY_FIXTURE="$ROOT_DIR/fixtures/bridge_replay/replay_validation_cases.json"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/bridge_bridge_replay_redaction_contract_lane.json"
DEEP_MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/bridge_bridge_replay_redaction_deep_lane.json"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"

if [ ! -x "$FAST_SCRIPT" ]; then
  echo "expected bridge replay/redaction contract lane script to be executable" >&2
  exit 1
fi

if [ ! -x "$DEEP_SCRIPT" ]; then
  echo "expected bridge replay/redaction deep lane script to be executable" >&2
  exit 1
fi

if [ ! -x "$DEEP_IMPL_SCRIPT" ]; then
  echo "expected bridge replay/redaction deep lane implementation script to be executable" >&2
  exit 1
fi

if [ ! -x "$FAST_IMPL_SCRIPT" ]; then
  echo "expected bridge replay/redaction contract lane implementation script to be executable" >&2
  exit 1
fi

if [ ! -f "$MANIFEST_FILE" ]; then
  echo "expected bridge replay/redaction contract lane manifest to exist" >&2
  exit 1
fi

if [ ! -f "$DEEP_MANIFEST_FILE" ]; then
  echo "expected bridge replay/redaction deep lane manifest to exist" >&2
  exit 1
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

replay_report="$TMP_DIR/replay-report.json"
bash "$REPLAY_SCRIPT" \
  --fixture "$REPLAY_FIXTURE" \
  --suites "bridge_adapter,discord_bridge" \
  --output-json "$replay_report" >/dev/null

bundle_file="$TMP_DIR/bridge-replay-redaction-contract-bundle.json"
lane_output="$(bash "$FAST_SCRIPT" --skip-replay --replay-report-file "$replay_report" --output-bundle "$bundle_file")"
if ! printf '%s\n' "$lane_output" | grep -q "bridge replay redaction contract lane tests passed."; then
  echo "expected bridge replay/redaction contract lane success marker" >&2
  exit 1
fi

if [ ! -f "$bundle_file" ]; then
  echo "expected bridge replay/redaction contract lane to emit evidence bundle" >&2
  exit 1
fi

if ! grep -q '"schema_version": "kamn.bridge.replay-redaction-evidence.v1"' "$bundle_file"; then
  echo "expected bridge replay/redaction evidence bundle schema version marker" >&2
  exit 1
fi

if ! grep -q '"final_decision": "GO"' "$bundle_file"; then
  echo "expected bridge replay/redaction contract bundle GO final decision" >&2
  exit 1
fi

if [ ! -L "$FAST_SCRIPT" ]; then
  echo "expected bridge replay/redaction contract lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi

if [ "$(readlink "$FAST_SCRIPT")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected bridge replay/redaction contract lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$FAST_SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected bridge replay/redaction wrapper to resolve bridge manifest via dispatcher" >&2
  exit 1
fi

if ! grep -Fq "run_bridge_replay_redaction_contract_lane_impl.sh" "$MANIFEST_FILE"; then
  echo "expected bridge replay/redaction manifest to dispatch to implementation script" >&2
  exit 1
fi

if ! grep -q -- "--skip-replay" "$FAST_IMPL_SCRIPT"; then
  echo "expected bridge replay/redaction implementation lane to support skip-replay mode" >&2
  exit 1
fi

if [ ! -L "$DEEP_SCRIPT" ]; then
  echo "expected bridge replay/redaction deep lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi

if [ "$(readlink "$DEEP_SCRIPT")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected bridge replay/redaction deep lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_deep_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$DEEP_SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_deep_manifest" != "$DEEP_MANIFEST_FILE" ]; then
  echo "expected bridge replay/redaction deep wrapper to resolve bridge deep manifest via dispatcher" >&2
  exit 1
fi

if ! grep -Fq "run_bridge_replay_redaction_deep_lane_impl.sh" "$DEEP_MANIFEST_FILE"; then
  echo "expected bridge replay/redaction deep manifest to dispatch to implementation script" >&2
  exit 1
fi

if ! grep -Fq "run_bridge_replay_redaction_contract_lane.sh" "$DEEP_IMPL_SCRIPT"; then
  echo "expected bridge replay/redaction deep implementation lane to execute contract lane baseline checks first" >&2
  exit 1
fi

if ! grep -q "bridge-replay-redaction-deep-bundle.json" "$DEEP_IMPL_SCRIPT"; then
  echo "expected bridge replay/redaction deep implementation lane to emit deep evidence bundle artifact" >&2
  exit 1
fi

echo "bridge replay/redaction contract lane script tests passed."
