#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FIXTURE_FILE="$ROOT_DIR/fixtures/kolme_commit/runtime_commit_request_cases.txt"
ROADMAP_DOC="$ROOT_DIR/docs/planning/kolme-integration-roadmap.md"

if [ ! -f "$FIXTURE_FILE" ]; then
  echo "expected Kolme runtime commit fixture file to exist" >&2
  exit 1
fi

if [ ! -f "$ROADMAP_DOC" ]; then
  echo "expected Kolme integration roadmap doc to exist" >&2
  exit 1
fi

start_epoch="$(date +%s)"

cargo test -p kamn-core \
  --test kolme_runtime_commit_client \
  --test kolme_runtime_commit_finality

if ! grep -q "run_runtime_commit_contract_lane.sh" "$ROADMAP_DOC"; then
  echo "expected Kolme integration roadmap to reference runtime commit contract lane command" >&2
  exit 1
fi

if ! grep -q "fixtures/kolme_commit/runtime_commit_request_cases.txt" "$ROADMAP_DOC"; then
  echo "expected Kolme integration roadmap to reference runtime commit fixture path" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt 60 ]; then
  echo "Kolme runtime commit contract lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

echo "Kolme runtime commit contract lane tests passed."
