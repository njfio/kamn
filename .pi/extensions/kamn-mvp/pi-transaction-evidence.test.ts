import assert from "node:assert/strict";
import { mkdtemp } from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import test from "node:test";
import { verifyPiTransactionActors, writePiTransactionActor } from "./pi-transaction-evidence.ts";

test("three actor artifacts bind independent runtime provenance to one transaction", async () => {
	const paths = await actorPaths();
	await writeActors(paths);
	const verified = await verifyPiTransactionActors(paths);

	assert.equal(verified.task_id, "task-live-7099");
	assert.equal(verified.escrow_id, "escrow-live-7099");
	assert.equal(verified.settlement_tx_signature, "devnet-signature-7099");
	assert.equal(verified.public_commitment, `sha256:${"d".repeat(64)}`);
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

test("actor verification rejects missing runtime projection provenance and private verifier data", async () => {
	const missing = await actorPaths();
	await writeActors(missing, { agent_c: { runtime_projection_digest: `sha256:${"f".repeat(64)}` } });
	await assert.rejects(verifyPiTransactionActors(missing), /PI_RUNTIME_RECEIPT_MISMATCH/);

	const leaked = await actorPaths();
	await writeActors(leaked, { agent_c: { private_receipt_digest: `sha256:${"e".repeat(64)}` } });
	await assert.rejects(verifyPiTransactionActors(leaked), /PI_VERIFIER_PRIVATE_LEAK/);
});

test("actor verification rejects copied facts and handoff authorization", async () => {
	const mismatch = await actorPaths();
	await writeActors(mismatch, { agent_b: { escrow_id: "escrow-other" } });
	await assert.rejects(verifyPiTransactionActors(mismatch), /PI_TRANSACTION_FACT_MISMATCH/);

	const handoff = await actorPaths();
	await writeActors(handoff, { agent_a: { handoff_authorized: true } });
	await assert.rejects(verifyPiTransactionActors(handoff), /PI_HANDOFF_AUTHORIZATION_FORBIDDEN/);
});

type Role = "agent_a" | "agent_b" | "agent_c";
type Overrides = Partial<Record<Role, Record<string, unknown>>>;

async function writeActors(paths: Record<Role, string>, overrides: Overrides = {}) {
	for (const role of ["agent_a", "agent_b", "agent_c"] as const) {
		await writePiTransactionActor(paths[role], { ...actor(role), ...overrides[role] });
	}
}

function actor(role: Role) {
	const index = { agent_a: 1, agent_b: 2, agent_c: 3 }[role];
	const projectionDigest = `sha256:${String(index).repeat(64)}`;
	return {
		actor: role,
		pi_process_id: index * 101,
		did: `kamn:did:${String.fromCharCode(96 + index)}`,
		mcp_child_process_id: 1000 + index,
		first_request_id: 1,
		last_request_id: 5,
		runtime_response_digests: [`sha256:${"a".repeat(64)}`, projectionDigest],
		runtime_projection_digest: projectionDigest,
		task_id: "task-live-7099",
		transaction_id: "transaction-live-7099",
		escrow_id: "escrow-live-7099",
		amount_lamports: 1000000,
		network: "solana-devnet",
		settlement_tx_signature: "devnet-signature-7099",
		settlement_commitment: "finalized",
		public_commitment: `sha256:${"d".repeat(64)}`,
		view_scope: role === "agent_c" ? "restricted-public" : "participant-private",
		...(role === "agent_c" ? {} : { private_receipt_digest: `sha256:${"e".repeat(64)}` }),
		source_handoff_digest: `sha256:${"b".repeat(64)}`,
		handoff_authorized: false,
	};
}

async function actorPaths(): Promise<Record<Role, string>> {
	const root = await mkdtemp(resolve(tmpdir(), "kamn-pi-transaction-"));
	return {
		agent_a: resolve(root, "agent-a.json"),
		agent_b: resolve(root, "agent-b.json"),
		agent_c: resolve(root, "agent-c.json"),
	};
}
