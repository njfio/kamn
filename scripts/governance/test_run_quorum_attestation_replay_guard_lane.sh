#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LANE_SCRIPT="$ROOT_DIR/scripts/governance/run_quorum_attestation_replay_guard_lane.sh"
LANE_IMPL="$ROOT_DIR/scripts/governance/run_quorum_attestation_replay_guard_lane_impl.sh"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/governance_quorum_attestation_replay_guard_lane.json"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

extract_value() {
  local output="$1"
  local key="$2"
  printf '%s\n' "$output" | awk -F= -v key="$key" '$1 == key { print $2; exit }'
}

if [ ! -x "$LANE_SCRIPT" ]; then
  echo "expected governance quorum attestation lane script to be executable" >&2
  exit 1
fi
if [ ! -x "$LANE_IMPL" ]; then
  echo "expected governance quorum attestation lane implementation to be executable" >&2
  exit 1
fi
if [ ! -x "$DISPATCHER" ]; then
  echo "expected shared non-Kolme dispatcher to be executable" >&2
  exit 1
fi
if [ ! -L "$LANE_SCRIPT" ]; then
  echo "expected governance quorum attestation lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi
if [ "$(readlink "$LANE_SCRIPT")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected governance quorum attestation lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi
resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$LANE_SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected governance quorum attestation lane wrapper to resolve governance manifest via dispatcher" >&2
  exit 1
fi
if ! grep -q 'run_quorum_attestation_replay_guard_lane_impl.sh' "$MANIFEST_FILE"; then
  echo "expected governance quorum attestation lane manifest to dispatch implementation module" >&2
  exit 1
fi

go_report="$TMP_DIR/governance-quorum-attestation-go.json"
go_output="$(
  KAMN_GOVERNANCE_QUORUM_ATTESTATION_SKIP_COMMANDS=true \
    bash "$LANE_SCRIPT" --output-file "$go_report"
)"
if [ "$(extract_value "$go_output" "status")" != "ok" ]; then
  echo "expected governance quorum attestation GO path status=ok" >&2
  exit 1
fi
if [ "$(extract_value "$go_output" "final_decision")" != "GO" ]; then
  echo "expected governance quorum attestation GO path final_decision=GO" >&2
  exit 1
fi
if [ "$(extract_value "$go_output" "reason_key")" != "governance_quorum_attestation_reason_codes:GO:v1" ]; then
  echo "expected governance quorum attestation GO path reason_key marker" >&2
  exit 1
fi
if ! grep -q '"schema_version": "kamn.governance.quorum-attestation-replay-report.v1"' "$go_report"; then
  echo "expected governance quorum attestation report schema marker" >&2
  exit 1
fi

no_go_report="$TMP_DIR/governance-quorum-attestation-no-go.json"
no_go_output="$(
  KAMN_GOVERNANCE_QUORUM_ATTESTATION_SKIP_COMMANDS=true \
  KAMN_GOVERNANCE_QUORUM_ATTESTATION_FORCE_REPLAY_DETECTED=true \
    bash "$LANE_SCRIPT" --output-file "$no_go_report"
)"
if [ "$(extract_value "$no_go_output" "final_decision")" != "NO-GO" ]; then
  echo "expected governance quorum attestation replay path final_decision=NO-GO" >&2
  exit 1
fi
if [ "$(extract_value "$no_go_output" "reason_key")" != "governance_quorum_attestation_reason_codes:NO-GO:v1" ]; then
  echo "expected governance quorum attestation replay path reason_key marker" >&2
  exit 1
fi
if ! grep -q '"quorum_attestation_replay_detected"' "$no_go_report"; then
  echo "expected governance quorum attestation replay reason in NO-GO report" >&2
  exit 1
fi

echo "governance quorum attestation replay lane script tests passed."
