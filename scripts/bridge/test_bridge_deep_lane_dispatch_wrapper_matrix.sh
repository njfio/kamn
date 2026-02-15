#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"

if [ ! -x "$DISPATCHER" ]; then
  echo "expected non-Kolme dispatcher to be executable: $DISPATCHER" >&2
  exit 1
fi

wrapper_names=(
  "run_bridge_credentialed_deep_lane.sh"
  "run_bridge_replay_redaction_deep_lane.sh"
  "run_cross_chain_outbound_intent_deep_lane.sh"
)

expected_manifests=(
  "bridge_bridge_credentialed_deep_lane.json"
  "bridge_bridge_replay_redaction_deep_lane.json"
  "bridge_cross_chain_outbound_intent_deep_lane.json"
)

expected_impl_paths=(
  "scripts/bridge/run_bridge_credentialed_deep_lane_impl.sh"
  "scripts/bridge/run_bridge_replay_redaction_deep_lane_impl.sh"
  "scripts/bridge/run_cross_chain_outbound_intent_deep_lane_impl.sh"
)

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

for index in "${!wrapper_names[@]}"; do
  wrapper_name="${wrapper_names[$index]}"
  expected_manifest="$ROOT_DIR/scripts/framework/manifests/${expected_manifests[$index]}"
  expected_impl="${expected_impl_paths[$index]}"
  wrapper_path="$ROOT_DIR/scripts/bridge/$wrapper_name"
  output_json="$TMP_DIR/${wrapper_name%.sh}.json"

  if [ ! -x "$wrapper_path" ]; then
    echo "expected deep lane wrapper to be executable: $wrapper_path" >&2
    exit 1
  fi
  if [ ! -L "$wrapper_path" ]; then
    echo "expected deep lane wrapper to be a dispatcher symlink: $wrapper_path" >&2
    exit 1
  fi
  if [ "$(readlink "$wrapper_path")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
    echo "expected deep lane wrapper symlink target to be shared dispatcher: $wrapper_path" >&2
    exit 1
  fi

  resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$wrapper_name" --resolve-manifest-path)"
  if [ "$resolved_manifest" != "$expected_manifest" ]; then
    echo "expected deep lane wrapper to resolve expected manifest: $wrapper_name" >&2
    exit 1
  fi
  if ! grep -Fq "$expected_impl" "$expected_manifest"; then
    echo "expected deep lane manifest to dispatch expected impl: $expected_impl" >&2
    exit 1
  fi

  wrapper_output="$(bash "$wrapper_path" --output-json "$output_json")"
  if ! printf '%s\n' "$wrapper_output" | grep -q 'deep lane tests passed'; then
    echo "expected deep lane wrapper success marker for $wrapper_name" >&2
    exit 1
  fi

  if [ ! -f "$output_json" ]; then
    echo "expected deep lane wrapper to emit output report: $output_json" >&2
    exit 1
  fi

  python3 - "$wrapper_name" "$output_json" <<'PY'
import json
import pathlib
import sys

wrapper_name = sys.argv[1]
payload = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))

if wrapper_name == "run_bridge_replay_redaction_deep_lane.sh":
    if payload.get("schema_version") != "kamn.bridge.replay-redaction-evidence.v1":
        raise SystemExit("unexpected replay/redaction deep schema marker")
    if payload.get("lane") != "deep":
        raise SystemExit("unexpected replay/redaction lane marker")
    if payload.get("final_decision") != "GO":
        raise SystemExit("unexpected replay/redaction final decision marker")
else:
    if payload.get("status") != "pass":
        raise SystemExit("expected deep matrix status=pass")
    if payload.get("failed_count") != 0:
        raise SystemExit("expected deep matrix failed_count=0")
PY
done

echo "bridge deep-lane dispatcher wrapper matrix tests passed."
