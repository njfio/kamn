import assert from "node:assert/strict";
import { mkdtemp, readFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import test from "node:test";
import { verifyPiTransactionActors, writePiTransactionActor } from "./pi-transaction-evidence.ts";

type Role = "agent_a" | "agent_b" | "agent_c";
type Receipt = ReturnType<typeof receipt>;

test("v2 actor artifacts bind exact role service authority and chain commitment", async () => {
	const paths = await actorPaths();
	await Promise.all((Object.keys(paths) as Role[]).map((role) =>
		writePiTransactionActor(paths[role], actorInput(role)),
	));

	const verified = await verifyPiTransactionActors(paths);
	assert.equal(verified.receipt_chain_commitment, digest("c"));
	const actorA = JSON.parse(await readFile(paths.agent_a, "utf8"));
	assert.equal(actorA.schema_version, "kamn.mvp.pi-transaction-actor.v2");
	assert.deepEqual(actorA.service_receipts.map((entry: Receipt) => entry.action), [
		"task:create", "escrow:fund", "escrow:release-authorize", "settlement:confirmed",
	]);
	assert.equal("runtime_response_receipts" in actorA, false);
	assert.equal("runtime_response_digests" in actorA, false);
});

for (const [label, mutate] of [
	["reordered", (input: Record<string, unknown>) => {
		(input.service_receipts as Receipt[]).reverse();
	}],
	["replayed", (input: Record<string, unknown>) => {
		const receipts = input.service_receipts as Receipt[];
		receipts[1] = { ...receipts[1], service_receipt_id: receipts[0].service_receipt_id };
	}],
	["cross-role", (input: Record<string, unknown>) => {
		const receipts = input.service_receipts as Receipt[];
		receipts[0] = { ...receipts[0], actor_did: roleDid("agent_b") };
	}],
] as const) {
	test(`v2 actor evidence rejects ${label} service receipts`, async () => {
		const paths = await actorPaths();
		const input = actorInput("agent_a");
		mutate(input);
		await assert.rejects(
			writePiTransactionActor(paths.agent_a, input),
			/PI_SERVICE_AUTHORITY_MISMATCH/,
		);
	});
}

test("Agent C rejects copied participant receipt authority", async () => {
	const paths = await actorPaths();
	const input = actorInput("agent_c");
	input.service_receipts = [receipt("agent_c", "task:create", "task-live-1", "submitted", "9")];

	await assert.rejects(
		writePiTransactionActor(paths.agent_c, input),
		/PI_SERVICE_AUTHORITY_MISMATCH/,
	);
});

test("legacy local runtime receipts cannot satisfy canonical v2 evidence", async () => {
	const paths = await actorPaths();
	const input = actorInput("agent_a");
	delete input.service_receipts;
	delete input.service_profile_commitment;
	delete input.transport_response_digests;
	delete input.receipt_chain_commitment;
	input.runtime_response_digests = [digest("1")];
	input.runtime_response_receipts = [{
		request_id: 1,
		tool: "register",
		outcome: "success",
		digest: digest("1"),
		public_result: { did: roleDid("agent_a") },
	}];
	input.runtime_projection_digest = digest("1");

	await assert.rejects(
		writePiTransactionActor(paths.agent_a, input),
		/PI_SERVICE_AUTHORITY_MISMATCH/,
	);
});

function actorInput(role: Role): Record<string, unknown> {
	return {
		actor: role,
		pi_process_id: { agent_a: 101, agent_b: 202, agent_c: 303 }[role],
		did: roleDid(role),
		mcp_child_process_id: { agent_a: 1001, agent_b: 1002, agent_c: 1003 }[role],
		first_request_id: 1,
		last_request_id: 1,
		transport_response_digests: [digest(role === "agent_a" ? "1" : role === "agent_b" ? "2" : "3")],
		service_profile_commitment: digest("f"),
		service_receipts: roleReceipts(role),
		task_id: "task-live-1",
		transaction_id: "transaction-live-1",
		escrow_id: "escrow-live-1",
		amount_lamports: 1_000_000,
		network: "solana-devnet",
		settlement_tx_signature: "devnet-signature-1",
		settlement_commitment: "finalized",
		receipt_chain_commitment: digest("c"),
		public_commitment: digest("d"),
		view_scope: role === "agent_c" ? "restricted-public" : "participant-private",
		...(role === "agent_c" ? {} : { participant_role: role === "agent_a" ? "creator" : "provider" }),
		source_handoff_digest: digest("e"),
		handoff_authorized: false,
	};
}

function roleReceipts(role: Role): Receipt[] {
	if (role === "agent_a") return [
		receipt(role, "task:create", "task-live-1", "submitted", "1"),
		receipt(role, "escrow:fund", "escrow-live-1", "funded", "2"),
		receipt(role, "escrow:release-authorize", "escrow-live-1", "release-authorized", "3"),
		receipt(role, "settlement:confirmed", "escrow-live-1", "confirmed", "6"),
	];
	if (role === "agent_b") return [
		receipt(role, "task:accept", "task-live-1", "accepted", "4"),
		receipt(role, "task:complete", "task-live-1", "completed", "5"),
	];
	return [];
}

function receipt(role: Role, action: string, resource: string, state: string, suffix: string) {
	return {
		actor_did: roleDid(role),
		tool: ({
			"task:create": "create_task",
			"task:accept": "accept_task",
			"task:complete": "complete_task",
			"escrow:fund": "fund_escrow",
			"escrow:release-authorize": "release_escrow",
			"settlement:confirmed": "release_escrow",
		} as Record<string, string>)[action],
		action,
		resource_id: resource,
		resulting_state: state,
		service_receipt_id: `service-receipt-${suffix}`,
		service_receipt_digest: digest(suffix),
	};
}

async function actorPaths() {
	const root = await mkdtemp(resolve(tmpdir(), "kamn-pi-service-authority-"));
	return {
		agent_a: resolve(root, "agent-a.json"),
		agent_b: resolve(root, "agent-b.json"),
		agent_c: resolve(root, "agent-c.json"),
	};
}

function roleDid(role: Role): string {
	return `kamn:did:${role.replace("_", "-")}`;
}

function digest(character: string): string {
	return `sha256:${character.repeat(64)}`;
}
