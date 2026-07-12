import { createHash } from "node:crypto";

type JsonObject = Record<string, unknown>;
export type McpResponseReceipt = {
	request_id: number;
	tool: string;
	outcome: "success" | "error";
	digest: string;
};
export type McpSessionProvenance = {
	child_process_id: number;
	first_request_id: number;
	last_request_id: number;
	runtime_response_digests: string[];
	runtime_response_receipts: McpResponseReceipt[];
};

export class McpProvenanceTracker {
	private readonly receipts: McpResponseReceipt[] = [];
	record(id: string, tool: string, outcome: McpResponseReceipt["outcome"], payload: JsonObject) {
		const requestId = Number(id);
		if (!Number.isSafeInteger(requestId) || requestId <= 0) throw new Error("KAMN live MCP request ID is invalid");
		const digest = createHash("sha256").update(JSON.stringify(payload)).digest("hex");
		this.receipts.push({ request_id: requestId, tool, outcome, digest: `sha256:${digest}` });
	}
	provenance(childProcessId: number): McpSessionProvenance {
		const first = this.receipts[0];
		const last = this.receipts.at(-1);
		if (!first || !last) throw new Error("KAMN live MCP session has no successful runtime provenance");
		return {
			child_process_id: childProcessId,
			first_request_id: first.request_id,
			last_request_id: last.request_id,
			runtime_response_digests: this.receipts.map(({ digest }) => digest),
			runtime_response_receipts: this.receipts.map((receipt) => ({ ...receipt })),
		};
	}
}
