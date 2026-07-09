import { mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname } from "node:path";
import { requiredActorReceipts } from "./actor-receipts";
import { canonicalObservationReceipts } from "./observation-receipts";

export type Report = {
	status?: string;
	devnet_mode?: string;
	claim_matrix?: Array<Record<string, unknown>>;
};

const ACTORS: Record<string, ActorInput> = {
	agent_a: {
		agent: "agent_a",
		actions: ["register", "invoke_transaction"],
		scopeField: "agent_a_view_scope",
		artifactField: "agent_a_view_artifact",
		digestField: "agent_a_view_digest",
	},
	agent_b: {
		agent: "agent_b",
		actions: ["register", "accept_task"],
		scopeField: "agent_b_view_scope",
		artifactField: "agent_b_view_artifact",
		digestField: "agent_b_view_digest",
	},
	agent_c_verifier: {
		agent: "agent_c_verifier",
		actions: ["verify_proof"],
		scopeField: "verifier_view_scope",
		artifactField: "agent_c_verifier_view_artifact",
		digestField: "agent_c_verifier_view_digest",
	},
};

export async function readReport(reportPath: string): Promise<Report> {
	const raw = await readFile(reportPath, "utf8");
	const report = JSON.parse(raw) as Report;
	if (!Array.isArray(report.claim_matrix)) {
		throw new Error("KAMN report is missing claim_matrix");
	}
	return report;
}

export async function writeEvidence(outputPath: string, reportPath: string, report: Report) {
	await mkdir(dirname(outputPath), { recursive: true });
	await writeFile(outputPath, JSON.stringify(evidence(reportPath, report)));
}

export function boundarySummary(report: Report) {
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

function evidence(reportPath: string, report: Report) {
	const actorRehearsal = threeAgentActorRehearsal(report);
	const actorReceipts = requiredActorReceipts(reportPath, report);
	const observationReceipts = canonicalObservationReceipts(report);
	return {
		schema_version: "kamn.mvp.agent-harness-evidence.v1",
		harness: "mcp-agent",
		execution_surface: "pi-extension-tools",
		report_path: reportPath,
		verifier_status: report.status === "NO-GO" ? "NO-GO" : "PASS",
		participant_agents: ["agent_a", "agent_b", "agent_c_verifier"],
		tool_markers: agentToolMarkers(),
		claim_boundaries: claimBoundaries(),
		three_agent_boundary: threeAgentBoundary(report),
		...(actorRehearsal ? { three_agent_actor_rehearsal: actorRehearsal } : {}),
		...(actorReceipts ? { three_agent_actor_tool_receipts: actorReceipts } : {}),
		...(observationReceipts
			? { three_agent_actor_observation_receipts: observationReceipts }
			: {}),
	};
}

function agentToolMarkers() {
	return ["register", "create_task", "fund_escrow", "release_escrow", "verify_proof"];
}

function claimBoundaries() {
	return {
		settlement_claim_label: "devnet-backed",
		dry_run_counted_as_success: false,
		placeholder_counted_as_success: false,
		verifier_private_view_visible: false,
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

function threeAgentActorRehearsal(report: Report) {
	const claim = findClaim(report, "three_agent_escrow_verification");
	if (!claim) return undefined;
	return {
		settlement_claim_label: requireString(claim, "label"),
		settlement_status: requireString(claim, "status"),
		private_payload_redacted: requireBoolean(claim, "private_payload_redacted"),
		...actorObservations(claim),
	};
}

function actorObservations(claim: Record<string, unknown>) {
	return {
		agent_a: actorObservation(claim, ACTORS.agent_a),
		agent_b: actorObservation(claim, ACTORS.agent_b),
		agent_c_verifier: actorObservation(claim, ACTORS.agent_c_verifier),
	};
}

function actorObservation(claim: Record<string, unknown>, input: ActorInput) {
	return {
		agent: input.agent,
		actions: input.actions,
		view_scope: requireString(claim, input.scopeField),
		view_artifact: requireString(claim, input.artifactField),
		[input.digestField]: requireString(claim, input.digestField),
	};
}

type ActorInput = {
	agent: string;
	actions: string[];
	scopeField: string;
	artifactField: string;
	digestField: string;
};

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
