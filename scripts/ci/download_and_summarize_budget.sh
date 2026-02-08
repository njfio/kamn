#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<USAGE
Usage: $0 [--repo owner/repo] [--lane fast-gate|deep-validate] [--limit N]

Downloads recent ci-budget artifacts from GitHub Actions and summarizes them.
Requires: gh, jq, unzip
USAGE
}

REPO="${GITHUB_REPOSITORY:-}"
LANE=""
LIMIT=20

while [ "$#" -gt 0 ]; do
  case "$1" in
    --repo)
      REPO="${2:-}"
      shift 2
      ;;
    --lane)
      LANE="${2:-}"
      shift 2
      ;;
    --limit)
      LIMIT="${2:-20}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if [ -z "$REPO" ]; then
  echo "--repo is required (or set GITHUB_REPOSITORY)" >&2
  exit 2
fi

for cmd in gh jq unzip; do
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "Missing dependency: $cmd" >&2
    exit 2
  fi
done

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

# Fetch recent artifacts for budget telemetry files
artifacts_json="$TMP_DIR/artifacts.json"
gh api "repos/$REPO/actions/artifacts?per_page=100" > "$artifacts_json"

jq -r '.artifacts[] | select(.expired == false) | [.id, .name] | @tsv' "$artifacts_json" | while IFS=$'\t' read -r artifact_id artifact_name; do
  case "$artifact_name" in
    ci-budget-fast-gate-*|ci-budget-deep-validate-*)
      out_zip="$TMP_DIR/${artifact_id}.zip"
      out_dir="$TMP_DIR/${artifact_id}"
      gh api "repos/$REPO/actions/artifacts/$artifact_id/zip" > "$out_zip"
      mkdir -p "$out_dir"
      unzip -oq "$out_zip" -d "$out_dir"
      ;;
  esac
done

mapfile -t json_files < <(find "$TMP_DIR" -type f -name 'ci-budget-*.json' | sort | head -n "$LIMIT")

if [ "${#json_files[@]}" -eq 0 ]; then
  echo "No CI budget artifacts found for repo $REPO"
  exit 0
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [ -n "$LANE" ]; then
  bash "$SCRIPT_DIR/summarize_budget_artifacts.sh" --lane "$LANE" "${json_files[@]}"
else
  bash "$SCRIPT_DIR/summarize_budget_artifacts.sh" "${json_files[@]}"
fi
