type JsonObject = Record<string, unknown>;

const SCHEMA = "kamn.mcp.authority-receipt.v1";
const SOURCE = "kamn-service";
const MUTATIONS = {
	create_task: { action: "task:create", resource: "task_id", states: ["submitted"] },
	accept_task: { action: "task:accept", resource: "task_id", states: ["accepted"] },
	complete_task: { action: "task:complete", resource: "task_id", states: ["completed"] },
	fund_escrow: { action: "escrow:fund", resource: "escrow_id", states: ["funded"] },
	release_escrow: { action: "escrow:release-authorize", resource: "escrow_id", states: ["release-authorized", "released"] },
} as const;

export type ServiceAuthorityReceipt = {
	actor_did: string;
	tool: string;
	action: string;
	resource_id: string;
	resulting_state: string;
	service_receipt_id: string;
	service_receipt_digest: string;
};
export type ValidatedAuthority = {
	serviceResult: JsonObject;
	profileCommitment?: string;
	receipt?: ServiceAuthorityReceipt;
};

export function requiresAuthority(tool: string): boolean {
	return tool === "register" || tool in MUTATIONS;
}

export function validateAuthority(tool: string, result: JsonObject, expectedActor?: string): ValidatedAuthority {
	if (!requiresAuthority(tool)) return { serviceResult: result };
	validateEnvelopeHeader(result);
	return tool === "register"
		? validateRegistration(result)
		: validateMutation(tool as keyof typeof MUTATIONS, result, expectedActor);
}

function validateEnvelopeHeader(result: JsonObject) {
	if (result.schema_version === undefined || result.authority_kind === undefined || result.source === undefined
		|| result.service_result === undefined) {
		throw new Error("MCP_AUTHORITY_RECEIPT_MISSING");
	}
	if (result.schema_version !== SCHEMA || result.source !== SOURCE || !isObject(result.service_result)) {
		throw new Error("MCP_AUTHORITY_RECEIPT_INVALID");
	}
}

function validateRegistration(result: JsonObject): ValidatedAuthority {
	const serviceResult = objectField(result, "service_result");
	const actor = stringField(result, "actor_did");
	const commitment = digestField(result, "profile_commitment");
	if (result.authority_kind !== "service-profile-commitment" || result.tool !== "register"
		|| result.resource_id !== actor || serviceResult.did !== actor || serviceResult.profile_commitment !== commitment) {
		throw new Error("MCP_AUTHORITY_RECEIPT_INVALID");
	}
	return { serviceResult, profileCommitment: commitment };
}

function validateMutation(tool: keyof typeof MUTATIONS, result: JsonObject, expectedActor?: string): ValidatedAuthority {
	const contract = MUTATIONS[tool];
	const serviceResult = objectField(result, "service_result");
	const actor = stringField(result, "actor_did");
	const resource = stringField(result, "resource_id");
	const state = stringField(result, "resulting_state");
	const receiptId = stringField(result, "service_receipt_id");
	const receiptDigest = digestField(result, "service_receipt_digest");
	if (!expectedActor || result.authority_kind !== "service-receipt" || result.tool !== tool
		|| actor !== expectedActor || serviceResult.actor_did !== actor || serviceResult.action !== contract.action
		|| serviceResult[contract.resource] !== resource || serviceResult.state !== state
		|| serviceResult.receipt_id !== receiptId || serviceResult.receipt_digest !== receiptDigest
		|| !contract.states.some((candidate) => candidate === state)) {
		throw new Error("MCP_AUTHORITY_RECEIPT_INVALID");
	}
	return {
		serviceResult,
		receipt: { actor_did: actor, tool, action: contract.action, resource_id: resource, resulting_state: state,
			service_receipt_id: receiptId, service_receipt_digest: receiptDigest },
	};
}

function objectField(value: JsonObject, field: string): JsonObject {
	if (value[field] === undefined) throw new Error("MCP_AUTHORITY_RECEIPT_MISSING");
	if (isObject(value[field])) return value[field];
	throw new Error("MCP_AUTHORITY_RECEIPT_INVALID");
}
function stringField(value: JsonObject, field: string): string {
	const parsed = value[field];
	if (parsed === undefined) throw new Error("MCP_AUTHORITY_RECEIPT_MISSING");
	if (typeof parsed === "string" && parsed.length > 0) return parsed;
	throw new Error("MCP_AUTHORITY_RECEIPT_INVALID");
}
function digestField(value: JsonObject, field: string): string {
	const parsed = stringField(value, field);
	if (/^sha256:[0-9a-f]{64}$/.test(parsed)) return parsed;
	throw new Error("MCP_AUTHORITY_RECEIPT_INVALID");
}
function isObject(value: unknown): value is JsonObject {
	return typeof value === "object" && value !== null && !Array.isArray(value);
}
