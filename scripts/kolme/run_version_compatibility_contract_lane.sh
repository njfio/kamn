#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATOR="$ROOT_DIR/scripts/kolme/validate_version_compatibility.py"
FORK_EVIDENCE_GENERATOR="$ROOT_DIR/scripts/kolme/generate_fork_compatibility_evidence.py"
REPLAY_RUNNER="$ROOT_DIR/scripts/kolme/run_version_compatibility_replay.py"
RUNTIME_COMMIT_LANE="$ROOT_DIR/scripts/kolme/run_runtime_commit_contract_lane.sh"
RUNTIME_COMMIT_REPLAY_LANE="$ROOT_DIR/scripts/kolme/run_runtime_commit_replay_contract_lane.sh"
FIXTURE_FILE="$ROOT_DIR/fixtures/kolme_compatibility/version_compatibility_cases.json"
FORK_FIXTURE_FILE="$ROOT_DIR/fixtures/kolme_compatibility/fork_compatibility_cases.json"
ROADMAP_DOC="$ROOT_DIR/docs/planning/kolme-integration-roadmap.md"
GONOGO_DOC="$ROOT_DIR/docs/foundation/release-gonogo-checklist.md"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$VALIDATOR" ]; then
  echo "expected Kolme version compatibility validator to be executable" >&2
  exit 1
fi

if [ ! -x "$FORK_EVIDENCE_GENERATOR" ]; then
  echo "expected Kolme fork compatibility evidence generator to be executable" >&2
  exit 1
fi

if [ ! -x "$REPLAY_RUNNER" ]; then
  echo "expected Kolme version compatibility replay runner to be executable" >&2
  exit 1
fi

if [ ! -x "$RUNTIME_COMMIT_LANE" ]; then
  echo "expected Kolme runtime commit contract lane script to be executable" >&2
  exit 1
fi

if [ ! -x "$RUNTIME_COMMIT_REPLAY_LANE" ]; then
  echo "expected Kolme runtime commit replay contract lane script to be executable" >&2
  exit 1
fi

if [ ! -f "$FIXTURE_FILE" ]; then
  echo "expected Kolme version compatibility fixture file to exist" >&2
  exit 1
fi

if [ ! -f "$FORK_FIXTURE_FILE" ]; then
  echo "expected Kolme fork compatibility fixture file to exist" >&2
  exit 1
fi

if [ ! -f "$ROADMAP_DOC" ] || [ ! -f "$GONOGO_DOC" ]; then
  echo "expected Kolme roadmap and release go/no-go docs to exist" >&2
  exit 1
fi

start_epoch="$(date +%s)"

go_output="$(
  python3 "$VALIDATOR" \
    --kamn-version "1.1.0" \
    --kolme-release-tag "v0.15.2" \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/go-report.json"
)"
if ! printf '%s\n' "$go_output" | grep -q '^final_decision=GO$'; then
  echo "expected supported Kolme/KAMN version pair to produce GO" >&2
  exit 1
fi

set +e
no_go_output="$(
  python3 "$VALIDATOR" \
    --kamn-version "1.2.0" \
    --kolme-release-tag "v0.14.9" \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/no-go-report.json" 2>&1
)"
no_go_code=$?
set -e
if [ "$no_go_code" -eq 0 ]; then
  echo "expected unsupported Kolme/KAMN version pair to fail closed" >&2
  exit 1
fi
if ! printf '%s\n' "$no_go_output" | grep -q '^final_decision=NO-GO$'; then
  echo "expected unsupported Kolme/KAMN version pair to produce NO-GO" >&2
  exit 1
fi

fork_go_output="$(
  python3 "$FORK_EVIDENCE_GENERATOR" \
    --upstream-release-tag "v0.15.2" \
    --fork-release-tag "v0.15.2" \
    --fork-repo "njfio/kolme_fork" \
    --fork-ref "refs/heads/main" \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/fork-go-report.json"
)"
if ! printf '%s\n' "$fork_go_output" | grep -q '^final_decision=GO$'; then
  echo "expected synced fork tuple to produce GO" >&2
  exit 1
fi

set +e
fork_no_go_output="$(
  python3 "$FORK_EVIDENCE_GENERATOR" \
    --upstream-release-tag "v0.15.2" \
    --fork-release-tag "v0.14.9" \
    --fork-repo "njfio/kolme_fork" \
    --fork-ref "refs/heads/main" \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/fork-no-go-report.json" 2>&1
)"
fork_no_go_code=$?
set -e
if [ "$fork_no_go_code" -eq 0 ]; then
  echo "expected drifted fork tuple to fail closed" >&2
  exit 1
fi
if ! printf '%s\n' "$fork_no_go_output" | grep -q '^final_decision=NO-GO$'; then
  echo "expected drifted fork tuple to produce NO-GO" >&2
  exit 1
fi
if ! printf '%s\n' "$fork_no_go_output" | grep -q 'fork_release_tag_mismatch'; then
  echo "expected drifted fork tuple to emit fork_release_tag_mismatch reason code" >&2
  exit 1
fi

python3 "$REPLAY_RUNNER" \
  --fixture "$FIXTURE_FILE" \
  --max-cases 2 \
  --output-json "$TMP_DIR/replay-smoke.json" \
  >/dev/null

bash "$RUNTIME_COMMIT_LANE" >/dev/null
bash "$RUNTIME_COMMIT_REPLAY_LANE" >/dev/null

if ! grep -q "validate_version_compatibility.py" "$ROADMAP_DOC"; then
  echo "expected Kolme roadmap doc to reference version validator command" >&2
  exit 1
fi

if ! grep -q "run_runtime_commit_contract_lane.sh" "$ROADMAP_DOC"; then
  echo "expected Kolme roadmap doc to reference runtime commit contract lane command" >&2
  exit 1
fi

if ! grep -q "generate_fork_compatibility_evidence.py" "$ROADMAP_DOC"; then
  echo "expected Kolme roadmap doc to reference fork compatibility evidence command" >&2
  exit 1
fi

if ! grep -q "fixtures/kolme_compatibility/fork_compatibility_cases.json" "$ROADMAP_DOC"; then
  echo "expected Kolme roadmap doc to reference fork compatibility fixture path" >&2
  exit 1
fi

if ! grep -q "run_runtime_commit_replay_contract_lane.sh" "$ROADMAP_DOC"; then
  echo "expected Kolme roadmap doc to reference runtime commit replay contract lane command" >&2
  exit 1
fi

if ! grep -q "run_version_compatibility_replay_deep_lane.sh" "$GONOGO_DOC"; then
  echo "expected release go/no-go doc to reference scheduled version replay lane" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt 60 ]; then
  echo "Kolme version compatibility contract lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

echo "Kolme version compatibility contract lane tests passed."
