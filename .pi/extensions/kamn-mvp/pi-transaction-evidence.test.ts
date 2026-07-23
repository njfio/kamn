import assert from "node:assert/strict";
import { mkdtemp } from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import test from "node:test";
import { verifyPiTransactionActors, writePiTransactionActor } from "./pi-transaction-evidence.ts";

type Role = "agent_a" | "agent_b" | "agent_c";
type Overrides = Partial<Record<Role, Record<string, unknown>>>;

test("three actor artifacts bind independent processes to one receipt chain", async () => {
	const paths = await actorPaths();
	await writeActors(paths);
	const verified = await verifyPiTransactionActors(paths);

	assert.equal(verified.task_id, "task-live-7099");
	assert.equal(verified.escrow_id, "escrow-live-7099");
	assert.equal(verified.receipt_chain_commitment, digest("c"));
	assert.deepEqual(verified.pi_process_ids, [101, 202, 303]);
	assert.deepEqual(verified.dids, ["kamn:did:a", "kamn:did:b", "kamn:did:c"]);
});

test("actor verification rejects process, DID, and MCP child reuse", async () => {
	for (const field of ["pi_process_id", "did", "mcp_child_process_id"] as const) {
		const paths = await actorPaths();
		await writeActors(paths, { agent_c: { [field]: actor("agent_a")[field] } });
		await assert.rejects(verifyPiTransactionActors(paths), /PI_ACTOR_(PROCESS_REUSED|IDENTITY_INVALID)/);
	}
});

test("actor verification rejects copied facts and handoff authorization", async () => {
	const mismatch = await actorPaths();
	await writeActors(mismatch, { agent_b: { receipt_chain_commitment: digest("9") } });
	await assert.rejects(verifyPiTransactionActors(mismatch), /PI_TRANSACTION_FACT_MISMATCH/);

	const handoff = await actorPaths();
	await assert.rejects(writeActors(handoff, { agent_a: { handoff_authorized: true } }), /PI_HANDOFF_AUTHORIZATION_FORBIDDEN/);
});

test("actor evidence rejects verifier private fields and unknown ambient evidence", async () => {
	const privateField = await actorPaths();
	await assert.rejects(writeActors(privateField, { agent_c: { participant_role: "creator" } }), /PI_VERIFIER_PRIVATE_LEAK/);

	const unknown = await actorPaths();
	await assert.rejects(writeActors(unknown, { agent_a: { actor_evidence: "trusted" } }), /PI_SERVICE_AUTHORITY_MISMATCH/);
});

test("actor artifact writes are idempotent and conflicts fail closed", async () => {
	const paths = await actorPaths();
	await writePiTransactionActor(paths.agent_a, actor("agent_a"));
	await assert.doesNotReject(writePiTransactionActor(paths.agent_a, actor("agent_a")));
	await assert.rejects(
		writePiTransactionActor(paths.agent_a, { ...actor("agent_a"), amount_lamports: 2_000_000 }),
		/PI_ACTOR_ARTIFACT_CONFLICT/,
	);
});

async function writeActors(paths: Record<Role, string>, overrides: Overrides = {}) {
	for (const role of ["agent_a", "agent_b", "agent_c"] as const) {
		await writePiTransactionActor(paths[role], { ...actor(role), ...overrides[role] });
	}
}

function actor(role: Role): Record<string, unknown> {
	const index = { agent_a: 1, agent_b: 2, agent_c: 3 }[role];
	return {
		actor: role,
		pi_process_id: index * 101,
		did: `kamn:did:${String.fromCharCode(96 + index)}`,
		mcp_child_process_id: 1000 + index,
		first_request_id: 1,
		last_request_id: 2,
		transport_response_digests: [digest(String(index)), digest(String(index + 3))],
		service_profile_commitment: digest("f"),
		service_receipts: receipts(role),
		task_id: "task-live-7099",
		transaction_id: "transaction-live-7099",
		escrow_id: "escrow-live-7099",
		amount_lamports: 1_000_000,
		network: "solana-devnet",
		settlement_tx_signature: "devnet-signature-7099",
		settlement_commitment: "finalized",
		receipt_chain_commitment: digest("c"),
		public_commitment: digest("d"),
		view_scope: role === "agent_c" ? "restricted-public" : "participant-private",
		...(role === "agent_c" ? {} : { participant_role: role === "agent_a" ? "creator" : "provider" }),
		source_handoff_digest: digest("b"),
		handoff_authorized: false,
	};
}

function receipts(role: Role) {
	if (role === "agent_a") return [
		receipt(role, "create_task", "task:create", "task-live-7099", "submitted", "1"),
		receipt(role, "fund_escrow", "escrow:fund", "escrow-live-7099", "funded", "2"),
		receipt(role, "release_escrow", "escrow:release-authorize", "escrow-live-7099", "release-authorized", "3"),
		receipt(role, "release_escrow", "settlement:confirmed", "escrow-live-7099", "confirmed", "6"),
	];
	if (role === "agent_b") return [
		receipt(role, "accept_task", "task:accept", "task-live-7099", "accepted", "4"),
		receipt(role, "complete_task", "task:complete", "task-live-7099", "completed", "5"),
	];
	return [];
}

function receipt(role: Role, tool: string, action: string, resource_id: string, resulting_state: string, id: string) {
	return {
		actor_did: `kamn:did:${role === "agent_a" ? "a" : "b"}`,
		tool, action, resource_id, resulting_state,
		service_receipt_id: `service-receipt-${id}`,
		service_receipt_digest: digest(id),
	};
}

async function actorPaths(): Promise<Record<Role, string>> {
	const root = await mkdtemp(resolve(tmpdir(), "kamn-pi-transaction-"));
	return { agent_a: resolve(root, "agent-a.json"), agent_b: resolve(root, "agent-b.json"), agent_c: resolve(root, "agent-c.json") };
}
function digest(character: string): string { return `sha256:${character.repeat(64)}`; }
