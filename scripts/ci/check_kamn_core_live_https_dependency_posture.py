#!/usr/bin/env python3
"""Validate kamn-core live HTTPS dependency posture contracts."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import tomllib

SCHEMA_VERSION = "kamn.ci.kamn-core-live-https-dependency-posture-report.v1"
REASON_TAXONOMY_VERSION = (
    "kamn.ci.kamn-core-live-https-dependency-posture-reason-taxonomy.v1"
)
EXPECTED_DEPS = ("rustls", "rustls-pemfile", "webpki-roots")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--cargo-manifest",
        default="crates/kamn-core/Cargo.toml",
        help="Path to kamn-core Cargo manifest.",
    )
    parser.add_argument(
        "--readme",
        default="README.md",
        help="Path to repository README.",
    )
    parser.add_argument(
        "--adr",
        default="docs/architecture/adr-kamn-core-live-tls-transport.md",
        help="Path to live TLS transport ADR.",
    )
    parser.add_argument(
        "--ci-strategy",
        default="docs/ci/strategy.md",
        help="Path to CI strategy document.",
    )
    parser.add_argument(
        "--output-json",
        default="",
        help="Optional output file path for report JSON.",
    )
    return parser.parse_args()


def read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def normalize_feature_entries(value: object) -> list[str]:
    if not isinstance(value, list):
        return []
    normalized: list[str] = []
    for item in value:
        if isinstance(item, str):
            normalized.append(item.strip())
    return normalized


def main() -> int:
    args = parse_args()

    manifest_path = Path(args.cargo_manifest)
    readme_path = Path(args.readme)
    adr_path = Path(args.adr)
    ci_strategy_path = Path(args.ci_strategy)

    checks: dict[str, str] = {}
    violations: list[str] = []
    reason_code_set: set[str] = set()

    for required_path, label in (
        (manifest_path, "cargo_manifest"),
        (readme_path, "readme"),
        (adr_path, "adr"),
        (ci_strategy_path, "ci_strategy"),
    ):
        if required_path.exists():
            checks[f"{label}_exists"] = "pass"
        else:
            checks[f"{label}_exists"] = "fail"
            violations.append(f"required file is missing: {required_path}")
            reason_code_set.add(f"{label}_file_missing")

    manifest_data: dict[str, object] = {}
    if manifest_path.exists():
        try:
            manifest_data = tomllib.loads(read_text(manifest_path))
        except tomllib.TOMLDecodeError as exc:
            checks["manifest_parses"] = "fail"
            violations.append(f"failed to parse Cargo manifest: {exc}")
            reason_code_set.add("cargo_manifest_parse_failed")
        else:
            checks["manifest_parses"] = "pass"
    else:
        checks["manifest_parses"] = "fail"

    features = manifest_data.get("features") if isinstance(manifest_data, dict) else None
    dependencies = (
        manifest_data.get("dependencies") if isinstance(manifest_data, dict) else None
    )

    live_https_entries = normalize_feature_entries(
        features.get("live-https") if isinstance(features, dict) else None
    )

    for dep in EXPECTED_DEPS:
        expected_feature = f"dep:{dep}"
        key_prefix = dep.replace("-", "_")
        if expected_feature in live_https_entries:
            checks[f"{key_prefix}_feature_mapping"] = "pass"
        else:
            checks[f"{key_prefix}_feature_mapping"] = "fail"
            violations.append(
                f"live-https feature must include mapping `{expected_feature}`"
            )
            reason_code_set.add(f"{key_prefix}_feature_mapping_missing")

        dep_config = dependencies.get(dep) if isinstance(dependencies, dict) else None
        if dep_config is None:
            checks[f"{key_prefix}_dependency_declared"] = "fail"
            violations.append(f"dependency `{dep}` must be declared under [dependencies]")
            reason_code_set.add(f"{key_prefix}_dependency_missing")
            continue

        checks[f"{key_prefix}_dependency_declared"] = "pass"
        if isinstance(dep_config, dict) and dep_config.get("optional") is True:
            checks[f"{key_prefix}_dependency_optional"] = "pass"
        else:
            checks[f"{key_prefix}_dependency_optional"] = "fail"
            violations.append(f"dependency `{dep}` must declare optional = true")
            reason_code_set.add(f"{key_prefix}_dependency_optional_flag_mismatch")

    if isinstance(dependencies, dict):
        rustls_dep = dependencies.get("rustls")
        if isinstance(rustls_dep, dict) and rustls_dep.get("default-features") is False:
            checks["rustls_default_features_disabled"] = "pass"
        else:
            checks["rustls_default_features_disabled"] = "fail"
            violations.append(
                "dependency `rustls` should disable default features for deterministic profile control"
            )
            reason_code_set.add("rustls_default_features_not_disabled")
    else:
        checks["rustls_default_features_disabled"] = "fail"
        reason_code_set.add("cargo_manifest_dependencies_section_missing")

    readme_text = read_text(readme_path) if readme_path.exists() else ""
    adr_text = read_text(adr_path) if adr_path.exists() else ""
    ci_strategy_text = read_text(ci_strategy_path) if ci_strategy_path.exists() else ""

    for dep in EXPECTED_DEPS:
        key_prefix = dep.replace("-", "_")
        if dep in readme_text:
            checks[f"readme_mentions_{key_prefix}"] = "pass"
        else:
            checks[f"readme_mentions_{key_prefix}"] = "fail"
            violations.append(f"README must mention dependency `{dep}`")
            reason_code_set.add(f"readme_{key_prefix}_reference_missing")

    if "docs/architecture/adr-kamn-core-live-tls-transport.md" in readme_text:
        checks["readme_links_adr"] = "pass"
    else:
        checks["readme_links_adr"] = "fail"
        violations.append("README must link live TLS transport ADR")
        reason_code_set.add("readme_adr_link_missing")

    if "cargo check -p kamn-core --no-default-features" in readme_text:
        checks["readme_mentions_no_default_features"] = "pass"
    else:
        checks["readme_mentions_no_default_features"] = "fail"
        violations.append("README must document no-default-features local profile check")
        reason_code_set.add("readme_no_default_features_marker_missing")

    if "Keep these dependencies in `kamn-core` for live HTTPS transport:" in adr_text:
        checks["adr_dependency_section_present"] = "pass"
    else:
        checks["adr_dependency_section_present"] = "fail"
        violations.append("ADR must include accepted dependency posture section")
        reason_code_set.add("adr_dependency_section_missing")

    for dep in EXPECTED_DEPS:
        key_prefix = dep.replace("-", "_")
        if dep in adr_text:
            checks[f"adr_mentions_{key_prefix}"] = "pass"
        else:
            checks[f"adr_mentions_{key_prefix}"] = "fail"
            violations.append(f"ADR must mention dependency `{dep}`")
            reason_code_set.add(f"adr_{key_prefix}_reference_missing")

    if "cargo check -p kamn-core --features live-https" in ci_strategy_text:
        checks["ci_strategy_mentions_live_https_feature_check"] = "pass"
    else:
        checks["ci_strategy_mentions_live_https_feature_check"] = "fail"
        violations.append("CI strategy must mention live-https feature check command")
        reason_code_set.add("ci_strategy_live_https_feature_check_missing")

    if "cargo check -p kamn-core --no-default-features" in ci_strategy_text:
        checks["ci_strategy_mentions_no_default_features_check"] = "pass"
    else:
        checks["ci_strategy_mentions_no_default_features_check"] = "fail"
        violations.append("CI strategy must mention no-default-features check command")
        reason_code_set.add("ci_strategy_no_default_features_check_missing")

    status = "pass" if not violations else "fail"
    reason_codes = ["none"] if status == "pass" else sorted(reason_code_set)
    reason_codes_csv = "none" if status == "pass" else ",".join(reason_codes)
    reason_codes_value = reason_codes_csv
    reason_class = "stable" if status == "pass" else "violation"

    report = {
        "schema_version": SCHEMA_VERSION,
        "reason_taxonomy_version": REASON_TAXONOMY_VERSION,
        "status": status,
        "reason_class": reason_class,
        "reason_codes": reason_codes,
        "reason_codes_csv": reason_codes_csv,
        "reason_codes_value": reason_codes_value,
        "cargo_manifest": str(manifest_path),
        "readme": str(readme_path),
        "adr": str(adr_path),
        "ci_strategy": str(ci_strategy_path),
        "checks": checks,
        "violation_count": len(violations),
        "violations": violations,
    }

    if args.output_json:
        output_path = Path(args.output_json)
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    if status == "pass":
        print("status=ok")
        print(f"reason_taxonomy_version={REASON_TAXONOMY_VERSION}")
        print(f"reason_codes_csv={reason_codes_csv}")
        print(f"reason_codes_value={reason_codes_value}")
        print(f"reason_class={reason_class}")
        print("reason_codes=none")
        print("violation_count=0")
        return 0

    print("status=fail")
    print(f"reason_taxonomy_version={REASON_TAXONOMY_VERSION}")
    print(f"reason_codes_csv={reason_codes_csv}")
    print(f"reason_codes_value={reason_codes_value}")
    print(f"reason_class={reason_class}")
    print(f"reason_codes={','.join(reason_codes)}")
    print(f"violation_count={len(violations)}")
    for violation in violations:
        print(f"violation={violation}")
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
