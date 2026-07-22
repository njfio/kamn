import { createHash } from "node:crypto";
import type { ServiceAuthorityReceipt } from "./mcp-authority.ts";

type JsonObject = Record<string, unknown>;
const PUBLIC_RESULT_FIELDS = new Set([
	"did", "task_id", "state", "transaction_id", "escrow_id", "amount_lamports", "network",
	"settlement_tx_signature", "settlement_commitment", "public_commitment", "view_scope", "participant_role",
]);
export type McpResponseReceipt = {
	request_id: number;
	tool: string;
	outcome: "success" | "error";
	digest: string;
	public_result: JsonObject;
};
export type McpSessionProvenance = {
	child_process_id: number;
	first_request_id: number;
	last_request_id: number;
	transport_response_digests: string[];
	transport_response_receipts: McpResponseReceipt[];
	service_profile_commitment: string;
	service_authority_receipts: ServiceAuthorityReceipt[];
};

export class McpProvenanceTracker {
	private readonly receipts: McpResponseReceipt[] = [];
	private profileCommitment?: string;
	private readonly authorityReceipts: ServiceAuthorityReceipt[] = [];
	record(id: string, tool: string, outcome: McpResponseReceipt["outcome"], payload: JsonObject) {
		const requestId = Number(id);
		if (!Number.isSafeInteger(requestId) || requestId <= 0) throw new Error("KAMN live MCP request ID is invalid");
		const digest = createHash("sha256").update(JSON.stringify(payload)).digest("hex");
		this.receipts.push({
			request_id: requestId, tool, outcome, digest: `sha256:${digest}`,
			public_result: outcome === "success" ? publicResult(payload) : {},
		});
	}
	recordProfileCommitment(commitment: string) {
		this.profileCommitment = commitment;
	}
	recordAuthorityReceipt(receipt: ServiceAuthorityReceipt) {
		this.authorityReceipts.push({ ...receipt });
	}
	provenance(childProcessId: number): McpSessionProvenance {
		const first = this.receipts[0];
		const last = this.receipts.at(-1);
		if (!first || !last || !this.profileCommitment) throw new Error("KAMN live MCP session has no successful authority provenance");
		return {
			child_process_id: childProcessId,
			first_request_id: first.request_id,
			last_request_id: last.request_id,
			transport_response_digests: this.receipts.map(({ digest }) => digest),
			transport_response_receipts: this.receipts.map((receipt) => ({ ...receipt })),
			service_profile_commitment: this.profileCommitment,
			service_authority_receipts: this.authorityReceipts.map((receipt) => ({ ...receipt })),
		};
	}
}

function publicResult(payload: JsonObject): JsonObject {
	return Object.fromEntries(Object.entries(payload).filter(([field, value]) =>
		PUBLIC_RESULT_FIELDS.has(field) && ["string", "number", "boolean"].includes(typeof value),
	));
}
