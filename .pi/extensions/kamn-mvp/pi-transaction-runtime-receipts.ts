export type Role = "agent_a" | "agent_b" | "agent_c";
type PublicResult = Record<string, string | number>;
const PUBLIC_STRING_FIELDS = new Set([
	"did", "task_id", "state", "transaction_id", "escrow_id", "network", "settlement_tx_signature",
	"settlement_commitment", "public_commitment", "view_scope", "participant_role",
]);
export type RuntimeReceipt = {
	request_id: number; tool: string; outcome: "success" | "error"; digest: string; public_result: PublicResult;
};

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
	for (const tool of requiredMutationTools(role)) {
		if (receipts.filter((receipt) => receipt.tool === tool && receipt.outcome === "success").length !== 1) throw mismatch();
	}
	if (!receipts.some((receipt) => receipt.tool === projectionTool(role) && receipt.outcome === "success")) throw mismatch();
}

export function validateFinalProjectionReceipt(role: Role, digest: string, receipts: RuntimeReceipt[]) {
	const final = receipts.at(-1);
	if (!final || final.tool !== projectionTool(role) || final.outcome !== "success" || final.digest !== digest) throw mismatch();
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
		public_result: normalizePublicResult(value.public_result, outcome),
	};
}

function normalizePublicResult(value: unknown, outcome: RuntimeReceipt["outcome"]): PublicResult {
	if (!isRecord(value)) throw mismatch();
	if (outcome === "error") {
		if (Object.keys(value).length !== 0) throw mismatch();
		return {};
	}
	return Object.fromEntries(Object.entries(value).map(([field, entry]) => [field, publicValue(field, entry)]));
}

function publicValue(field: string, value: unknown): string | number {
	if (field === "amount_lamports" && typeof value === "number" && Number.isSafeInteger(value) && value > 0) return value;
	if (PUBLIC_STRING_FIELDS.has(field) && typeof value === "string" && value.trim()) return value;
	throw mismatch();
}

function requiredMutationTools(role: Role): string[] {
	if (role === "agent_a") return ["register", "create_task", "fund_escrow", "release_escrow"];
	if (role === "agent_b") return ["register", "accept_task", "complete_task"];
	return ["register"];
}
function projectionTool(role: Role): string {
	return role === "agent_c" ? "query_verifier_task_projection" : "query_participant_task_projection";
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
