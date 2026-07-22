import { createHash } from "node:crypto";
import { readFile, writeFile } from "node:fs/promises";
import {
	normalizeServiceReceipts, ROLES, validateGlobalReceiptUniqueness, validateRoleAuthority,
	type Role, type ServiceReceipt,
} from "./pi-service-authority-evidence.ts";

const SCHEMA = "kamn.mvp.pi-transaction-actor.v2";
const SHARED = [
	"task_id", "transaction_id", "escrow_id", "amount_lamports", "network", "settlement_tx_signature",
	"settlement_commitment", "receipt_chain_commitment", "public_commitment",
] as const;
const INPUT_FIELDS = new Set([
	"actor", "pi_process_id", "did", "mcp_child_process_id", "first_request_id", "last_request_id",
	"transport_response_digests", "service_profile_commitment", "service_receipts", ...SHARED,
	"view_scope", "participant_role", "source_handoff_digest", "handoff_authorized",
]);
type ActorArtifact = {
	schema_version: string; actor: Role; pi_process_id: number; did: string; mcp_child_process_id: number;
	first_request_id: number; last_request_id: number; transport_response_digests: string[];
	service_profile_commitment: string; service_receipts: ServiceReceipt[]; task_id: string; transaction_id: string;
	escrow_id: string; amount_lamports: number; network: string; settlement_tx_signature: string;
	settlement_commitment: string; receipt_chain_commitment: string; public_commitment: string; view_scope: string;
	participant_role?: string; source_handoff_digest: string; handoff_authorized: boolean; artifact_digest: string;
};

export async function writePiTransactionActor(path: string, input: Record<string, unknown>) {
	const unsigned = normalizeActor(input);
	const artifact = { ...unsigned, artifact_digest: digest(unsigned) };
	const serialized = `${JSON.stringify(artifact)}\n`;
	try {
		await writeFile(path, serialized, { flag: "wx", mode: 0o600 });
	} catch (error) {
		if (!isNodeError(error, "EEXIST")) throw error;
		if (await readFile(path, "utf8") !== serialized) throw new Error("PI_ACTOR_ARTIFACT_CONFLICT");
	}
}

export async function verifyPiTransactionActors(paths: Record<Role, string>) {
	const actors = await Promise.all(ROLES.map((role) => readActor(paths[role], role)));
	requireDistinct(actors.map((actor) => actor.pi_process_id), "PI_ACTOR_PROCESS_REUSED");
	requireDistinct(actors.map((actor) => actor.mcp_child_process_id), "PI_ACTOR_PROCESS_REUSED");
	requireDistinct(actors.map((actor) => actor.did), "PI_ACTOR_IDENTITY_INVALID");
	validateSharedFacts(actors);
	validateGlobalReceiptUniqueness(actors.flatMap((actor) => actor.service_receipts));
	const first = actors[0];
	return {
		task_id: first.task_id, escrow_id: first.escrow_id,
		settlement_tx_signature: first.settlement_tx_signature,
		receipt_chain_commitment: first.receipt_chain_commitment,
		public_commitment: first.public_commitment,
		pi_process_ids: actors.map((actor) => actor.pi_process_id),
		dids: actors.map((actor) => actor.did),
	};
}

function normalizeActor(input: Record<string, unknown>): Omit<ActorArtifact, "artifact_digest"> {
	if (Object.keys(input).some((field) => !INPUT_FIELDS.has(field))) authorityFail();
	const actor = requiredRole(input.actor);
	const artifact = {
		schema_version: SCHEMA,
		actor,
		pi_process_id: positiveInteger(input.pi_process_id, "PI_ACTOR_PROCESS_REUSED"),
		did: requiredString(input.did, "PI_ACTOR_IDENTITY_INVALID"),
		mcp_child_process_id: positiveInteger(input.mcp_child_process_id, "PI_ACTOR_PROCESS_REUSED"),
		first_request_id: positiveInteger(input.first_request_id, "PI_TRANSPORT_PROVENANCE_INVALID"),
		last_request_id: positiveInteger(input.last_request_id, "PI_TRANSPORT_PROVENANCE_INVALID"),
		transport_response_digests: digestList(input.transport_response_digests),
		service_profile_commitment: shaDigest(input.service_profile_commitment, "PI_SERVICE_AUTHORITY_MISMATCH"),
		service_receipts: normalizeServiceReceipts(input.service_receipts),
		...transactionFields(input), ...disclosureFields(input),
		source_handoff_digest: shaDigest(input.source_handoff_digest, "PI_SERVICE_AUTHORITY_MISMATCH"),
		handoff_authorized: input.handoff_authorized === true,
	};
	validateActor(artifact);
	return artifact;
}

