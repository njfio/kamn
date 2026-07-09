import type { Report } from "./evidence";

const RECEIPTS: ActorReceipt[] = [];
const TOOLS: ActorToolSpec[] = [
	{
		tool: "kamn_agent_a_register",
		agent: "agent_a",
		action: "register",
		scopeField: "agent_a_view_scope",
		artifactField: "agent_a_view_artifact",
		digestField: "agent_a_view_digest",
	},
	{
		tool: "kamn_agent_a_invoke_transaction",
		agent: "agent_a",
		action: "invoke_transaction",
		scopeField: "agent_a_view_scope",
		artifactField: "agent_a_view_artifact",
		digestField: "agent_a_view_digest",
	},
	{
		tool: "kamn_agent_b_register",
		agent: "agent_b",
		action: "register",
		scopeField: "agent_b_view_scope",
		artifactField: "agent_b_view_artifact",
		digestField: "agent_b_view_digest",
	},
	{
		tool: "kamn_agent_b_accept_task",
		agent: "agent_b",
		action: "accept_task",
		scopeField: "agent_b_view_scope",
		artifactField: "agent_b_view_artifact",
		digestField: "agent_b_view_digest",
	},
	{
		tool: "kamn_agent_c_verify_three_agent_proof",
		agent: "agent_c_verifier",
		action: "verify_proof",
		scopeField: "verifier_view_scope",
		artifactField: "agent_c_verifier_view_artifact",
		digestField: "agent_c_verifier_view_digest",
	},
];

export type ActorReceipt = {
	sequence: number;
	tool: string;
	agent: string;
	action: string;
	outcome: "PASS";
	report_path: string;
	view_scope: string;
	view_artifact: string;
	view_digest: string;
};

export function actorToolSpecs() {
	return TOOLS;
}

export function recordActorReceipt(reportPath: string, report: Report, tool: string) {
	const spec = requireToolSpec(tool);
	const claim = requireThreeAgentClaim(report);
	const sequence = receiptsForPath(reportPath).length + 1;
	const receipt = actorReceipt(reportPath, claim, spec, sequence);
	RECEIPTS.push(receipt);
	return receipt;
}

export function requiredActorReceipts(reportPath: string, report: Report) {
	if (!findClaim(report, "three_agent_escrow_verification")) return undefined;
	const receipts = receiptsForPath(reportPath);
	const missing = TOOLS.filter((tool) => !receipts.some((receipt) => receipt.tool === tool.tool));
	if (missing.length > 0) {
		throw new Error(`Missing KAMN actor tool receipts: ${missing.map((tool) => tool.tool).join(",")}`);
	}
	return receipts;
}

function actorReceipt(
	reportPath: string,
	claim: Record<string, unknown>,
	spec: ActorToolSpec,
	sequence: number,
): ActorReceipt {
	return {
		sequence,
		tool: spec.tool,
		agent: spec.agent,
		action: spec.action,
		outcome: "PASS",
		report_path: reportPath,
		view_scope: requireString(claim, spec.scopeField),
		view_artifact: requireString(claim, spec.artifactField),
		view_digest: requireString(claim, spec.digestField),
	};
}

function receiptsForPath(reportPath: string) {
	return RECEIPTS.filter((receipt) => receipt.report_path === reportPath);
}

function requireToolSpec(tool: string) {
	const spec = TOOLS.find((candidate) => candidate.tool === tool);
	if (!spec) throw new Error(`Unknown KAMN actor tool: ${tool}`);
	return spec;
}

function requireThreeAgentClaim(report: Report) {
	const claim = findClaim(report, "three_agent_escrow_verification");
	if (!claim) {
		throw new Error("KAMN report is missing three_agent_escrow_verification");
	}
	return claim;
}

function findClaim(report: Report, id: string) {
	return (report.claim_matrix ?? []).find((entry) => entry.id === id);
}

function requireString(claim: Record<string, unknown>, field: string): string {
	const value = claim[field];
	if (typeof value === "string") return value;
	throw new Error(`KAMN report three-agent claim is missing string field: ${field}`);
}

type ActorToolSpec = {
	tool: string;
	agent: string;
	action: string;
	scopeField: string;
	artifactField: string;
	digestField: string;
};
