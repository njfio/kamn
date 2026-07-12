export type Role = "agent_a" | "agent_b" | "agent_c";
export type RuntimeReceipt = { request_id: number; tool: string; outcome: "success" | "error"; digest: string };

export function normalizeRuntimeReceipts(value: unknown): RuntimeReceipt[] {
	if (!Array.isArray(value) || value.length === 0) throw mismatch();
	return value.map((entry) => normalizeReceipt(entry));
}

export function validateRuntimeReceipts(
	role: Role,
	firstRequestId: number,
	digests: string[],
	receipts: RuntimeReceipt[],
) {
	if (receipts.length !== digests.length) throw mismatch();
	receipts.forEach((receipt, index) => {
		if (receipt.request_id !== firstRequestId + index || receipt.digest !== digests[index]) throw mismatch();
	});
	for (const tool of requiredTools(role)) {
		if (receipts.filter((receipt) => receipt.tool === tool && receipt.outcome === "success").length !== 1) throw mismatch();
	}
}

function normalizeReceipt(value: unknown): RuntimeReceipt {
	if (!isRecord(value)) throw mismatch();
	const outcome = value.outcome;
	if (outcome !== "success" && outcome !== "error") throw mismatch();
	return {
		request_id: positiveInteger(value.request_id),
		tool: requiredString(value.tool),
		outcome,
		digest: shaDigest(value.digest),
	};
}

function requiredTools(role: Role): string[] {
	if (role === "agent_a") return ["register", "create_task", "fund_escrow", "release_escrow", "query_participant_task_projection"];
	if (role === "agent_b") return ["register", "accept_task", "complete_task", "query_participant_task_projection"];
	return ["register", "query_verifier_task_projection"];
}
function positiveInteger(value: unknown): number {
	if (typeof value === "number" && Number.isSafeInteger(value) && value > 0) return value;
	throw mismatch();
}
function requiredString(value: unknown): string {
	if (typeof value === "string" && value.trim()) return value;
	throw mismatch();
}
function shaDigest(value: unknown): string {
	const parsed = requiredString(value);
	if (/^sha256:[0-9a-f]{64}$/.test(parsed)) return parsed;
	throw mismatch();
}
function isRecord(value: unknown): value is Record<string, unknown> {
	return typeof value === "object" && value !== null && !Array.isArray(value);
}
function mismatch(): Error {
	return new Error("PI_RUNTIME_RECEIPT_MISMATCH");
}
