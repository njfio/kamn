import { resolve } from "node:path";
import type { ExtensionAPI, ExtensionContext, ExecResult } from "@earendil-works/pi-coding-agent";
import { Type } from "typebox";
import { actorToolSpecs, recordActorReceipt } from "./actor-receipts";
import { boundarySummary, readReport, writeEvidence } from "./evidence";

const DEFAULT_REPORT = ".kamn/demo/latest/proof/report.json";
const DEFAULT_EVIDENCE = "/tmp/kamn-pi-mcp-agent-harness-evidence.json";
const PROOF_ENV = ["CARGO_TARGET_DIR=target/mvp-demo-proof", "CARGO_BUILD_JOBS=1", "CARGO_INCREMENTAL=0"];
type ReportPath = {
	artifactPath: string;
	readPath: string;
};
export default function kamnMvpExtension(pi: ExtensionAPI) {
	registerVerifyTool(pi);
	registerInspectTool(pi);
	registerActorReceiptTools(pi);
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
function registerActorReceiptTools(pi: ExtensionAPI) {
	for (const spec of actorToolSpecs()) {
		registerActorReceiptTool(pi, spec.tool);
	}
}
function registerActorReceiptTool(pi: ExtensionAPI, tool: string) {
	pi.registerTool({
		name: tool,
		label: tool.replaceAll("_", " "),
		description: "Record one KAMN three-agent actor receipt for Pi harness evidence.",
		promptSnippet: `Record KAMN actor receipt with ${tool}`,
		parameters: Type.Object({ reportPath: Type.Optional(Type.String()) }),
		executionMode: "sequential",
		async execute(_id, params, _signal, _onUpdate, ctx) {
			const reportPath = safeReportPath(ctx, params.reportPath);
			const report = await readReport(reportPath.readPath);
			const receipt = recordActorReceipt(reportPath.artifactPath, report, tool);
			return textResult(`Recorded KAMN actor receipt ${tool}.`, receipt);
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
