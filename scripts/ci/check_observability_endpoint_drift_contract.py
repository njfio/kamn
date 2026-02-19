#!/usr/bin/env python3
"""Fail-closed drift checker for async observability endpoint marker and docs parity."""

from __future__ import annotations

import argparse
from pathlib import Path
import sys
import time

SCRIPT_DIR = Path(__file__).resolve().parent
ROOT_DIR = SCRIPT_DIR.parent.parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_framework import (  # noqa: E402
    ContractError,
    DecisionAccumulator,
    fail,
    write_json,
)

SCHEMA_VERSION = "kamn.ci.observability-endpoint-drift-report.v1"
DOCS_MIGRATION_MARKER = (
    "Runtime observability endpoint ingress runs on async tokio listener "
    "path; drift contracts enforce fail-closed parity for unknown-path, "
    "malformed-request, and timeout compatibility behavior."
)

SOURCE_MARKERS: dict[str, str] = {
    "axum_import": "use axum::{",
    "async_listener_bind": "let listener = tokio::net::TcpListener::bind(config.bind_addr.as_str())",
    "async_route_handler": "async fn handle_observability_http_route(",
    "async_dispatch": "async fn dispatch_observability_endpoint_request(",
    "async_not_found_handler": "handle_observability_not_found_path().await",
    "request_budget_record": "state.request_budget.record_request();",
    "async_idle_timeout_reason": "observability endpoint timed out after {} ms waiting for requests",
}

TELEMETRY_SOURCE_MARKERS: dict[str, str] = {
    "stream_schema_marker": "const OBSERVABILITY_STREAM_SCHEMA_VERSION: &str = \"kamn.runtime.observability.stream.v1\";",
    "health_schema_marker": "const OBSERVABILITY_HEALTH_SCHEMA_VERSION: &str = \"kamn.runtime.observability.health.v1\";",
    "readiness_schema_marker": "const OBSERVABILITY_READINESS_SCHEMA_VERSION: &str = \"kamn.runtime.observability.readiness.v1\";",
    "readiness_reason_taxonomy_version_marker": "const OBSERVABILITY_READINESS_REASON_TAXONOMY_VERSION: &str =",
    "metrics_readiness_reason_code_marker": "kamn_observability_readiness_reason_code{{readiness_reason_code=\\\"{}\\\"}} 1",
    "readiness_reason_projection_fn": "fn readiness_reason_code(snapshot: &RuntimeObservabilitySnapshot) -> &'static str {",
}

TELEMETRY_DOC_MARKERS: dict[str, str] = {
    "stream_schema_marker": "schema_version=\"kamn.runtime.observability.stream.v1\"",
    "health_schema_marker": "`schema_version` (`kamn.runtime.observability.health.v1`)",
    "readiness_schema_marker": "`schema_version` (`kamn.runtime.observability.readiness.v1`)",
    "readiness_reason_taxonomy_version_marker": "`readiness_reason_taxonomy_version` (`kamn.runtime.observability.readiness.reason-taxonomy.v1`)",
    "readiness_reason_taxonomy_marker": "`readiness_reason_code` (dependency-derived readiness taxonomy)",
    "readiness_transport_marker": "`readiness_transport_dependency_unhealthy`",
    "readiness_signer_marker": "`readiness_signer_dependency_unhealthy`",
    "readiness_commit_marker": "`readiness_commit_dependency_unhealthy`",
    "readiness_runtime_health_marker": "`readiness_runtime_health_degraded`",
}

OBSERVABILITY_CONTRACT_DOC_MARKERS: dict[str, str] = {
    "health_schema_marker": "schema_version=\"kamn.runtime.observability.health.v1\"",
    "readiness_schema_marker": "schema_version=\"kamn.runtime.observability.readiness.v1\"",
    "stream_schema_marker": "schema_version=\"kamn.runtime.observability.stream.v1\"",
    "readiness_reason_taxonomy_version_marker": "readiness_reason_taxonomy_version=\"kamn.runtime.observability.readiness.reason-taxonomy.v1\"",
}

TELEMETRY_STRATEGY_MARKER = (
    "telemetry schema docs-contract marker set remains fail-closed for "
    "health/readiness/stream schema_version markers and readiness_reason_code taxonomy."
)

MAIN_WIRING_MARKER = "serve_observability_endpoint(&endpoint_config, &snapshot)"
FRAMEWORK_INGRESS_MARKER = "tokio::net::TcpListener::bind(config.bind_addr.as_str())"

