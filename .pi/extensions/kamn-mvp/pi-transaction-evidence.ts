import { createHash } from "node:crypto";
import { readFile, writeFile } from "node:fs/promises";
import { normalizeRuntimeReceipts, validateRuntimeReceipts, type Role, type RuntimeReceipt } from "./pi-transaction-runtime-receipts.ts";

const SCHEMA = "kamn.mvp.pi-transaction-actor.v1";
const ROLES = ["agent_a", "agent_b", "agent_c"] as const;
const SHARED = [
	"task_id", "transaction_id", "escrow_id", "amount_lamports", "network",
	"settlement_tx_signature", "settlement_commitment", "public_commitment",
] as const;
const INPUT_FIELDS = new Set([
	"actor", "pi_process_id", "did", "mcp_child_process_id", "first_request_id", "last_request_id",
	"runtime_response_digests", "runtime_response_receipts", "runtime_projection_digest", ...SHARED,
	"view_scope", "participant_role", "private_receipt_digest", "source_handoff_digest", "handoff_authorized",
]);
type ActorArtifact = {
	schema_version: string;
	actor: Role;
	pi_process_id: number;
	did: string;
	mcp_child_process_id: number;
	first_request_id: number;
	last_request_id: number;
	runtime_response_digests: string[];
	runtime_response_receipts: RuntimeReceipt[];
	runtime_projection_digest: string;
	task_id: string;
	transaction_id: string;
	escrow_id: string;
	amount_lamports: number;
	network: string;
	settlement_tx_signature: string;
	settlement_commitment: string;
	public_commitment: string;
	view_scope: string;
	participant_role?: string;
	private_receipt_digest?: string;
	source_handoff_digest: string;
	handoff_authorized: boolean;
	artifact_digest: string;
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
	for (const actor of actors) validateActor(actor);
	validateSharedFacts(actors);
	const first = actors[0];
	return {
		task_id: first.task_id,
		escrow_id: first.escrow_id,
		settlement_tx_signature: first.settlement_tx_signature,
		public_commitment: first.public_commitment,
		pi_process_ids: actors.map((actor) => actor.pi_process_id),
		dids: actors.map((actor) => actor.did),
	};
}

function normalizeActor(input: Record<string, unknown>): Omit<ActorArtifact, "artifact_digest"> {
	assertKnownInputFields(input);
	const artifact = {
		schema_version: SCHEMA,
		...identityFields(input),
		...runtimeFields(input),
		...transactionFields(input),
		...disclosureFields(input),
		source_handoff_digest: shaDigest(input.source_handoff_digest, "PI_RUNTIME_RECEIPT_MISMATCH"),
		handoff_authorized: input.handoff_authorized === true,
	};
	validateActor(artifact as ActorArtifact);
	return artifact;
}
function assertKnownInputFields(input: Record<string, unknown>) {
	if (Object.keys(input).some((field) => !INPUT_FIELDS.has(field))) throw new Error("PI_RUNTIME_RECEIPT_MISMATCH");
}
function identityFields(input: Record<string, unknown>) {
	return {
		actor: requiredRole(input.actor),
		pi_process_id: positiveInteger(input.pi_process_id, "PI_ACTOR_PROCESS_REUSED"),
		did: requiredString(input.did, "PI_ACTOR_IDENTITY_INVALID"),
		mcp_child_process_id: positiveInteger(input.mcp_child_process_id, "PI_ACTOR_PROCESS_REUSED"),
	};
}
function runtimeFields(input: Record<string, unknown>) {
	return {
		first_request_id: positiveInteger(input.first_request_id, "PI_ACTOR_NONCE_STREAM_INVALID"),
		last_request_id: positiveInteger(input.last_request_id, "PI_ACTOR_NONCE_STREAM_INVALID"),
		runtime_response_digests: digestList(input.runtime_response_digests),
		runtime_response_receipts: normalizeRuntimeReceipts(input.runtime_response_receipts),
		runtime_projection_digest: shaDigest(input.runtime_projection_digest, "PI_RUNTIME_RECEIPT_MISMATCH"),
	};
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
		public_commitment: shaDigest(input.public_commitment, "PI_TRANSACTION_FACT_MISMATCH"),
	};
}
function disclosureFields(input: Record<string, unknown>) {
	return {
		view_scope: requiredString(input.view_scope, "PI_VERIFIER_PROJECTION_MISSING"),
		...(input.participant_role === undefined ? {} : { participant_role: requiredString(input.participant_role, "PI_ACTOR_IDENTITY_INVALID") }),
		...(input.private_receipt_digest === undefined ? {} : { private_receipt_digest: shaDigest(input.private_receipt_digest, "PI_VERIFIER_PRIVATE_LEAK") }),
	};
}