function validateActor(actor: Omit<ActorArtifact, "artifact_digest">) {
	const expectedCount = actor.last_request_id - actor.first_request_id + 1;
	if (expectedCount !== actor.transport_response_digests.length) throw new Error("PI_TRANSPORT_PROVENANCE_INVALID");
	validateRoleAuthority(actor.actor, actor.did, actor.task_id, actor.escrow_id, actor.service_receipts);
	if (actor.handoff_authorized) throw new Error("PI_HANDOFF_AUTHORIZATION_FORBIDDEN");
	if (actor.actor === "agent_c") {
		if (actor.view_scope !== "restricted-public") throw new Error("PI_VERIFIER_PROJECTION_MISSING");
		if (actor.participant_role !== undefined) throw new Error("PI_VERIFIER_PRIVATE_LEAK");
		return;
	}
	const expectedRole = actor.actor === "agent_a" ? "creator" : "provider";
	if (actor.view_scope !== "participant-private") authorityFail();
	if (actor.participant_role !== expectedRole) throw new Error("PI_ACTOR_IDENTITY_INVALID");
}

function transactionFields(input: Record<string, unknown>) {
	return {
		task_id: requiredString(input.task_id, "PI_TRANSACTION_FACT_MISMATCH"),
		transaction_id: requiredString(input.transaction_id, "PI_TRANSACTION_FACT_MISMATCH"),
		escrow_id: requiredString(input.escrow_id, "PI_TRANSACTION_FACT_MISMATCH"),
		amount_lamports: positiveInteger(input.amount_lamports, "PI_TRANSACTION_FACT_MISMATCH"),
		network: requiredString(input.network, "PI_TRANSACTION_FACT_MISMATCH"),
		settlement_tx_signature: requiredString(input.settlement_tx_signature, "PI_TRANSACTION_FACT_MISMATCH"),
		settlement_commitment: requiredString(input.settlement_commitment, "PI_TRANSACTION_FACT_MISMATCH"),
		receipt_chain_commitment: shaDigest(input.receipt_chain_commitment, "PI_TRANSACTION_FACT_MISMATCH"),
		public_commitment: shaDigest(input.public_commitment, "PI_TRANSACTION_FACT_MISMATCH"),
	};
}
function disclosureFields(input: Record<string, unknown>) {
	return {
		view_scope: requiredString(input.view_scope, "PI_VERIFIER_PROJECTION_MISSING"),
		...(input.participant_role === undefined ? {} : { participant_role: requiredString(input.participant_role, "PI_ACTOR_IDENTITY_INVALID") }),
	};
}
async function readActor(path: string, role: Role): Promise<ActorArtifact> {
	const parsed = JSON.parse(await readFile(path, "utf8")) as Record<string, unknown>;
	const digestValue = shaDigest(parsed.artifact_digest, "PI_SERVICE_AUTHORITY_MISMATCH");
	const unsigned = Object.fromEntries(Object.entries(parsed).filter(([key]) => key !== "artifact_digest"));
	if (digest(unsigned) !== digestValue || unsigned.schema_version !== SCHEMA) authorityFail();
	const input = Object.fromEntries(Object.entries(unsigned).filter(([key]) => key !== "schema_version"));
	const normalized = normalizeActor(input);
	if (normalized.actor !== role) throw new Error("PI_ACTOR_IDENTITY_INVALID");
	return { ...normalized, artifact_digest: digestValue };
}
function validateSharedFacts(actors: ActorArtifact[]) {
	const first = actors[0] as unknown as Record<string, unknown>;
	for (const actor of actors.slice(1) as unknown as Array<Record<string, unknown>>) {
		if (SHARED.some((field) => actor[field] !== first[field])) throw new Error("PI_TRANSACTION_FACT_MISMATCH");
	}
}
function requireDistinct(values: Array<string | number>, code: string) {
	if (new Set(values).size !== values.length) throw new Error(code);
}
function requiredRole(value: unknown): Role {
	if (ROLES.includes(value as Role)) return value as Role;
	throw new Error("PI_ACTOR_IDENTITY_INVALID");
}
function requiredString(value: unknown, code: string): string {
	if (typeof value === "string" && value.trim()) return value;
	throw new Error(code);
}
function positiveInteger(value: unknown, code: string): number {
	if (typeof value === "number" && Number.isSafeInteger(value) && value > 0) return value;
	throw new Error(code);
}
function shaDigest(value: unknown, code: string): string {
	const parsed = requiredString(value, code);
	if (/^sha256:[0-9a-f]{64}$/.test(parsed)) return parsed;
	throw new Error(code);
}
function digestList(value: unknown): string[] {
	if (!Array.isArray(value) || value.length === 0) throw new Error("PI_TRANSPORT_PROVENANCE_INVALID");
	return value.map((entry) => shaDigest(entry, "PI_TRANSPORT_PROVENANCE_INVALID"));
}
function digest(value: unknown): string {
	return `sha256:${createHash("sha256").update(JSON.stringify(value)).digest("hex")}`;
}
function isNodeError(error: unknown, code: string): boolean {
	return error instanceof Error && "code" in error && error.code === code;
}
function authorityFail(): never { throw new Error("PI_SERVICE_AUTHORITY_MISMATCH"); }
