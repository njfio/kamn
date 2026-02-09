#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any


def _load_snapshot(path: Path) -> dict[str, Any]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    if payload.get("schema_version") != "kamn.kolme.compatibility-snapshot.v1":
        raise ValueError("unsupported snapshot schema_version")
    return payload


def _flatten_snapshot(payload: dict[str, Any]) -> dict[str, str]:
    repository = payload.get("kolme_repository")
    upstream = payload.get("upstream")
    docs_contracts = payload.get("docs_contracts")
    protocols = payload.get("protocols")

    if not isinstance(repository, dict):
        raise ValueError("kolme_repository must be an object")
    if not isinstance(upstream, dict):
        raise ValueError("upstream must be an object")
    if not isinstance(docs_contracts, list):
        raise ValueError("docs_contracts must be an array")
    if not isinstance(protocols, list):
        raise ValueError("protocols must be an array")

    flattened: dict[str, str] = {}
    owner = str(repository.get("owner", "")).strip()
    repo = str(repository.get("repo", "")).strip()
    release_tag = str(upstream.get("release_tag", "")).strip()
    commit_sha = str(upstream.get("commit_sha", "")).strip()

    if not owner or not repo:
        raise ValueError("kolme_repository owner/repo are required")
    if not release_tag or not commit_sha:
        raise ValueError("upstream release_tag/commit_sha are required")

    flattened["repository.owner"] = owner
    flattened["repository.repo"] = repo
    flattened["upstream.release_tag"] = release_tag
    flattened["upstream.commit_sha"] = commit_sha

    for item in docs_contracts:
        if not isinstance(item, dict):
            raise ValueError("docs_contracts entries must be objects")
        path = str(item.get("path", "")).strip()
        sha256 = str(item.get("sha256", "")).strip()
        if not path or not sha256:
            raise ValueError("docs_contracts entries require path and sha256")
        flattened[f"docs.{path}"] = sha256

    for item in protocols:
        if not isinstance(item, dict):
            raise ValueError("protocols entries must be objects")
        name = str(item.get("name", "")).strip()
        version = str(item.get("version", "")).strip()
        if not name or not version:
            raise ValueError("protocols entries require name and version")
        flattened[f"protocols.{name}.version"] = version

    return flattened


def _compare(
    baseline: dict[str, str], candidate: dict[str, str]
) -> list[dict[str, str]]:
    changed_fields: list[dict[str, str]] = []
    all_keys = sorted(set(baseline.keys()) | set(candidate.keys()))
    for key in all_keys:
        expected = baseline.get(key, "<missing>")
        actual = candidate.get(key, "<missing>")
        if expected == actual:
            continue
        changed_fields.append(
            {
                "field": key,
                "expected": expected,
                "actual": actual,
            }
        )
    return changed_fields


def _parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Compare baseline/candidate Kolme compatibility snapshots deterministically."
    )
    parser.add_argument("--baseline-file", required=True)
    parser.add_argument("--candidate-file", required=True)
    parser.add_argument("--output-json", default="")
    return parser.parse_args()


def main() -> int:
    args = _parse_args()
    baseline_file = Path(args.baseline_file).resolve()
    candidate_file = Path(args.candidate_file).resolve()

    baseline_payload = _load_snapshot(baseline_file)
    candidate_payload = _load_snapshot(candidate_file)
    baseline_flat = _flatten_snapshot(baseline_payload)
    candidate_flat = _flatten_snapshot(candidate_payload)

    changed_fields = _compare(baseline_flat, candidate_flat)
    status = "pass" if not changed_fields else "fail"
    changed_field_names = ",".join(item["field"] for item in changed_fields)

    report = {
        "schema_version": "kamn.kolme.snapshot-drift-report.v1",
        "baseline_file": str(baseline_file),
        "candidate_file": str(candidate_file),
        "status": status,
        "changed_fields_count": len(changed_fields),
        "changed_fields": changed_fields,
    }

    if args.output_json:
        output_file = Path(args.output_json).resolve()
        output_file.parent.mkdir(parents=True, exist_ok=True)
        output_file.write_text(json.dumps(report, sort_keys=True, indent=2) + "\n", encoding="utf-8")

    print(f"status={status}")
    print(f"changed_fields_count={len(changed_fields)}")
    print(f"changed_fields={changed_field_names if changed_field_names else 'none'}")
    if args.output_json:
        print(f"report_file={Path(args.output_json).resolve()}")

    return 0 if status == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
