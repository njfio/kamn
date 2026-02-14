#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LANE_SCRIPT="$ROOT_DIR/scripts/signer/run_signer_provider_deep_lane.sh"
LANE_IMPL="$ROOT_DIR/scripts/signer/run_signer_provider_deep_lane_impl.sh"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/signer_signer_provider_deep_lane.json"

if [ ! -x "$LANE_SCRIPT" ]; then
  echo "expected signer provider deep lane wrapper to be executable" >&2
  exit 1
fi

if [ ! -x "$DISPATCHER" ]; then
  echo "expected shared non-Kolme dispatcher to be executable" >&2
  exit 1
fi

if [ ! -x "$LANE_IMPL" ]; then
  echo "expected signer provider deep lane implementation to be executable" >&2
  exit 1
fi

if [ ! -L "$LANE_SCRIPT" ]; then
  echo "expected signer provider deep lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi

if [ "$(readlink "$LANE_SCRIPT")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected signer provider deep lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$LANE_SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected signer provider deep lane wrapper to resolve signer manifest via dispatcher" >&2
  exit 1
fi

if ! grep -Fq "run_signer_provider_deep_lane_impl.sh" "$MANIFEST_FILE"; then
  echo "expected signer provider deep lane manifest to dispatch to shared implementation module" >&2
  exit 1
fi

if ! grep -Fq "performance_signer_emulator_bulk_signing_deep_lane -- --ignored" "$LANE_IMPL"; then
  echo "expected signer provider deep implementation to execute ignored signer provider stress test" >&2
  exit 1
fi

if ! grep -Fq "run_signer_incident_recovery_deep_lane.sh" "$LANE_IMPL"; then
  echo "expected signer provider deep implementation to execute signer incident recovery deep lane" >&2
  exit 1
fi

if ! grep -Fq "KAMN_SIGNER_INCIDENT_RECOVERY_DEEP_CADENCE=scheduled" "$LANE_IMPL"; then
  echo "expected signer provider deep implementation to set scheduled cadence guard for incident recovery deep lane" >&2
  exit 1
fi

echo "signer provider deep lane script tests passed."
