import type { Report } from "./evidence";

export function canonicalObservationReceipts(report: Report) {
	const claim = findClaim(report, "three_agent_escrow_verification");
	if (!claim) return undefined;
	return {
		agent_a: observationReceipt(
			claim,
			"agent_a",
			"participant-private",
			"agent_a_observation_receipt_artifact",
			"agent_a_observation_receipt_digest",
		),
		agent_b: observationReceipt(
			claim,
			"agent_b",
			"participant-private",
			"agent_b_observation_receipt_artifact",
			"agent_b_observation_receipt_digest",
		),
		agent_c_verifier: observationReceipt(
			claim,
			"agent_c_verifier",
			"restricted-public",
			"agent_c_verifier_observation_receipt_artifact",
			"agent_c_verifier_observation_receipt_digest",
		),
	};
}

function observationReceipt(
	claim: Record<string, unknown>,
	agent: string,
	viewScope: string,
	artifactField: string,
	digestField: string,
) {
	return {
		agent,
		view_scope: viewScope,
		artifact: requireString(claim, artifactField),
		digest: requireString(claim, digestField),
	};
}

function findClaim(report: Report, id: string) {
	return (report.claim_matrix ?? []).find((entry) => entry.id === id);
}

function requireString(claim: Record<string, unknown>, field: string): string {
	const value = claim[field];
	if (typeof value === "string") return value;
	throw new Error(`KAMN report three-agent claim is missing string field: ${field}`);
}
