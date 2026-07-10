import { createHash } from "node:crypto";
import { readFile, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import { readVerifiedActorEvidence } from "./live-task-coordination.ts";

const SCHEMA = "kamn.mvp.live-task-restricted-observation.v1";
const CLAIM = "real local-only independent Agent C artifact observation";
const SECRET_MARKERS = [".kamn/devnet", "auth.json", ".env", "keypair", "id_rsa", "oauth"];
const FIELDS = [
	"agent_c_pi_process_id", "artifact_digest", "private_field_count", "private_payload_redacted",
	"public_commitment", "schema_version", "source_agent_a_receipt_digest",
	"source_agent_b_receipt_digest", "source_handoff_digest", "state", "task_id", "view_scope",
];
type Environment = Record<string, string | undefined>;
type Observation = {
	schema_version: string;
	task_id: string;
	state: string;
	view_scope: string;
	private_field_count: number;
	private_payload_redacted: boolean;
	agent_c_pi_process_id: number;
	source_handoff_digest: string;
	source_agent_a_receipt_digest: string;
	source_agent_b_receipt_digest: string;
	public_commitment: string;
	artifact_digest: string;
};

export async function writeRestrictedTaskObservation(
	handoffPath: string, agentAPath: string, agentBPath: string, outputPath: string, agentCPid: number,
) {
	assertSafePath(outputPath);
	assertDistinctOutput(outputPath, [handoffPath, agentAPath, agentBPath]);
	const source = await readVerifiedActorEvidence(handoffPath, agentAPath, agentBPath);
	assertThirdProcess(agentCPid, source.agent_a_pi_process_id, source.agent_b_pi_process_id);
	const publicFields = {
		task_id: source.task_id,
		state: source.state,
		source_handoff_digest: source.source_handoff_digest,
		source_agent_a_receipt_digest: source.source_agent_a_receipt_digest,
		source_agent_b_receipt_digest: source.source_agent_b_receipt_digest,
	};
	const unsigned = {
		schema_version: SCHEMA, ...publicFields, view_scope: "restricted-public",
		private_field_count: 0, private_payload_redacted: true, agent_c_pi_process_id: agentCPid,
		public_commitment: digest(publicFields),
	};
	await writeIdempotent(outputPath, { ...unsigned, artifact_digest: digest(unsigned) });
	return verifyRestrictedTaskObservation(outputPath, handoffPath, agentAPath, agentBPath);
}

export async function verifyRestrictedTaskObservation(
	path: string, handoffPath: string, agentAPath: string, agentBPath: string,
) {
	assertSafePath(path);
	const artifact = parse(await readFile(path, "utf8"));
	assertPolicy(artifact);
	const source = await readVerifiedActorEvidence(handoffPath, agentAPath, agentBPath);
	assertBoundToSource(artifact, source);
	assertThirdProcess(artifact.agent_c_pi_process_id, source.agent_a_pi_process_id, source.agent_b_pi_process_id);
	return {
		claim_boundary: CLAIM,
		task_id: artifact.task_id,
		state: artifact.state,
		view_scope: artifact.view_scope,
		private_field_count: artifact.private_field_count,
		private_payload_redacted: artifact.private_payload_redacted,
		agent_c_pi_process_id: artifact.agent_c_pi_process_id,
		public_commitment: artifact.public_commitment,
	};
}

function assertBoundToSource(artifact: Observation, source: Awaited<ReturnType<typeof readVerifiedActorEvidence>>) {
	if (artifact.task_id !== source.task_id || artifact.state !== source.state) {
		throw new Error("KAMN Agent C observation task or state mismatch");
	}
	const actual = [artifact.source_handoff_digest, artifact.source_agent_a_receipt_digest, artifact.source_agent_b_receipt_digest];
	const expected = [source.source_handoff_digest, source.source_agent_a_receipt_digest, source.source_agent_b_receipt_digest];
	if (actual.some((digestValue, index) => digestValue !== expected[index])) {
		throw new Error("KAMN Agent C observation source digest mismatch");
	}
}

export function restrictedObservationConfig(env: Environment, cwd: string) {
	return {
		handoffPath: configuredPath(env, "KAMN_MVP_LIVE_TASK_HANDOFF_FILE", cwd),
		agentAPath: configuredPath(env, "KAMN_MVP_LIVE_TASK_AGENT_A_RECEIPT_FILE", cwd),
		agentBPath: configuredPath(env, "KAMN_MVP_LIVE_TASK_AGENT_B_RECEIPT_FILE", cwd),
		observationPath: configuredPath(env, "KAMN_MVP_LIVE_TASK_AGENT_C_OBSERVATION_FILE", cwd),
	};
}

function parse(raw: string): Observation {
	let artifact: Record<string, unknown>;
	try { artifact = JSON.parse(raw) as Record<string, unknown>; } catch { throw new Error("KAMN Agent C observation is malformed JSON"); }
	if (!artifact || Array.isArray(artifact) || artifact.schema_version !== SCHEMA) throw new Error("KAMN Agent C observation schema mismatch");
	if (Object.keys(artifact).sort().join(",") !== FIELDS.join(",")) throw new Error("KAMN Agent C observation field mismatch");
	const unsigned = Object.fromEntries(Object.entries(artifact).filter(([key]) => key !== "artifact_digest"));
	if (artifact.artifact_digest !== digest(unsigned)) throw new Error("KAMN Agent C observation digest mismatch");
	return artifact as Observation;
}

function assertPolicy(artifact: Observation) {
	if (artifact.state !== "accepted") throw new Error("KAMN Agent C observation state must be accepted");
	if (artifact.view_scope !== "restricted-public" || artifact.private_field_count !== 0 || artifact.private_payload_redacted !== true) {
		throw new Error("KAMN Agent C observation must remain restricted-public with no private fields");
	}
	assertThirdProcess(artifact.agent_c_pi_process_id, 0, 0);
	for (const value of [artifact.source_handoff_digest, artifact.source_agent_a_receipt_digest, artifact.source_agent_b_receipt_digest]) {
		if (!/^[a-f0-9]{64}$/.test(value)) throw new Error("KAMN Agent C observation source digest is invalid");
	}
	const expected = digest({
		task_id: artifact.task_id, state: artifact.state, source_handoff_digest: artifact.source_handoff_digest,
		source_agent_a_receipt_digest: artifact.source_agent_a_receipt_digest,
		source_agent_b_receipt_digest: artifact.source_agent_b_receipt_digest,
	});
	if (artifact.public_commitment !== expected) throw new Error("KAMN Agent C public commitment mismatch");
}

function assertThirdProcess(agentCPid: number, agentAPid: number, agentBPid: number) {
	if (!Number.isInteger(agentCPid) || agentCPid <= 0 || agentCPid === agentAPid || agentCPid === agentBPid) {
		throw new Error("KAMN Agent C must run in a third distinct Pi process");
	}
}

async function writeIdempotent(path: string, artifact: Record<string, unknown>) {
	const json = `${JSON.stringify(artifact)}\n`;
	try { await writeFile(path, json, { flag: "wx", mode: 0o600 }); } catch (error) {
		if (!isNodeError(error, "EEXIST")) throw error;
		if (await readFile(path, "utf8") !== json) throw new Error("KAMN Agent C observation conflicts with existing artifact");
	}
}

function configuredPath(env: Environment, name: string, cwd: string): string {
	const value = env[name]?.trim();
	if (!value) throw new Error(`Missing required environment variable: ${name}`);
	const path = value.startsWith("/") ? value : resolve(cwd, value);
	assertSafePath(path);
	return path;
}

function assertSafePath(path: string) {
	const lower = path.toLowerCase();
	if (SECRET_MARKERS.some((marker) => lower.includes(marker))) throw new Error("Refusing secret-like KAMN Agent C observation path");
}

function assertDistinctOutput(outputPath: string, sourcePaths: string[]) {
	const output = resolve(outputPath);
	if (sourcePaths.some((path) => resolve(path) === output)) {
		throw new Error("KAMN Agent C observation path must differ from source artifacts");
	}
}

function digest(value: Record<string, unknown>): string {
	return createHash("sha256").update(JSON.stringify(value)).digest("hex");
}

function isNodeError(error: unknown, code: string): boolean {
	return error instanceof Error && "code" in error && error.code === code;
}
