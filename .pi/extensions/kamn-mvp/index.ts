import { mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import type { ExtensionAPI, ExtensionContext, ExecResult } from "@earendil-works/pi-coding-agent";
import { Type } from "typebox";
const DEFAULT_REPORT = ".kamn/demo/latest/proof/report.json";
const DEFAULT_EVIDENCE = "/tmp/kamn-pi-mcp-agent-harness-evidence.json";
const PROOF_ENV = ["CARGO_TARGET_DIR=target/mvp-demo-proof", "CARGO_BUILD_JOBS=1", "CARGO_INCREMENTAL=0"];
type Report = {
	status?: string;
	devnet_mode?: string;
	claim_matrix?: Array<Record<string, unknown>>;
};
type ReportPath = {
	artifactPath: string;
	readPath: string;
};
export default function kamnMvpExtension(pi: ExtensionAPI) {
	registerVerifyTool(pi);
	registerInspectTool(pi);
	registerEvidenceTool(pi);
	registerDemoTool(pi);
}
function registerVerifyTool(pi: ExtensionAPI) {
	pi.registerTool({
		name: "kamn_verify_mvp_report",
		label: "KAMN Verify MVP",
		description: "Run KAMN verify-mvp-demo against a local proof report.",
		promptSnippet: "Verify a KAMN MVP proof report with the repo verifier",
		parameters: Type.Object({ reportPath: Type.Optional(Type.String()) }),
		async execute(_id, params, signal, _onUpdate, ctx) {
			const reportPath = safeReportPath(ctx, params.reportPath).artifactPath;
			const result = await proofExec(pi, ctx, verifyArgs(reportPath), signal);
			assertSuccess(result, "verify-mvp-demo");
			return textResult("KAMN MVP report verifier passed.", { reportPath });
		},
	});
}
function registerInspectTool(pi: ExtensionAPI) {
	pi.registerTool({
		name: "kamn_inspect_mvp_report_boundaries",
		label: "KAMN Inspect MVP",
		description: "Inspect MVP proof report claim boundaries without reading secret files.",
		promptSnippet: "Summarize KAMN MVP proof claim boundaries",
		parameters: Type.Object({ reportPath: Type.Optional(Type.String()) }),
		async execute(_id, params, _signal, _onUpdate, ctx) {
			const reportPath = safeReportPath(ctx, params.reportPath);
			const report = await readReport(reportPath.readPath);
			const summary = boundarySummary(report);
			return textResult(JSON.stringify(summary), summary);
		},
	});
}
function registerEvidenceTool(pi: ExtensionAPI) {
	pi.registerTool({
		name: "kamn_write_agent_harness_evidence",
		label: "KAMN Write Harness Evidence",
		description: "Write a Pi-tool generated KAMN agent-harness evidence artifact.",
		promptSnippet: "Write Pi extension evidence for KAMN MVP agent-harness verification",
		parameters: Type.Object({
			reportPath: Type.Optional(Type.String()),
			outputPath: Type.Optional(Type.String()),
		}),
		executionMode: "sequential",
		async execute(_id, params, _signal, _onUpdate, ctx) {
			const reportPath = safeReportPath(ctx, params.reportPath);
			const outputPath = safeOutputPath(ctx, params.outputPath);
			const report = await readReport(reportPath.readPath);
			await writeEvidence(outputPath, reportPath.artifactPath, report);
			return textResult(`Wrote KAMN Pi harness evidence to ${outputPath}`, { outputPath });
		},
	});
}
function registerDemoTool(pi: ExtensionAPI) {
	pi.registerTool({
		name: "kamn_run_demo_mvp_with_agent_evidence",
		label: "KAMN Demo MVP",
		description: "Run make demo-mvp with a Pi-generated agent-harness evidence artifact.",
		promptSnippet: "Run the KAMN MVP demo with Pi agent-harness evidence enabled",
		parameters: Type.Object({ evidencePath: Type.Optional(Type.String()) }),
		executionMode: "sequential",
		async execute(_id, params, signal, _onUpdate, ctx) {
			const evidencePath = safeOutputPath(ctx, params.evidencePath);
			const result = await proofExec(pi, ctx, demoArgs(evidencePath), signal);
			assertSuccess(result, "make demo-mvp");
			return textResult("KAMN make demo-mvp passed with Pi harness evidence.", {
				evidencePath,
			});
		},
	});
}
function verifyArgs(reportPath: string): string[] {
	return ["cargo", "run", "-p", "kamn-e2e-harness", "--", "verify-mvp-demo", "--report", reportPath];
}
function demoArgs(evidencePath: string): string[] {
	return [`KAMN_MVP_AGENT_HARNESS_EVIDENCE=${evidencePath}`, "make", "demo-mvp"];
}
async function proofExec(
	pi: ExtensionAPI,
	ctx: ExtensionContext,
	args: string[],
	signal: AbortSignal | undefined,
): Promise<ExecResult> {
	return pi.exec("env", [...PROOF_ENV, ...args], {
		cwd: ctx.cwd,
		signal,
		timeout: 180000,
	});
}
function safeReportPath(ctx: ExtensionContext, input: string | undefined): ReportPath {
	const path = cleanPath(input ?? DEFAULT_REPORT);
	rejectSecretLikePath(path);
	return { artifactPath: path, readPath: resolve(ctx.cwd, path) };
}
function safeOutputPath(ctx: ExtensionContext, input: string | undefined): string {
	const path = cleanPath(input ?? DEFAULT_EVIDENCE);
	rejectSecretLikePath(path);
	return path.startsWith("/") ? path : resolve(ctx.cwd, path);
}
function cleanPath(path: string): string {
	return path.startsWith("@") ? path.slice(1) : path;
}
function rejectSecretLikePath(path: string) {
	for (const marker of [".kamn/devnet", "auth.json", ".env", "keypair", "id_rsa", "oauth"]) {
		if (path.toLowerCase().includes(marker)) {
			throw new Error(`Refusing secret-like path: ${marker}`);
		}
	}
}
async function readReport(reportPath: string): Promise<Report> {
	const raw = await readFile(reportPath, "utf8");
	const report = JSON.parse(raw) as Report;
	if (!Array.isArray(report.claim_matrix)) {
		throw new Error("KAMN report is missing claim_matrix");
	}
	return report;
}
async function writeEvidence(outputPath: string, reportPath: string, report: Report) {
	await mkdir(dirname(outputPath), { recursive: true });
	await writeFile(outputPath, JSON.stringify(evidence(reportPath, report)));
}
function evidence(reportPath: string, report: Report) {
	return {
		schema_version: "kamn.mvp.agent-harness-evidence.v1",
		harness: "mcp-agent",
		execution_surface: "pi-extension-tools",
		report_path: reportPath,
		verifier_status: report.status === "NO-GO" ? "NO-GO" : "PASS",
		participant_agents: ["agent_a", "agent_b", "agent_c_verifier"],
		tool_markers: agentToolMarkers(),
		claim_boundaries: claimBoundaries(report),
		three_agent_boundary: threeAgentBoundary(report),
	};
}
function agentToolMarkers() {
	return ["register", "create_task", "fund_escrow", "release_escrow", "verify_proof"];
}
function claimBoundaries(_report: Report) {
	return {
		settlement_claim_label: "devnet-backed",
		dry_run_counted_as_success: false,
		placeholder_counted_as_success: false,
		verifier_private_view_visible: false,
	};
}
function boundarySummary(report: Report) {
	const claims = report.claim_matrix ?? [];
	return {
		status: report.status,
		devnet_mode: report.devnet_mode,
		local_runtime: claimStatus(claims, "local_runtime_startup"),
		settlement: claimStatus(claims, "devnet_settlement_asset_movement"),
		three_agent: claimStatus(claims, "three_agent_escrow_verification"),
		production_readiness: claimStatus(claims, "production_readiness"),
	};
}

function threeAgentBoundary(report: Report) {
	const claim = findClaim(report, "three_agent_escrow_verification");
	if (!claim) {
		return {
			claim_status: "NOT_PRESENT",
			claim_label: "NOT_PRESENT",
			claim_present: false,
		};
	}
	return {
		claim_status: requireString(claim, "status"),
		claim_label: requireString(claim, "label"),
		claim_present: true,
		agent_a_private_field_count: requireNumber(claim, "agent_a_private_field_count"),
		agent_b_private_field_count: requireNumber(claim, "agent_b_private_field_count"),
		verifier_private_field_count: requireNumber(claim, "verifier_private_field_count"),
		private_payload_redacted: requireBoolean(claim, "private_payload_redacted"),
		verifier_private_view_digest_present: typeof claim.verifier_private_view_digest === "string",
	};
}

function claimStatus(claims: Array<Record<string, unknown>>, id: string) {
	const claim = findClaim({ claim_matrix: claims }, id);
	return claim ? { label: claim.label, status: claim.status } : { status: "NOT_PRESENT" };
}

function findClaim(report: Report, id: string) {
	return (report.claim_matrix ?? []).find((entry) => entry.id === id);
}

function requireString(claim: Record<string, unknown>, field: string): string {
	const value = claim[field];
	if (typeof value === "string") return value;
	throw new Error(`KAMN report three-agent claim is missing string field: ${field}`);
}

function requireNumber(claim: Record<string, unknown>, field: string): number {
	const value = claim[field];
	if (typeof value === "number" && Number.isFinite(value)) return value;
	throw new Error(`KAMN report three-agent claim is missing number field: ${field}`);
}

function requireBoolean(claim: Record<string, unknown>, field: string): boolean {
	const value = claim[field];
	if (typeof value === "boolean") return value;
	throw new Error(`KAMN report three-agent claim is missing boolean field: ${field}`);
}
function assertSuccess(result: ExecResult, label: string) {
	if (result.code === 0) return;
	throw new Error(`${label} failed with exit ${result.code}: ${trimOutput(result)}`);
}
function trimOutput(result: ExecResult): string {
	return `${result.stdout}\n${result.stderr}`.trim().slice(-4000);
}
function textResult(text: string, details: unknown) {
	return { content: [{ type: "text" as const, text }], details };
}