async function readActor(path: string, role: Role): Promise<ActorArtifact> {
	const parsed = JSON.parse(await readFile(path, "utf8")) as Record<string, unknown>;
	const digestValue = shaDigest(parsed.artifact_digest, "PI_RUNTIME_RECEIPT_MISMATCH");
	const unsigned = Object.fromEntries(Object.entries(parsed).filter(([key]) => key !== "artifact_digest"));
	if (digest(unsigned) !== digestValue) throw new Error("PI_RUNTIME_RECEIPT_MISMATCH");
	if (unsigned.schema_version !== SCHEMA) throw new Error("PI_RUNTIME_RECEIPT_MISMATCH");
	const input = Object.fromEntries(Object.entries(unsigned).filter(([key]) => key !== "schema_version"));
	const normalized = normalizeActor(input) as ActorArtifact;
	if (normalized.actor !== role) throw new Error("PI_ACTOR_IDENTITY_INVALID");
	return { ...normalized, artifact_digest: digestValue };
}

function validateActor(actor: Omit<ActorArtifact, "artifact_digest">) {
	const expectedCount = actor.last_request_id - actor.first_request_id + 1;
	if (expectedCount !== actor.runtime_response_digests.length) throw new Error("PI_ACTOR_NONCE_STREAM_INVALID");
	validateRuntimeReceipts(actor.actor, actor.first_request_id, actor.runtime_response_digests, actor.runtime_response_receipts);
	if (!actor.runtime_response_digests.includes(actor.runtime_projection_digest)) throw new Error("PI_RUNTIME_RECEIPT_MISMATCH");
	if (actor.handoff_authorized) throw new Error("PI_HANDOFF_AUTHORIZATION_FORBIDDEN");
	if (actor.actor === "agent_c") {
		if (actor.view_scope !== "restricted-public") throw new Error("PI_VERIFIER_PROJECTION_MISSING");
		if (actor.private_receipt_digest !== undefined) throw new Error("PI_VERIFIER_PRIVATE_LEAK");
		if (actor.participant_role !== undefined) throw new Error("PI_VERIFIER_PRIVATE_LEAK");
		return;
	}
	const expectedRole = actor.actor === "agent_a" ? "creator" : "provider";
	if (actor.view_scope !== "participant-private" || !actor.private_receipt_digest) throw new Error("PI_RUNTIME_RECEIPT_MISMATCH");
	if (actor.participant_role !== expectedRole) throw new Error("PI_ACTOR_IDENTITY_INVALID");
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
	if (!Array.isArray(value) || value.length === 0) throw new Error("PI_RUNTIME_RECEIPT_MISMATCH");
	return value.map((entry) => shaDigest(entry, "PI_RUNTIME_RECEIPT_MISMATCH"));
}
function digest(value: unknown): string {
	return `sha256:${createHash("sha256").update(JSON.stringify(value)).digest("hex")}`;
}
function isNodeError(error: unknown, code: string): boolean {
	return error instanceof Error && "code" in error && error.code === code;
}