DEFAULT_SOURCE_FILES = [
    ROOT_DIR / "crates/kamn-node/src/observability_endpoint.rs",
    ROOT_DIR / "crates/kamn-node/src/observability_endpoint/endpoint_server.rs",
    ROOT_DIR / "crates/kamn-node/src/observability_endpoint/payload_contract.rs",
    ROOT_DIR / "crates/kamn-node/src/observability_endpoint/payload_render.rs",
    ROOT_DIR / "crates/kamn-node/src/observability_endpoint/tls_mode.rs",
]


def _read_text(path: Path, *, reason_code: str) -> str:
    if not path.is_file():
        fail(reason_code)
    return path.read_text(encoding="utf-8")


def _run(args: argparse.Namespace) -> int:
    if args.source_file:
        source_files = [Path(args.source_file).resolve()]
    else:
        source_files = [path.resolve() for path in DEFAULT_SOURCE_FILES]

    main_file = Path(args.main_file).resolve()
    framework_file = Path(args.framework_file).resolve()
    docs_file = Path(args.docs_file).resolve()
    observability_doc_file = Path(args.observability_doc_file).resolve()
    observability_contract_doc_file = Path(args.observability_contract_doc_file).resolve()
    strategy_doc_file = Path(args.strategy_doc_file).resolve()

    source_texts = [
        _read_text(path, reason_code="observability_source_file_missing")
        for path in source_files
    ]
    source_text = "\n".join(source_texts)
    source_contract_text = "\n".join(
        text.split("\n#[cfg(test)]", 1)[0] for text in source_texts
    )
    main_text = _read_text(
        main_file,
        reason_code="observability_main_file_missing",
    )
    framework_text = _read_text(
        framework_file,
        reason_code="observability_framework_file_missing",
    )
    docs_text = _read_text(
        docs_file,
        reason_code="observability_docs_file_missing",
    )
    observability_docs_text = _read_text(
        observability_doc_file,
        reason_code="observability_schema_docs_file_missing",
    )
    observability_contract_docs_text = _read_text(
        observability_contract_doc_file,
        reason_code="observability_contract_docs_file_missing",
    )
    strategy_doc_text = _read_text(
        strategy_doc_file,
        reason_code="observability_strategy_doc_file_missing",
    )

    decision = DecisionAccumulator()

    missing_source_markers: list[str] = []
    for marker_key, marker_value in SOURCE_MARKERS.items():
        if marker_value not in source_contract_text:
            missing_source_markers.append(marker_key)
            decision.reject_if(
                True,
                f"observability_source_marker_missing:{marker_key}",
            )

    missing_telemetry_source_markers: list[str] = []
    for marker_key, marker_value in TELEMETRY_SOURCE_MARKERS.items():
        if marker_value not in source_contract_text:
            missing_telemetry_source_markers.append(marker_key)
            decision.reject_if(
                True,
                f"observability_source_marker_missing:{marker_key}",
            )

    missing_telemetry_doc_markers: list[str] = []
    for marker_key, marker_value in TELEMETRY_DOC_MARKERS.items():
        if marker_value not in observability_docs_text:
            missing_telemetry_doc_markers.append(marker_key)
            decision.reject_if(
                True,
                f"observability_schema_docs_marker_missing:{marker_key}",
            )

    missing_contract_doc_markers: list[str] = []
    for marker_key, marker_value in OBSERVABILITY_CONTRACT_DOC_MARKERS.items():
        if marker_value not in observability_contract_docs_text:
            missing_contract_doc_markers.append(marker_key)
            decision.reject_if(
                True,
                f"observability_contract_docs_marker_missing:{marker_key}",
            )

    decision.reject_if(
        MAIN_WIRING_MARKER not in main_text,
        "observability_main_wiring_marker_missing",
    )
    decision.reject_if(
        FRAMEWORK_INGRESS_MARKER not in framework_text,
        "observability_framework_marker_missing",
    )
    decision.reject_if(
        DOCS_MIGRATION_MARKER not in docs_text,
        "observability_docs_marker_missing",
    )
    decision.reject_if(
        TELEMETRY_STRATEGY_MARKER not in strategy_doc_text,
        "observability_strategy_marker_missing:telemetry_schema_contract_marker",
    )

    final_decision, reason_codes = decision.finalize("none")
    status = "pass" if final_decision == "GO" else "fail"

    report = {
        "schema_version": SCHEMA_VERSION,
        "status": status,
        "final_decision": final_decision,
        "observability_async_ingress_contract_status": (
            "verified" if not missing_source_markers else "rejected"
        ),
        "observability_framework_parity_status": (
            "verified"
            if MAIN_WIRING_MARKER in main_text and FRAMEWORK_INGRESS_MARKER in framework_text
            else "rejected"
        ),
        "docs_migration_contract_status": (
            "verified" if DOCS_MIGRATION_MARKER in docs_text else "rejected"
        ),
        "telemetry_schema_contract_status": (
            "verified"
            if (
                not missing_telemetry_source_markers
                and not missing_telemetry_doc_markers
                and not missing_contract_doc_markers
                and TELEMETRY_STRATEGY_MARKER in strategy_doc_text
            )
            else "rejected"
        ),
        "source_marker_count": len(SOURCE_MARKERS) - len(missing_source_markers),
        "expected_source_marker_count": len(SOURCE_MARKERS),
        "telemetry_source_marker_count": (
            len(TELEMETRY_SOURCE_MARKERS) - len(missing_telemetry_source_markers)
        ),
        "expected_telemetry_source_marker_count": len(TELEMETRY_SOURCE_MARKERS),
        "telemetry_doc_marker_count": (
            len(TELEMETRY_DOC_MARKERS) - len(missing_telemetry_doc_markers)
        ),
        "expected_telemetry_doc_marker_count": len(TELEMETRY_DOC_MARKERS),
        "contract_doc_marker_count": (
            len(OBSERVABILITY_CONTRACT_DOC_MARKERS) - len(missing_contract_doc_markers)
        ),
        "expected_contract_doc_marker_count": len(OBSERVABILITY_CONTRACT_DOC_MARKERS),
        "reason_codes": reason_codes,
        "source_file": str(source_files[0]),
        "source_files": [str(path) for path in source_files],
        "main_file": str(main_file),
        "framework_file": str(framework_file),
        "docs_file": str(docs_file),
        "observability_doc_file": str(observability_doc_file),
        "observability_contract_doc_file": str(observability_contract_doc_file),
        "strategy_doc_file": str(strategy_doc_file),
        "generated_at_epoch": int(time.time()),
    }

    output_json = None
    if args.output_json:
        output_json = Path(args.output_json).resolve()
        write_json(output_json, report)

    reason_codes_csv = ",".join(reason_codes)
    print(f"status={status}")
    print(f"final_decision={final_decision}")
    print(
        "observability_async_ingress_contract_status="
        f"{report['observability_async_ingress_contract_status']}"
    )
    print(
        "observability_framework_parity_status="
        f"{report['observability_framework_parity_status']}"
    )
    print(f"docs_migration_contract_status={report['docs_migration_contract_status']}")
    print(f"telemetry_schema_contract_status={report['telemetry_schema_contract_status']}")
    print(f"reason_codes={reason_codes_csv}")
    if output_json is not None:
        print(f"report_file={output_json}")

    if final_decision != "GO":
        fail(f"observability endpoint drift contract failed: {reason_codes_csv}")

    return 0


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Check fail-closed observability endpoint drift contract markers."
    )
    parser.add_argument(
        "--source-file",
        default="",
        help=(
            "Optional single-source override path for targeted drift tests. "
            "When omitted, checker reads the observability endpoint source bundle."
        ),
    )
    parser.add_argument(
        "--main-file",
        default=str(ROOT_DIR / "crates/kamn-node/src/main.rs"),
        help="Main runtime wiring source path.",
    )
    parser.add_argument(
        "--framework-file",
        default=str(ROOT_DIR / "crates/kamn-node/src/service_api_endpoint/server.rs"),
        help="Framework ingress source path for async marker checks.",
    )
    parser.add_argument(
        "--docs-file",
        default=str(ROOT_DIR / "docs/foundation/node-runtime-cli.md"),
        help="Node runtime CLI documentation path.",
    )
    parser.add_argument(
        "--observability-doc-file",
        default=str(ROOT_DIR / "docs/foundation/observability-slo-dashboards.md"),
        help="Observability telemetry schema documentation path.",
    )
    parser.add_argument(
        "--observability-contract-doc-file",
        default=str(ROOT_DIR / "docs/observability/contracts.md"),
        help="Observability runtime contracts documentation path.",
    )
    parser.add_argument(
        "--strategy-doc-file",
        default=str(ROOT_DIR / "docs/ci/strategy.md"),
        help="CI strategy documentation path for telemetry schema contract markers.",
    )
    parser.add_argument(
        "--output-json",
        default="",
        help="Optional output path for drift report JSON.",
    )

    args = parser.parse_args()
    return _run(args)


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except ContractError as error:
        print(error, file=sys.stderr)
        raise SystemExit(1)
