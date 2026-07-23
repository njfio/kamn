export const ROLES = ["agent_a", "agent_b", "agent_c"] as const;
export type Role = typeof ROLES[number];
export type ServiceReceipt = {
	actor_did: string;
	tool: string;
	action: string;
	resource_id: string;
	resulting_state: string;
	service_receipt_id: string;
	service_receipt_digest: string;
};

const ROLE_CONTRACTS = {
	agent_a: [
		["create_task", "task:create", "task", "submitted"],
		["fund_escrow", "escrow:fund", "escrow", "funded"],
		["release_escrow", "escrow:release-authorize", "escrow", "release-authorized"],
		["release_escrow", "settlement:confirmed", "escrow", "confirmed"],
	],
	agent_b: [
		["accept_task", "task:accept", "task", "accepted"],
		["complete_task", "task:complete", "task", "completed"],
	],
	agent_c: [],
} as const;
const RECEIPT_FIELDS = new Set([
	"actor_did", "tool", "action", "resource_id", "resulting_state", "service_receipt_id", "service_receipt_digest",
]);

export function normalizeServiceReceipts(value: unknown): ServiceReceipt[] {
	if (!Array.isArray(value)) fail();
	return value.map((entry) => normalizeReceipt(entry));
}

export function validateRoleAuthority(
	role: Role,
	did: string,
	taskId: string,
	escrowId: string,
	receipts: ServiceReceipt[],
) {
	const expected = ROLE_CONTRACTS[role];
	if (receipts.length !== expected.length) fail();
	for (const [index, contract] of expected.entries()) {
		const receipt = receipts[index];
		const [tool, action, resourceType, state] = contract;
		const resource = resourceType === "task" ? taskId : escrowId;
		if (!receipt || receipt.actor_did !== did || receipt.tool !== tool || receipt.action !== action
			|| receipt.resource_id !== resource || receipt.resulting_state !== state) fail();
	}
	requireUnique(receipts.map((receipt) => receipt.service_receipt_id));
	requireUnique(receipts.map((receipt) => `${receipt.service_receipt_id}:${receipt.service_receipt_digest}`));
}

export function validateGlobalReceiptUniqueness(receipts: ServiceReceipt[]) {
	requireUnique(receipts.map((receipt) => receipt.service_receipt_id));
	requireUnique(receipts.map((receipt) => `${receipt.service_receipt_id}:${receipt.service_receipt_digest}`));
}

function normalizeReceipt(value: unknown): ServiceReceipt {
	if (!isObject(value) || Object.keys(value).some((field) => !RECEIPT_FIELDS.has(field))) fail();
	return {
		actor_did: text(value.actor_did),
		tool: text(value.tool),
		action: text(value.action),
		resource_id: text(value.resource_id),
		resulting_state: text(value.resulting_state),
		service_receipt_id: text(value.service_receipt_id),
		service_receipt_digest: shaDigest(value.service_receipt_digest),
	};
}
function requireUnique(values: string[]) {
	if (new Set(values).size !== values.length) fail();
}
function text(value: unknown): string {
	if (typeof value === "string" && value.trim()) return value;
	fail();
}
function shaDigest(value: unknown): string {
	const parsed = text(value);
	if (/^sha256:[0-9a-f]{64}$/.test(parsed)) return parsed;
	fail();
}
function isObject(value: unknown): value is Record<string, unknown> {
	return typeof value === "object" && value !== null && !Array.isArray(value);
}
function fail(): never {
	throw new Error("PI_SERVICE_AUTHORITY_MISMATCH");
}
